use async_compat::CompatExt;
use async_imap::imap_proto::Address;
use async_imap::types::{Fetch, Flag};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use mail_parser::{MessageParser, MimeHeaders};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Mutex;
use tokio_native_tls::TlsStream;
use uuid::Uuid;

use crate::database::models::email::EmailAddress;
use crate::sync::{
    auth::CredentialStore,
    error::{SyncError, SyncResult},
    provider::EmailProvider,
    types::*,
};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

// Wrapper to provide Debug trait for Compat
struct DebugCompat(async_compat::Compat<TlsStream<tokio::net::TcpStream>>);

impl std::fmt::Debug for DebugCompat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DebugCompat(TlsStream)")
    }
}

impl futures::io::AsyncRead for DebugCompat {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl futures::io::AsyncWrite for DebugCompat {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_close(cx)
    }
}

type ImapSession = async_imap::Session<DebugCompat>;

pub struct ImapProvider {
    account_id: Uuid,
    session: Arc<Mutex<Option<ImapSession>>>,
    config: Arc<Mutex<Option<ImapConfig>>>,
    account_settings: Option<AccountSettings>,
    credential_store: Arc<CredentialStore>,
}

#[derive(Debug, Clone)]
struct ImapConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    use_tls: bool,
}

impl ImapProvider {
    pub fn new(account_id: Uuid, credential_store: Arc<CredentialStore>) -> SyncResult<Self> {
        Ok(Self {
            account_id,
            session: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(None)),
            account_settings: None,
            credential_store,
        })
    }

    pub fn with_settings(mut self, settings: AccountSettings) -> Self {
        self.account_settings = Some(settings);
        self
    }

    async fn ensure_connected(&self) -> SyncResult<()> {
        // First, ensure config is loaded
        {
            let config_guard = self.config.lock().await;
            if config_guard.is_none() {
                drop(config_guard);

                log::info!(
                    "[ImapProvider] Config not set, loading credentials for account {}",
                    self.account_id
                );

                let settings = self.account_settings.as_ref().ok_or_else(|| {
                    SyncError::InvalidConfiguration("Account settings not provided".to_string())
                })?;

                let host = settings.imap_host.as_ref().ok_or_else(|| {
                    SyncError::InvalidConfiguration("IMAP host not configured".to_string())
                })?;

                let port = settings.imap_port.unwrap_or(993);
                let use_tls = settings.imap_use_tls.unwrap_or(true);

                let creds = self
                    .credential_store
                    .get_imap(self.account_id)
                    .await
                    .map_err(|e| {
                        log::error!(
                            "[ImapProvider] Failed to load credentials for account {}: {}",
                            self.account_id,
                            e
                        );
                        SyncError::InvalidConfiguration(format!(
                            "Failed to load IMAP credentials: {}",
                            e
                        ))
                    })?;

                let mut config_guard = self.config.lock().await;
                *config_guard = Some(ImapConfig {
                    host: host.clone(),
                    port,
                    username: creds.username,
                    password: creds.password,
                    use_tls,
                });

                log::info!(
                    "[ImapProvider] Config initialized for account {} ({}:{})",
                    self.account_id,
                    host,
                    port
                );
            }
        }

        let mut session = self.session.lock().await;

        if session.is_none() {
            let config_guard = self.config.lock().await;
            let config = config_guard.as_ref().unwrap(); // Safe because we just ensured it's Some above

            let addr = format!("{}:{}", config.host, config.port);

            let tcp_stream = tokio::net::TcpStream::connect(&addr)
                .await
                .map_err(|e| SyncError::ImapError(format!("TCP connection failed: {}", e)))?;

            if config.use_tls {
                let tls_connector = tokio_native_tls::native_tls::TlsConnector::builder()
                    .build()
                    .map_err(|e| SyncError::ImapError(format!("TLS setup failed: {}", e)))?;
                let tls_connector = tokio_native_tls::TlsConnector::from(tls_connector);

                let tls_stream = tls_connector
                    .connect(&config.host, tcp_stream)
                    .await
                    .map_err(|e| SyncError::ImapError(format!("TLS connection failed: {}", e)))?;

                let client = async_imap::Client::new(DebugCompat(tls_stream.compat()));
                let imap_session = client
                    .login(&config.username, &config.password)
                    .await
                    .map_err(|e| {
                        SyncError::AuthenticationError(format!("IMAP login failed: {:?}", e))
                    })?;

                *session = Some(imap_session);
                log::info!(
                    "[ImapProvider] IMAP connection established for account {}",
                    self.account_id
                );
            } else {
                return Err(SyncError::InvalidConfiguration(
                    "Non-TLS IMAP connections are not supported".to_string(),
                ));
            }
        }

        Ok(())
    }

    async fn get_session(&self) -> SyncResult<tokio::sync::MutexGuard<'_, Option<ImapSession>>> {
        self.ensure_connected().await?;
        Ok(self.session.lock().await)
    }

    fn map_folder_type(name: &str, _attributes: &[async_imap::types::NameAttribute]) -> FolderType {
        // 1) if attributes contain special-use hints, prefer them
        for attr in _attributes.iter() {
            let s = format!("{:?}", attr).to_lowercase();

            log::debug!("Checking folder {} attribute for type mapping: {}", name, s);

            if s.contains("sent") || s.contains("\"sent\"") || s.contains("\"$sent\"") {
                return FolderType::Sent;
            }
            if s.contains("draft") || s.contains("\"draft\"") {
                return FolderType::Draft;
            }
            if s.contains("trash") || s.contains("deleted") || s.contains("\"$trash\"") {
                return FolderType::Trash;
            }
            if s.contains("spam") || s.contains("junk") {
                return FolderType::Spam;
            }
            if s.contains("archive") || s.contains("\"archive\"") {
                return FolderType::Archive;
            }
            if s.contains("flagged") || s.contains("starred") || s.contains("\"$flagged\"") {
                return FolderType::Starred;
            }
            if s.contains("inbox") {
                return FolderType::Inbox;
            }
        }

        // 2) extract base name (strip any path prefixes like INBOX/ or Mail/)
        let remote = name;
        let sep_pos = match (remote.rfind('/'), remote.rfind('.')) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            _ => None,
        };
        let base = if let Some(pos) = sep_pos {
            &remote[pos + 1..]
        } else {
            remote
        };

        // 3) decode IMAP modified UTF-7 if necessary and normalize
        let decoded = Self::decode_modified_utf7(base);
        let norm = Self::normalize_name(&decoded);

        log::debug!(
            "Mapping folder name '{}' (decoded: '{}', normalized: '{}')",
            name,
            decoded,
            norm
        );

        // 4) english-first checks
        if norm.contains("inbox") {
            return FolderType::Inbox;
        }
        if norm.contains("sent") || norm.contains("sentitems") || norm.contains("sent-mail") {
            return FolderType::Sent;
        }
        if norm.contains("draft") || norm.contains("drafts") {
            return FolderType::Draft;
        }
        if norm.contains("trash") || norm.contains("deleted") || norm.contains("bin") {
            return FolderType::Trash;
        }
        if norm.contains("spam") || norm.contains("junk") {
            return FolderType::Spam;
        }
        if norm.contains("archive") || norm.contains("archiv") {
            return FolderType::Archive;
        }
        if norm.contains("star")
            || norm.contains("flag")
            || norm.contains("favourite")
            || norm.contains("favorite")
        {
            return FolderType::Starred;
        }

        // 5) localized heuristics (examples: German and a few common variants)
        // German
        if norm.contains("gesend") {
            // Gesendet, Gesendete Elemente
            return FolderType::Sent;
        }
        if norm.contains("entw") || norm.contains("entwurf") {
            // Entwürfe -> Drafts
            return FolderType::Draft;
        }
        if norm.contains("papierkorb") || norm.contains("geloscht") || norm.contains("gelöscht") {
            return FolderType::Trash;
        }
        // French
        if norm.contains("envoy") {
            // envoy -> envoyé
            return FolderType::Sent;
        }
        if norm.contains("brouillon") {
            // brouillons
            return FolderType::Draft;
        }
        // Spanish
        if norm.contains("enviado") || norm.contains("enviados") {
            return FolderType::Sent;
        }
        if norm.contains("borrador") || norm.contains("borradores") {
            return FolderType::Draft;
        }

        // fallback: custom
        FolderType::Custom
    }

    // Decode IMAP modified UTF-7 (associated impl function). Handles &...- and
    // performs modified base64 normalization (',' -> '/') and padding.
    fn decode_modified_utf7(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '&' {
                if let Some(&next) = chars.peek() {
                    if next == '-' {
                        chars.next();
                        out.push('&');
                        continue;
                    }
                }

                let mut b64 = String::new();
                while let Some(c) = chars.next() {
                    if c == '-' {
                        break;
                    }
                    b64.push(c);
                }

                let mut b64_mod = b64.replace(',', "/");
                let rem = b64_mod.len() % 4;
                if rem != 0 {
                    b64_mod.push_str(&"=".repeat(4 - rem));
                }
                if b64_mod.is_empty() {
                    continue;
                }

                if let Ok(bytes) = STANDARD.decode(&b64_mod) {
                    if bytes.len() % 2 != 0 {
                        out.push_str(&String::from_utf8_lossy(&bytes));
                        continue;
                    }
                    let mut u16_buf = Vec::with_capacity(bytes.len() / 2);
                    let mut i = 0;
                    while i + 1 < bytes.len() {
                        let hi = (bytes[i] as u16) << 8;
                        let lo = bytes[i + 1] as u16;
                        u16_buf.push(hi | lo);
                        i += 2;
                    }
                    if let Ok(s) = String::from_utf16(&u16_buf) {
                        out.push_str(&s);
                    } else {
                        out.push_str(&String::from_utf8_lossy(&bytes));
                    }
                }
            } else {
                out.push(ch);
            }
        }

        out
    }

    // Lightweight normalization for mailbox names: lowercase, trim, simple
    // diacritic folding for common characters (German, a few Latin accents).
    fn normalize_name(s: &str) -> String {
        let mut t = s.to_lowercase();
        t = t.trim().to_string();
        t = t.replace('ä', "a");
        t = t.replace('ö', "o");
        t = t.replace('ü', "u");
        t = t.replace('ß', "ss");
        t = t.replace('é', "e").replace('è', "e").replace('ê', "e");
        t = t.replace('à', "a").replace('á', "a");
        t = t.replace('ó', "o").replace('ò', "o");
        t
    }

    fn parse_email_headers(
        fetch: &Fetch,
        folder_id: Uuid,
        account_id: Uuid,
        fallback_uid: Option<u32>,
    ) -> SyncResult<SyncEmail> {
        log::debug!("Parsing email headers with UID: {:?}", fetch.uid);

        let uid = fetch.uid.or(fallback_uid).ok_or_else(|| {
            SyncError::ParseError("UID not present in fetch response".to_string())
        })?;

        let envelope = fetch.envelope().ok_or_else(|| {
            SyncError::ParseError("Envelope not found in fetch response".to_string())
        })?;

        let from = envelope
            .from
            .as_ref()
            .and_then(|addrs| addrs.first())
            .map(|addr| EmailAddress {
                name: addr
                    .name
                    .as_ref()
                    .map(|n| String::from_utf8_lossy(n).to_string()),
                address: addr
                    .mailbox
                    .as_ref()
                    .and_then(|m| {
                        addr.host.as_ref().map(|h| {
                            format!(
                                "{}@{}",
                                String::from_utf8_lossy(m),
                                String::from_utf8_lossy(h)
                            )
                        })
                    })
                    .unwrap_or_else(|| "unknown@unknown.com".to_string()),
            })
            .unwrap_or_else(|| EmailAddress {
                name: None,
                address: "unknown@unknown.com".to_string(),
            });

        let parse_addresses = |addrs: &Option<Vec<_>>| -> Vec<EmailAddress> {
            addrs
                .as_ref()
                .map(|list| {
                    list.iter()
                        .filter_map(|addr: &Address| {
                            let mailbox = addr.mailbox.as_ref()?;
                            let host = addr.host.as_ref()?;
                            Some(EmailAddress {
                                name: addr
                                    .name
                                    .as_ref()
                                    .map(|n| String::from_utf8_lossy(n).to_string()),
                                address: format!(
                                    "{}@{}",
                                    String::from_utf8_lossy(mailbox),
                                    String::from_utf8_lossy(host)
                                ),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        let to = parse_addresses(&envelope.to);
        let cc = parse_addresses(&envelope.cc);
        let bcc = parse_addresses(&envelope.bcc);
        let reply_to = envelope
            .reply_to
            .as_ref()
            .and_then(|addrs| addrs.first())
            .and_then(|addr| {
                let mailbox = addr.mailbox.as_ref()?;
                let host = addr.host.as_ref()?;
                Some(EmailAddress {
                    name: addr
                        .name
                        .as_ref()
                        .map(|n| String::from_utf8_lossy(n).to_string()),
                    address: format!(
                        "{}@{}",
                        String::from_utf8_lossy(mailbox),
                        String::from_utf8_lossy(host)
                    ),
                })
            });

        let subject = envelope
            .subject
            .as_ref()
            .map(|s| String::from_utf8_lossy(s).to_string());

        let message_id = envelope
            .message_id
            .as_ref()
            .map(|id| String::from_utf8_lossy(id).to_string())
            .unwrap_or_else(|| format!("{}@{}", uid, "imap.local"));

        // Parse the date for received_at and sent_at
        let received_at = envelope
            .date
            .as_ref()
            .and_then(|date_bytes| {
                let date_str = String::from_utf8_lossy(date_bytes);
                chrono::DateTime::parse_from_rfc2822(&date_str).ok()
            })
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| Utc::now());

        // For sent_at, use the date from envelope (when the email was sent)
        // This is especially important for sent folders
        let sent_at = envelope
            .date
            .as_ref()
            .and_then(|date_bytes| {
                let date_str = String::from_utf8_lossy(date_bytes);
                chrono::DateTime::parse_from_rfc2822(&date_str).ok()
            })
            .map(|dt| dt.with_timezone(&Utc));

        let flags: Vec<String> = fetch
            .flags()
            .map(|flag| match flag {
                Flag::Seen => "\\Seen".to_string(),
                Flag::Answered => "\\Answered".to_string(),
                Flag::Flagged => "\\Flagged".to_string(),
                Flag::Deleted => "\\Deleted".to_string(),
                Flag::Draft => "\\Draft".to_string(),
                Flag::Recent => "\\Recent".to_string(),
                Flag::Custom(s) => s.to_string(),
                _ => String::new(),
            })
            .filter(|s| !s.is_empty())
            .collect();

        let size = fetch.size.unwrap_or(0) as i64;

        // Extract basic headers from envelope for headers-only mode
        let headers_json = {
            let mut headers_map = serde_json::Map::new();
            if let Some(from) = envelope.from.as_ref().and_then(|addrs| addrs.first()) {
                if let Some(name) = from.name.as_ref() {
                    headers_map.insert(
                        "From-Name".to_string(),
                        serde_json::Value::String(String::from_utf8_lossy(name).to_string()),
                    );
                }
            }
            if let Some(subj) = &subject {
                headers_map.insert(
                    "Subject".to_string(),
                    serde_json::Value::String(subj.clone()),
                );
            }
            if let Some(msg_id) = envelope.message_id.as_deref() {
                headers_map.insert(
                    "Message-ID".to_string(),
                    serde_json::Value::String(String::from_utf8_lossy(msg_id).to_string()),
                );
            }
            if let Some(in_reply_to) = envelope.in_reply_to.as_ref() {
                headers_map.insert(
                    "In-Reply-To".to_string(),
                    serde_json::Value::String(String::from_utf8_lossy(in_reply_to).to_string()),
                );
            }
            Some(serde_json::Value::Object(headers_map))
        };

        // Leave snippet as None for headers-only mode
        // It will be populated during body fetch
        let snippet = None;

        // For headers-only mode, check BODYSTRUCTURE for attachments
        // This allows us to set has_attachments without fetching the full body
        let has_attachments = fetch
            .bodystructure()
            .map(|_bs| {
                // Check if bodystructure indicates attachments
                // This is a simplified check - a full implementation would parse the structure
                // For now, we'll be conservative and set to false
                false
            })
            .unwrap_or(false);

        Ok(SyncEmail {
            id: None,
            account_id,
            folder_id,
            message_id,
            conversation_id: None,
            remote_id: uid.to_string(),
            from,
            to,
            cc,
            bcc,
            reply_to,
            subject,
            snippet,
            body_plain: None,
            body_html: None,
            ai_cache: None,
            other_mails: None,
            category: None,
            received_at,
            sent_at,
            flags,
            headers: headers_json,
            size,
            has_attachments,
            attachments: Vec::new(),
            change_key: None,
            last_modified_at: None,
        })
    }

    fn parse_email(
        fetch: &Fetch,
        folder_id: Uuid,
        account_id: Uuid,
        fallback_uid: Option<u32>,
    ) -> SyncResult<SyncEmail> {
        // log::debug!(
        //     "Parsing email with attributes: {:?}",
        //     fetch.body()
        // );

        // Get UID: prefer server-provided one, fall back to UID from SEARCH result
        let uid = fetch.uid.or(fallback_uid).ok_or_else(|| {
            SyncError::ParseError("UID not present in fetch response".to_string())
        })?;

        let body = fetch
            .body()
            .ok_or_else(|| SyncError::ParseError("Email body not found".to_string()))?;

        let parser = MessageParser::default();
        let message = parser
            .parse(body)
            .ok_or_else(|| SyncError::ParseError("Failed to parse email".to_string()))?;

        // Extract fields
        let from = message
            .from()
            .and_then(|addr| addr.first())
            .map(|addr| EmailAddress {
                name: addr.name().map(|n| n.to_string()),
                address: addr.address().unwrap_or("").to_string(),
            })
            .unwrap_or_else(|| EmailAddress {
                name: None,
                address: "unknown@unknown.com".to_string(),
            });

        let to: Vec<EmailAddress> = message
            .to()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| EmailAddress {
                        name: addr.name().map(|n| n.to_string()),
                        address: addr.address().unwrap_or("").to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let cc: Vec<EmailAddress> = message
            .cc()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| EmailAddress {
                        name: addr.name().map(|n| n.to_string()),
                        address: addr.address().unwrap_or("").to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let bcc: Vec<EmailAddress> = message
            .bcc()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| EmailAddress {
                        name: addr.name().map(|n| n.to_string()),
                        address: addr.address().unwrap_or("").to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let reply_to = message
            .reply_to()
            .and_then(|addrs| addrs.first())
            .map(|addr| EmailAddress {
                name: addr.name().map(|n| n.to_string()),
                address: addr.address().unwrap_or("").to_string(),
            });

        let subject = message.subject().map(|s| s.to_string());

        let body_html = message.body_html(0).map(|s| s.to_string());
        let body_plain = message.body_text(0).map(|s| s.to_string());

        let message_id = message
            .message_id()
            .map(|id| id.to_string())
            .unwrap_or_else(|| format!("{}@{}", uid, "imap.local"));

        let received_at = message
            .date()
            .and_then(|ts| DateTime::from_timestamp(ts.to_timestamp(), 0))
            .unwrap_or_else(|| Utc::now());

        // For sent_at, use the same date (when the email was sent)
        let sent_at = message
            .date()
            .and_then(|ts| DateTime::from_timestamp(ts.to_timestamp(), 0));

        let flags: Vec<String> = fetch
            .flags()
            .map(|flag| match flag {
                Flag::Seen => "\\Seen".to_string(),
                Flag::Answered => "\\Answered".to_string(),
                Flag::Flagged => "\\Flagged".to_string(),
                Flag::Deleted => "\\Deleted".to_string(),
                Flag::Draft => "\\Draft".to_string(),
                Flag::Recent => "\\Recent".to_string(),
                Flag::Custom(s) => s.to_string(),
                _ => String::new(),
            })
            .filter(|s| !s.is_empty())
            .collect();

        let attachments: Vec<SyncAttachment> = message
            .attachments()
            .map(|att| {
                let content = att.contents();
                let hash = format!("{:x}", md5::compute(content));
                let content_id = att.content_id().map(|s| s.to_string());

                // Determine if attachment is inline by checking if its Content-ID
                // is actually referenced in the HTML body (e.g., src="cid:xxx")
                let is_inline = if let (Some(cid), Some(html)) = (&content_id, &body_html) {
                    crate::sync::cid_utils::is_cid_referenced(html, cid)
                } else {
                    false
                };

                SyncAttachment {
                    id: None,
                    email_id: None,
                    filename: att.attachment_name().unwrap_or("attachment").to_string(),
                    content_type: att
                        .content_type()
                        .map(|ct| ct.ctype())
                        .unwrap_or("application/octet-stream")
                        .to_string(),
                    size: content.len() as i64,
                    hash,
                    cache_path: None,
                    remote_url: None,
                    remote_path: None,
                    is_inline,
                    is_cached: false,
                    content_id,
                    data: Some(content.to_vec()), // Include data for immediate caching
                }
            })
            .collect();

        let has_attachments = !attachments.is_empty();

        // Use snippet extraction utility for proper trimming at word boundary
        let snippet = crate::sync::snippet_utils::extract_snippet(body_plain.as_deref());

        log::debug!("[Imap] Extracted snippet for UID {}: {:?}", uid, snippet);

        // Extract comprehensive headers as JSON (including DKIM, List-*, Return-Path, etc.)
        let headers_json = {
            let headers_map = serde_json::Map::new();
            // for header in message.headers().iter() {
            //     let value_str = String::from_utf8_lossy(header.value.as_text().unwrap().as_ref()).to_string();
            //     headers_map.insert(header.name.to_string(), serde_json::Value::String(value_str));
            // }
            Some(serde_json::Value::Object(headers_map))
        };

        Ok(SyncEmail {
            id: None,
            account_id,
            folder_id,
            message_id,
            conversation_id: None,
            remote_id: uid.to_string(),
            from,
            to,
            cc,
            bcc,
            reply_to,
            subject,
            snippet,
            body_plain,
            body_html,
            other_mails: None,
            category: None,
            ai_cache: None,
            received_at,
            sent_at,
            flags,
            headers: headers_json,
            size: body.len() as i64,
            has_attachments,
            attachments,
            change_key: None,
            last_modified_at: None,
        })
    }

    /// Helper to fetch messages with whole bodies using sequence numbers.
    /// UID can't be fetched reliably; we only request full RFC822 and FLAGS.
    async fn fetch_messages_with_bodies(
        session: &mut ImapSession,
        seqset: &str,
        _use_uid: bool, // kept for compatibility; ignored
    ) -> SyncResult<Vec<Fetch>> {
        let fetch_attrs = "RFC822";
        let messages: Vec<_> = session
            .fetch(seqset, fetch_attrs)
            .await?
            .try_collect()
            .await?;
        Ok(messages)
    }

    /// Helper to fetch message headers only (no body) using sequence numbers.
    /// Fetches: ENVELOPE, FLAGS, RFC822.SIZE, BODYSTRUCTURE
    async fn fetch_messages_headers_only(
        session: &mut ImapSession,
        seqset: &str,
    ) -> SyncResult<Vec<Fetch>> {
        let fetch_attrs = "(FLAGS ENVELOPE RFC822.SIZE BODYSTRUCTURE)";
        let messages: Vec<_> = session
            .fetch(seqset, fetch_attrs)
            .await?
            .try_collect()
            .await?;
        Ok(messages)
    }

    /// Fetch email headers only (without body) for incremental or full sync
    ///
    /// When since_uid is provided, fetches only headers after that UID (incremental).
    /// When since_uid is None, fetches all headers with pagination (full sync).
    pub async fn fetch_email_headers(
        &self,
        folder: &SyncFolder,
        since_uid: Option<u32>,
    ) -> SyncResult<Vec<SyncEmail>> {
        log::debug!(
            "[IMAP] Starting header-only fetch for folder: {}",
            folder.name
        );

        let mut session_guard = self.get_session().await?;
        let session = session_guard
            .as_mut()
            .ok_or_else(|| SyncError::ImapError("No active session".to_string()))?;

        let mailbox = session.select(&folder.remote_id).await?;
        let exists = mailbox.exists;

        log::debug!(
            "[IMAP] Selected folder {} - {} messages exist",
            folder.remote_id,
            exists
        );

        if exists == 0 {
            log::info!("[IMAP] Folder {} is empty", folder.name);
            return Ok(Vec::new());
        }

        let uids: Vec<u32> = if let Some(uid) = since_uid {
            log::debug!(
                "[IMAP] UID SEARCH for folder {}: UID {}:*",
                folder.remote_id,
                uid + 1
            );
            let set = session.uid_search(format!("UID {}:*", uid + 1)).await?;
            let mut v: Vec<u32> = set.into_iter().collect();
            v.sort_unstable();
            v
        } else {
            log::debug!("[IMAP] UID SEARCH for folder {}: ALL", folder.remote_id);
            let set = session.uid_search("ALL").await?;
            let mut v: Vec<u32> = set.into_iter().collect();
            v.sort_unstable();
            v
        };

        if uids.is_empty() {
            log::info!("[IMAP] No matching UIDs in folder {}", folder.name);
            return Ok(Vec::new());
        }

        log::debug!("[IMAP] Fetching headers for {} emails", uids.len());

        let mut emails: Vec<SyncEmail> = Vec::new();
        for uid in uids {
            let seq_nums_set = session.search(format!("UID {}", uid)).await?;
            let seq = seq_nums_set.iter().min().copied();
            if seq.is_none() {
                continue;
            }
            let seq = seq.unwrap();

            let messages = Self::fetch_messages_headers_only(session, &seq.to_string()).await?;
            if let Some(fetch) = messages.first() {
                if let Some(folder_id) = folder.id {
                    match Self::parse_email_headers(fetch, folder_id, self.account_id, Some(uid)) {
                        Ok(email) => emails.push(email),
                        Err(e) => {
                            log::warn!("[IMAP] Failed to parse email headers UID {}: {}", uid, e)
                        }
                    }
                }
            }
        }

        log::info!(
            "[IMAP] Successfully fetched headers for {} emails from folder {} for account {}",
            emails.len(),
            folder.name,
            self.account_id
        );

        Ok(emails)
    }

    /// Fetch the full body for an email that only has headers
    /// Returns: (body_plain, body_html, headers, sent_at, attachments)
    pub async fn fetch_email_body(
        &self,
        folder: &SyncFolder,
        remote_id: &str,
    ) -> SyncResult<(
        Option<String>,
        Option<String>,
        Option<serde_json::Value>,
        Option<DateTime<Utc>>,
        Vec<SyncAttachment>,
        Option<String>,
    )> {
        log::debug!(
            "[IMAP] Fetching body for email {} in folder {}",
            remote_id,
            folder.name
        );

        let email = self.fetch_email(folder, remote_id).await?;

        log::debug!("[IMAP] Successfully fetched body for email {}", remote_id);

        Ok((
            email.body_plain,
            email.body_html,
            email.headers,
            email.sent_at,
            email.attachments,
            email.snippet,
        ))
    }
}

#[async_trait]
impl EmailProvider for ImapProvider {
    fn name(&self) -> &str {
        "IMAP"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn authenticate(&mut self, credentials: ProviderCredentials) -> SyncResult<()> {
        match credentials {
            ProviderCredentials::Imap(creds) => {
                // Get IMAP settings from account settings
                let settings = self.account_settings.as_ref().ok_or_else(|| {
                    SyncError::InvalidConfiguration("Account settings not provided".to_string())
                })?;

                let host = settings.imap_host.as_ref().ok_or_else(|| {
                    SyncError::InvalidConfiguration("IMAP host not configured".to_string())
                })?;

                let port = settings.imap_port.unwrap_or(993);
                let use_tls = settings.imap_use_tls.unwrap_or(true);

                let mut config_guard = self.config.lock().await;
                *config_guard = Some(ImapConfig {
                    host: host.clone(),
                    port,
                    username: creds.username.clone(),
                    password: creds.password.clone(),
                    use_tls,
                });
                drop(config_guard);

                // Test connection
                self.ensure_connected().await?;

                // Store credentials
                self.credential_store
                    .store_imap(self.account_id, &creds)
                    .await?;

                log::info!(
                    "IMAP authentication successful for account {} ({}:{})",
                    self.account_id,
                    host,
                    port
                );

                Ok(())
            }
            _ => Err(SyncError::InvalidConfiguration(
                "IMAP provider requires IMAP credentials".to_string(),
            )),
        }
    }

    async fn test_connection(&self) -> SyncResult<bool> {
        self.ensure_connected().await?;
        Ok(true)
    }

    async fn fetch_folders(&self) -> SyncResult<Vec<SyncFolder>> {
        let mut session_guard = self.get_session().await?;
        let session = session_guard
            .as_mut()
            .ok_or_else(|| SyncError::ImapError("No active session".to_string()))?;

        let folders = session
            .list(Some(""), Some("*"))
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        let sync_folders: Vec<SyncFolder> = folders
            .iter()
            .map(|folder| {
                let remote_id = folder.name();
                let name = decode_modified_utf7(remote_id);
                let folder_type = Self::map_folder_type(&*name, folder.attributes());
                let attributes: Vec<String> = folder
                    .attributes()
                    .iter()
                    .map(|attr| format!("{:?}", attr))
                    .collect();

                SyncFolder {
                    id: None,
                    account_id: self.account_id,
                    name: name.to_string(),
                    folder_type,
                    icon: None,
                    color: None,
                    synced_at: None,
                    sync_interval: 0,
                    remote_id: remote_id.to_string(),
                    parent_id: None,
                    attributes,
                    unread_count: 0,
                    total_count: 0,
                    expanded: false,
                    hidden: false,
                }
            })
            .collect();

        log::info!(
            "Fetched {} folders for account {}",
            sync_folders.len(),
            self.account_id
        );

        Ok(sync_folders)
    }

    async fn sync_messages(
        &self,
        folder: &SyncFolder,
        sync_token: Option<String>,
    ) -> SyncResult<crate::sync::types::SyncDiff> {
        // Parse sync_token (last UID) into since_uid for incremental sync
        let since_uid = sync_token.and_then(|t| t.parse::<u32>().ok());

        let mut session_guard = self.get_session().await?;
        let session = session_guard
            .as_mut()
            .ok_or_else(|| SyncError::ImapError("No active session".to_string()))?;

        // Select folder and get mailbox info
        let mailbox = session.select(&folder.remote_id).await?;
        let exists = mailbox.exists;

        log::debug!(
            "Selected folder {} - {} messages exist",
            folder.remote_id,
            exists
        );

        if exists == 0 {
            log::info!("Folder {} is empty", folder.name);
            return Ok(crate::sync::types::SyncDiff {
                added: Vec::new(),
                modified: Vec::new(),
                deleted: Vec::new(),
                next_sync_token: None,
            });
        }

        // 1) Use SEARCH to get candidate UIDs (HashSet -> sorted Vec)
        let uids: Vec<u32> = if let Some(uid) = since_uid {
            log::debug!(
                "UID SEARCH for folder {}: UID {}:*",
                folder.remote_id,
                uid + 1
            );
            let set = session.uid_search(format!("UID {}:*", uid + 1)).await?;
            let mut v: Vec<u32> = set.into_iter().collect();
            v.sort_unstable();
            v
        } else {
            log::debug!("UID SEARCH for folder {}: ALL", folder.remote_id);
            let set = session.uid_search("ALL").await?;
            let mut v: Vec<u32> = set.into_iter().collect();
            v.sort_unstable();
            v
        };

        if uids.is_empty() {
            log::info!("No matching UIDs in folder {}", folder.name);
            return Ok(crate::sync::types::SyncDiff {
                added: Vec::new(),
                modified: Vec::new(),
                deleted: Vec::new(),
                next_sync_token: None,
            });
        }

        log::debug!("Fetching {} emails", uids.len());

        // 2) Iterate per UID: map to seq num, fetch by seq, and parse using fallback UID
        let mut emails: Vec<SyncEmail> = Vec::new();
        for uid in uids {
            let seq_nums_set = session.search(format!("UID {}", uid)).await?;
            let seq = seq_nums_set.iter().min().copied();
            if seq.is_none() {
                continue;
            }
            let seq = seq.unwrap();

            let messages =
                Self::fetch_messages_with_bodies(session, &seq.to_string(), false).await?;
            if let Some(fetch) = messages.first() {
                if let Some(folder_id) = folder.id {
                    match Self::parse_email(fetch, folder_id, self.account_id, Some(uid)) {
                        Ok(email) => emails.push(email),
                        Err(e) => log::warn!("Failed to parse email UID {}: {}", uid, e),
                    }
                }
            }
        }

        log::info!(
            "Successfully parsed {} emails from folder {} for account {}",
            emails.len(),
            folder.name,
            self.account_id
        );

        // Get the highest UID from fetched emails for next sync
        let next_token = emails
            .iter()
            .filter_map(|e| e.last_modified_at)
            .max()
            .map(|dt| dt.timestamp().to_string());

        Ok(crate::sync::types::SyncDiff {
            added: emails,
            modified: Vec::new(),
            deleted: Vec::new(),
            next_sync_token: next_token,
        })
    }

    async fn fetch_email(&self, folder: &SyncFolder, remote_id: &str) -> SyncResult<SyncEmail> {
        let mut session_guard = self.get_session().await?;
        let session = session_guard
            .as_mut()
            .ok_or_else(|| SyncError::ImapError("No active session".to_string()))?;

        // Read-only is fine
        session.examine(&folder.remote_id).await?;

        let uid: u32 = remote_id
            .parse()
            .map_err(|_| SyncError::ParseError("Invalid UID".to_string()))?;

        // Map UID -> sequence number via SEARCH (returns HashSet)
        let seq_nums_set = session.search(format!("UID {}", uid)).await?;
        let seq = seq_nums_set
            .iter()
            .min()
            .copied()
            .ok_or_else(|| SyncError::EmailNotFound(remote_id.to_string()))?;

        // Fetch by sequence number, parse using fallback UID from the request
        let messages = Self::fetch_messages_with_bodies(session, &seq.to_string(), false).await?;
        let fetch = messages
            .first()
            .ok_or_else(|| SyncError::EmailNotFound(remote_id.to_string()))?;

        let folder_id = folder
            .id
            .ok_or_else(|| SyncError::DatabaseError("Folder ID is required".to_string()))?;

        Self::parse_email(fetch, folder_id, self.account_id, Some(uid))
    }

    async fn fetch_attachment(&self, _attachment: &SyncAttachment) -> SyncResult<Vec<u8>> {
        // For IMAP, attachments are already fetched with the email
        // This method would re-fetch the email if needed
        Err(SyncError::NotSupported(
            "IMAP attachments are fetched with email".to_string(),
        ))
    }

    async fn move_email(
        &self,
        email_remote_id: &str,
        from_folder: &SyncFolder,
        to_folder: &SyncFolder,
    ) -> SyncResult<()> {
        let mut session_guard = self.get_session().await?;
        let session = session_guard
            .as_mut()
            .ok_or_else(|| SyncError::ImapError("No active session".to_string()))?;

        session.select(&from_folder.remote_id).await?;

        let uid: u32 = email_remote_id
            .parse()
            .map_err(|_| SyncError::ParseError("Invalid UID".to_string()))?;

        // Copy to destination
        session
            .uid_copy(uid.to_string(), &to_folder.remote_id)
            .await?;

        // Mark as deleted in source
        let _ = session
            .uid_store(uid.to_string(), "+FLAGS (\\Deleted)")
            .await?;

        // Expunge to permanently remove
        let _ = session.expunge().await?;

        log::info!(
            "Moved email {} from {} to {}",
            email_remote_id,
            from_folder.name,
            to_folder.name
        );

        Ok(())
    }

    async fn delete_email(
        &self,
        email_remote_id: &str,
        folder: &SyncFolder,
        permanent: bool,
    ) -> SyncResult<()> {
        let mut session_guard = self.get_session().await?;
        let session = session_guard
            .as_mut()
            .ok_or_else(|| SyncError::ImapError("No active session".to_string()))?;

        session.select(&folder.remote_id).await?;

        let uid: u32 = email_remote_id
            .parse()
            .map_err(|_| SyncError::ParseError("Invalid UID".to_string()))?;

        let _ = session
            .uid_store(uid.to_string(), "+FLAGS (\\Deleted)")
            .await?;

        if permanent {
            let _ = session.expunge().await?;
        }

        log::info!("Deleted email {} from {}", email_remote_id, folder.name);

        Ok(())
    }

    async fn mark_as_read(
        &self,
        email_remote_id: &str,
        folder: &SyncFolder,
        is_read: bool,
    ) -> SyncResult<()> {
        let mut session_guard = self.get_session().await?;
        let session = session_guard
            .as_mut()
            .ok_or_else(|| SyncError::ImapError("No active session".to_string()))?;

        session.select(&folder.remote_id).await?;

        let uid: u32 = email_remote_id
            .parse()
            .map_err(|_| SyncError::ParseError("Invalid UID".to_string()))?;

        let flag_cmd = if is_read {
            "+FLAGS (\\Seen)"
        } else {
            "-FLAGS (\\Seen)"
        };

        let _ = session.uid_store(uid.to_string(), flag_cmd).await?;

        Ok(())
    }

    async fn set_flag(
        &self,
        email_remote_id: &str,
        folder: &SyncFolder,
        flagged: bool,
    ) -> SyncResult<()> {
        let mut session_guard = self.get_session().await?;
        let session = session_guard
            .as_mut()
            .ok_or_else(|| SyncError::ImapError("No active session".to_string()))?;

        session.select(&folder.remote_id).await?;

        let uid: u32 = email_remote_id
            .parse()
            .map_err(|_| SyncError::ParseError("Invalid UID".to_string()))?;

        let flag_cmd = if flagged {
            "+FLAGS (\\Flagged)"
        } else {
            "-FLAGS (\\Flagged)"
        };

        let _ = session.uid_store(uid.to_string(), flag_cmd).await?;

        Ok(())
    }

    async fn rename_folder(&self, folder: &SyncFolder, new_name: &str) -> SyncResult<()> {
        let mut session_guard = self.get_session().await?;
        let session = session_guard
            .as_mut()
            .ok_or_else(|| SyncError::ImapError("No active session".to_string()))?;

        // IMAP RENAME command: RENAME old_name new_name
        // Build the full path for the new name (keeping the same parent)
        let old_path = &folder.remote_id;
        let new_path = if let Some(parent_sep_pos) = old_path.rfind('/') {
            format!("{}/{}", &old_path[..parent_sep_pos], new_name)
        } else {
            new_name.to_string()
        };

        session.rename(old_path, &new_path).await?;

        log::info!("Renamed IMAP folder from '{}' to '{}'", old_path, new_path);
        Ok(())
    }

    async fn move_folder(
        &self,
        folder: &SyncFolder,
        new_parent_path: Option<&str>,
    ) -> SyncResult<()> {
        let mut session_guard = self.get_session().await?;
        let session = session_guard
            .as_mut()
            .ok_or_else(|| SyncError::ImapError("No active session".to_string()))?;

        let old_path = &folder.remote_id;

        // Extract folder name from current path
        let folder_name = old_path.rsplit('/').next().unwrap_or(old_path);

        // Build new path based on parent
        let new_path = if let Some(parent) = new_parent_path {
            format!("{}/{}", parent, folder_name)
        } else {
            // Moving to root
            folder_name.to_string()
        };

        // IMAP RENAME command works for moving folders too
        session.rename(old_path, &new_path).await?;

        log::info!("Moved IMAP folder from '{}' to '{}'", old_path, new_path);
        Ok(())
    }

    async fn get_sync_token(&self) -> SyncResult<Option<String>> {
        // IMAP doesn't have sync tokens, use UID instead
        Ok(None)
    }

    async fn sync_since_token(&self, _token: &str) -> SyncResult<Vec<SyncEmail>> {
        Err(SyncError::NotSupported(
            "IMAP doesn't support token-based sync".to_string(),
        ))
    }
}

fn decode_modified_utf7(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '&' {
            // Special case: "&-" encodes literal '&'
            if let Some(&next) = chars.peek() {
                if next == '-' {
                    // consume '-'
                    chars.next();
                    out.push('&');
                    continue;
                }
            }

            // Collect base64 chunk until '-'
            let mut b64 = String::new();
            while let Some(c) = chars.next() {
                if c == '-' {
                    break;
                }
                b64.push(c);
            }

            // Convert modified base64 (',' -> '/') and decode.
            // Ensure padding so base64 decoder behaves consistently.
            let mut b64_mod = b64.replace(',', "/");
            let rem = b64_mod.len() % 4;
            if rem != 0 {
                b64_mod.push_str(&"=".repeat(4 - rem));
            }
            if b64_mod.is_empty() {
                // Nothing to decode
                continue;
            }

            match STANDARD.decode(&b64_mod) {
                Ok(bytes) => {
                    if bytes.len() % 2 != 0 {
                        // malformed - append replacement for safety
                        out.push_str(&String::from_utf8_lossy(&bytes));
                        continue;
                    }

                    // Interpret as UTF-16BE
                    let mut u16_buf = Vec::with_capacity(bytes.len() / 2);
                    let mut i = 0;
                    while i + 1 < bytes.len() {
                        let hi = (bytes[i] as u16) << 8;
                        let lo = bytes[i + 1] as u16;
                        u16_buf.push(hi | lo);
                        i += 2;
                    }

                    match String::from_utf16(&u16_buf) {
                        Ok(s) => out.push_str(&s),
                        Err(_) => out.push_str(&String::from_utf8_lossy(&bytes)),
                    }
                }
                Err(_) => {
                    // on decode error, skip and continue
                }
            }
        } else {
            out.push(ch);
        }
    }

    out
}
