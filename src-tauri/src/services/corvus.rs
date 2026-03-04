use crate::config::Settings;
use crate::database::models::account::Account;
use crate::database::models::email::Email;
use crate::licensing::LicenseManager;
use openrouter_rs::api::chat::{
    ChatCompletionRequest as ChatRequest, Message as OpenRouterChatMessage,
};
use openrouter_rs::client::OpenRouterClient;
use openrouter_rs::types::{ProviderPreferences, ProviderSortBy, Role};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use turndown::Turndown;

const MAX_PRIOR_EMAIL_TOKENS: usize = 500;
const MAX_CURRENT_TEXT_TOKENS: usize = 300;
const MAX_OTHER_MAILS_TOKENS: usize = 800;
const APPROX_CHARS_PER_TOKEN: usize = 4;

pub struct CorvusService {
    settings: Arc<Settings>,
    license_manager: Arc<LicenseManager>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailAnalysisResponse {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmailAnalysis {
    pub gist: String,
    pub responses: Vec<EmailAnalysisResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelPricing {
    pub prompt: f32,
    pub completion: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvailableModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub context_length: f64,
    pub pricing: ModelPricing,
}

#[derive(Debug, Clone)]
pub struct AskAiRequest {
    pub history: Vec<ChatMessage>,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct EmailCompletionRequest {
    pub metadata: EmailMetadata,
    pub prior_email: Option<String>,
    pub current_text: String,
    pub cursor_position: usize,
    /// AI notes for the primary contacts involved in this email (keyed by email address)
    pub contact_notes: Vec<ContactNote>,
}

#[derive(Debug, Clone)]
pub struct ContactNote {
    pub email: String,
    pub display_name: Option<String>,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct EmailMetadata {
    pub sender: String,
    pub subject: String,
    pub is_reply: bool,
    pub recipients: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GenerateSubjectRequest {
    pub body_content: String,
    pub sender: String,
    pub recipients: Vec<String>,
    pub is_reply: bool,
    pub current_subject: Option<String>,
    /// AI notes for the primary contacts involved in this email
    pub contact_notes: Vec<ContactNote>,
}

#[derive(Debug, Clone)]
pub struct GenerateSearchQueryRequest {
    pub natural_language_query: String,
}

/// Represents the current user / account performing the analysis.
#[derive(Debug, Clone)]
pub struct UserContext {
    pub name: String,
    pub email: String,
}

impl UserContext {
    pub fn from_account(account: &Account) -> Self {
        Self {
            name: account.name.clone(),
            email: account.email.clone(),
        }
    }
}

/// Describes the user's relationship to the email being analysed.
#[derive(Debug, Clone, PartialEq)]
enum UserEmailRole {
    Sender,
    PrimaryRecipient,
    CcRecipient,
    BccRecipient,
    Unknown,
}

impl UserEmailRole {
    fn detect(user_email: &str, email: &Email) -> Self {
        let user_lower = user_email.to_lowercase();

        // Check if user is the sender
        if email.from().address.to_lowercase() == user_lower {
            return UserEmailRole::Sender;
        }

        // Check To recipients
        if email
            .to()
            .iter()
            .any(|a| a.address.to_lowercase() == user_lower)
        {
            return UserEmailRole::PrimaryRecipient;
        }

        // Check CC recipients
        if email
            .cc()
            .iter()
            .any(|a| a.address.to_lowercase() == user_lower)
        {
            return UserEmailRole::CcRecipient;
        }

        // Check BCC recipients
        if email
            .bcc()
            .iter()
            .any(|a| a.address.to_lowercase() == user_lower)
        {
            return UserEmailRole::BccRecipient;
        }

        UserEmailRole::Unknown
    }

    fn as_str(&self) -> &'static str {
        match self {
            UserEmailRole::Sender => "sender",
            UserEmailRole::PrimaryRecipient => "primary recipient (in To)",
            UserEmailRole::CcRecipient => "CC'd recipient (informational copy)",
            UserEmailRole::BccRecipient => "BCC'd recipient (blind copy)",
            UserEmailRole::Unknown => "indirect participant",
        }
    }

    fn action_guidance(&self) -> &'static str {
        match self {
            UserEmailRole::Sender => {
                "You sent this email. Suggest follow-up actions, reminders, or clarifications \
                 you might want to send. Do NOT suggest responses as if you are receiving it."
            }
            UserEmailRole::PrimaryRecipient => {
                "You are a primary recipient. This email is directly addressed to you and likely \
                 requires your attention or a direct reply."
            }
            UserEmailRole::CcRecipient => {
                "You are CC'd for information only. This email may not require action from you \
                 unless you choose to contribute. Keep suggested responses brief and optional."
            }
            UserEmailRole::BccRecipient => {
                "You received a blind copy of this email. You are not expected to reply; \
                 only suggest an action if there is a clear reason to act independently."
            }
            UserEmailRole::Unknown => {
                "Your exact role in this email thread is unclear. Provide balanced, \
                 context-neutral response options."
            }
        }
    }
}

impl CorvusService {
    pub fn new(settings: Arc<Settings>, license_manager: Arc<LicenseManager>) -> Self {
        Self {
            settings,
            license_manager,
        }
    }

    async fn get_api_key(&self) -> Result<String, String> {
        // First check if user has configured their own API key
        if let Ok(user_key) = self.settings.get::<String>("ai.api.key") {
            if !user_key.is_empty() {
                log::debug!("Using user-configured API key");
                return Ok(user_key);
            }
        }

        // If no user key, check if license provides one (SaaS mode)
        if let Some(license_token) = self.license_manager.get_ai_token().await {
            log::debug!("Using license-provided API key (SaaS mode)");
            return Ok(license_token);
        }

        Err(
            "API key not configured. Please set ai.api.key in settings or activate a SaaS license."
                .to_string(),
        )
    }

    pub async fn is_enabled(&self) -> bool {
        let user_api_key = self.settings.get::<String>("ai.api.key").ok();
        self.license_manager.should_enable_ai(user_api_key).await
    }

    pub async fn get_ai_limits(&self) -> (f64, f64) {
        self.license_manager.get_ai_limits().await
    }

    fn get_base_url(&self) -> Result<String, String> {
        self.settings
            .get::<String>("ai.api.baseUrl")
            .or_else(|_| Ok("https://openrouter.ai/api/v1".to_string()))
    }

    fn get_model(&self, model_type: &str) -> Result<String, String> {
        let model_path = format!("ai.models.{}", model_type);
        self.settings
            .get::<String>(&model_path)
            .map_err(|e| format!("Failed to get {} model from settings: {}", model_type, e))
    }

    fn get_prompt(&self, prompt_type: &str) -> Result<String, String> {
        let prompt_path = format!("ai.prompts.{}", prompt_type);
        self.settings
            .get::<String>(&prompt_path)
            .map_err(|e| format!("Failed to get {} prompt from settings: {}", prompt_type, e))
    }

    fn get_sorting_preference(&self) -> Result<ProviderSortBy, String> {
        self.settings
            .get::<String>("ai.models.sorting")
            .map(|sort_str| match sort_str.as_str() {
                "price" => ProviderSortBy::Price,
                "latency" => ProviderSortBy::Latency,
                "throughput" => ProviderSortBy::Throughput,
                _ => ProviderSortBy::Throughput,
            })
            .map_err(|e| format!("Failed to get model sorting preference: {}", e))
    }

    fn get_writing_style(&self) -> Option<String> {
        self.settings
            .get::<Option<String>>("ai.writingStyle")
            .ok()
            .flatten()
    }

    fn build_writing_style_context(&self) -> String {
        match self.get_writing_style() {
            Some(style) => {
                format!(
                    "\n\nFollow these personal writing guides strictly:\n{}",
                    style
                )
            }
            None => String::new(),
        }
    }

    fn build_contact_notes_context(contact_notes: &[ContactNote]) -> String {
        if contact_notes.is_empty() {
            return String::new();
        }

        let notes_text = contact_notes
            .iter()
            .map(|cn| {
                let label = match &cn.display_name {
                    Some(name) if !name.is_empty() => format!("{} <{}>", name, cn.email),
                    _ => cn.email.clone(),
                };
                format!("- {}: {}", label, cn.notes)
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            "\n\n## Personal Notes about Contacts\nUse the following notes about the people involved to personalise your response:\n{}",
            notes_text
        )
    }

    async fn get_client(&self) -> Result<OpenRouterClient, String> {
        let api_key = self.get_api_key().await?;
        let base_url = self.get_base_url()?;

        Ok(OpenRouterClient::builder()
            .api_key(api_key)
            .base_url(&base_url)
            .http_referer("https://ravnmail.com")
            .x_title("RAVN Mail")
            .build()
            .unwrap())
    }

    fn get_provider_preferences(&self) -> Result<ProviderPreferences, String> {
        let sorting_preference = self.get_sorting_preference()?;
        Ok(ProviderPreferences {
            allow_fallbacks: None,
            require_parameters: None,
            data_collection: None,
            order: None,
            ignore: None,
            quantizations: None,
            sort: Some(sorting_preference),
        })
    }

    pub async fn ask_ai(&self, request: AskAiRequest) -> Result<String, String> {
        if !self.is_enabled().await {
            return Err(
                "AI service is not enabled. Please configure an API key or activate a license."
                    .to_string(),
            );
        }

        log::debug!(
            "Processing ask_ai request with {} messages",
            request.history.len()
        );

        let client = self.get_client().await?;
        let model = self.get_model("normal")?;
        let mut system_prompt = self.get_prompt("askAi")?;
        system_prompt.push_str(&self.build_writing_style_context());

        let messages: Vec<OpenRouterChatMessage> = request
            .history
            .into_iter()
            .map(|msg| {
                let role = match msg.role.as_str() {
                    "system" => Role::System,
                    "assistant" => Role::Assistant,
                    _ => Role::User,
                };
                OpenRouterChatMessage::new(role, &*msg.content)
            })
            .collect();

        let chat_request = ChatRequest::builder()
            .model(model.clone())
            .messages(messages)
            .provider(self.get_provider_preferences()?)
            .build()
            .map_err(|e| format!("Failed to build chat request: {}", e))?;

        let response = client
            .send_chat_completion(&chat_request)
            .await
            .map_err(|e| format!("OpenRouter API request failed: {}", e))?;

        Ok(response.choices[0].content().unwrap().to_string())
    }

    pub async fn generate_email_completion(
        &self,
        request: EmailCompletionRequest,
    ) -> Result<String, String> {
        if !self.is_enabled().await {
            return Err(
                "AI service is not enabled. Please configure an API key or activate a license."
                    .to_string(),
            );
        }

        log::debug!("Processing email completion request");

        let client = self.get_client().await?;
        let model = self.get_model("fast")?;

        let user_message = self.build_autocomplete_prompt(&request);
        let mut system_prompt = self.get_prompt("generateCompletion")?;
        system_prompt.push_str(&self.build_writing_style_context());
        system_prompt.push_str(&Self::build_contact_notes_context(&request.contact_notes));

        let messages = vec![
            OpenRouterChatMessage::new(Role::System, &*system_prompt),
            OpenRouterChatMessage::new(Role::User, &*user_message),
        ];

        let chat_request = ChatRequest::builder()
            .model(model.clone())
            .messages(messages)
            .provider(self.get_provider_preferences()?)
            .build()
            .map_err(|e| format!("Failed to build chat request: {}", e))?;

        let response = client
            .send_chat_completion(&chat_request)
            .await
            .map_err(|e| format!("OpenRouter API request failed: {}", e))?;

        Ok(response.choices[0].content().unwrap().to_string())
    }

    pub async fn generate_subject(
        &self,
        request: GenerateSubjectRequest,
    ) -> Result<String, String> {
        if !self.is_enabled().await {
            return Err(
                "AI service is not enabled. Please configure an API key or activate a license."
                    .to_string(),
            );
        }

        log::debug!("Processing generate subject request");

        let client = self.get_client().await?;
        let model = self.get_model("normal")?;
        let mut system_prompt = self.get_prompt("generateSubject")?;
        system_prompt.push_str(&self.build_writing_style_context());
        system_prompt.push_str(&Self::build_contact_notes_context(&request.contact_notes));

        let prompt = format!(
            "Generate a concise, professional email subject line based on the following email content. The subject should be clear, specific, and engaging. Email content: {}\n\nCurrent subject (if any): {}\n\nGenerate only the subject line, nothing else.",
            request.body_content,
            request.current_subject.unwrap_or_else(|| "None".to_string())
        );

        let messages = vec![OpenRouterChatMessage::new(Role::User, &*prompt)];

        let chat_request = ChatRequest::builder()
            .model(model.clone())
            .messages(messages)
            .provider(self.get_provider_preferences()?)
            .build()
            .map_err(|e| format!("Failed to build chat request: {}", e))?;

        let response = client
            .send_chat_completion(&chat_request)
            .await
            .map_err(|e| format!("OpenRouter API request failed: {}", e))?;

        Ok(response.choices[0].content().unwrap().to_string())
    }

    pub async fn analyze_email(
        &self,
        email: &Email,
        user_context: Option<&UserContext>,
        contact_notes: &[ContactNote],
    ) -> Result<EmailAnalysis, String> {
        if !self.is_enabled().await {
            return Err(
                "AI service is not enabled. Please configure an API key or activate a license."
                    .to_string(),
            );
        }

        log::debug!("Processing email analysis request for email {}", email.id);

        let client = self.get_client().await?;
        let model = self.get_model("normal")?;
        let system_prompt = self.get_prompt("analyzeEmail")?;
        let writing_style = self.get_writing_style().unwrap_or_default();

        // Helper closure to format an email address as "Name <address>" or just "address"
        let fmt_addr = |name: &Option<String>, address: &str| -> String {
            match name.as_deref().filter(|n| !n.is_empty()) {
                Some(n) => format!("{} <{}>", n, address),
                None => address.to_owned(),
            }
        };

        // Extract email fields
        let from = fmt_addr(&email.from().name, &email.from().address);

        let to = email
            .to()
            .iter()
            .map(|addr| fmt_addr(&addr.name, &addr.address))
            .collect::<Vec<_>>()
            .join(", ");

        let cc = email
            .cc()
            .iter()
            .map(|addr| fmt_addr(&addr.name, &addr.address))
            .collect::<Vec<_>>()
            .join(", ");

        let bcc = email
            .bcc()
            .iter()
            .map(|addr| fmt_addr(&addr.name, &addr.address))
            .collect::<Vec<_>>()
            .join(", ");

        let subject = email
            .subject
            .clone()
            .unwrap_or_else(|| "(No subject)".to_string());
        let content = email
            .body_plain
            .clone()
            .or_else(|| email.body_html.clone())
            .unwrap_or_default();

        // Convert other_mails (quoted/forwarded HTML thread) to plain text and token-cap it
        let thread_context_section = match &email.other_mails {
            Some(html) if !html.is_empty() => {
                let turndown = Turndown::default();
                let plain = turndown.convert(html);
                let max_chars = MAX_OTHER_MAILS_TOKENS * APPROX_CHARS_PER_TOKEN;
                let truncated = if plain.len() > max_chars {
                    log::debug!(
                        "Email {} thread context truncated from {} to {} chars",
                        email.id,
                        plain.len(),
                        max_chars
                    );
                    let half = max_chars / 2;
                    format!(
                        "{}\n\n[... thread truncated ...]\n\n{}",
                        plain[..half].trim_end(),
                        plain[plain.len() - half..].trim_start()
                    )
                } else {
                    plain
                };
                format!("\n\n## Prior Thread / Quoted Content\n```{}```", truncated)
            }
            _ => String::new(),
        };

        let current_datetime = chrono::Utc::now().to_rfc3339();
        let received_at = email.received_at.to_rfc3339();

        // Build user-context section for the prompt
        let user_context_section = match user_context {
            Some(ctx) => {
                let role = UserEmailRole::detect(&ctx.email, email);
                log::debug!(
                    "Email {} analysis: user '{}' detected as '{}'",
                    email.id,
                    ctx.email,
                    role.as_str()
                );
                format!(
                    "\n## Current User\nName: {}\nEmail: {}\nRole in this email: {}\n\nAction guidance: {}",
                    ctx.name,
                    ctx.email,
                    role.as_str(),
                    role.action_guidance()
                )
            }
            None => String::new(),
        };

        // Flag special email states
        let email_flags = {
            let mut flags = Vec::new();
            if email.is_draft {
                flags.push("draft");
            }
            if email.has_attachments {
                flags.push("has attachments");
            }
            if email.is_flagged {
                flags.push("starred/flagged");
            }
            if flags.is_empty() {
                String::new()
            } else {
                format!("\nFlags: {}", flags.join(", "))
            }
        };

        let bcc_line = if bcc.is_empty() {
            String::new()
        } else {
            format!("\nBcc: {}", bcc)
        };

        let user_prompt = format!(
            r#"Current DateTime: {}
{}
## Email Details
From: {}
To: {}
Cc: {}{}{}
Subject: {}
Received At: {}

## Email Content
```{}```
{}
"#,
            current_datetime,
            user_context_section,
            from,
            to,
            cc,
            bcc_line,
            email_flags,
            subject,
            received_at,
            content,
            thread_context_section,
        );

        let system_with_style = if writing_style.is_empty() {
            system_prompt
        } else {
            format!(
                "{}\n\n## Personal Writing Style\n{}",
                system_prompt, writing_style
            )
        };

        let system_with_style = if contact_notes.is_empty() {
            system_with_style
        } else {
            format!(
                "{}{}",
                system_with_style,
                Self::build_contact_notes_context(contact_notes)
            )
        };

        log::debug!(
            "Sending analyze_email request to OpenRouter: model='{}', email_id='{}', subject='{}'",
            model,
            email.id,
            subject
        );
        log::info!(
            "analyze_email system prompt ({} chars): {}",
            system_with_style.len(),
            system_with_style
        );
        log::info!(
            "analyze_email user prompt ({} chars): {}",
            user_prompt.len(),
            user_prompt
        );

        let messages = vec![
            OpenRouterChatMessage::new(Role::System, &*system_with_style),
            OpenRouterChatMessage::new(Role::User, &*user_prompt),
        ];

        let chat_request = ChatRequest::builder()
            .model(model.clone())
            .messages(messages)
            .provider(self.get_provider_preferences()?)
            .build()
            .map_err(|e| format!("Failed to build chat request: {}", e))?;

        let response = client
            .send_chat_completion(&chat_request)
            .await
            .map_err(|e| format!("OpenRouter API request failed: {}", e))?;

        let response_text = response.choices[0].content().unwrap().to_string();

        log::debug!(
            "analyze_email received response from OpenRouter ({} chars) for email '{}'",
            response_text.len(),
            email.id
        );
        log::trace!("analyze_email raw response: {}", response_text);

        // Strip a possible markdown code fence that some models add around JSON
        let json_str = response_text
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        serde_json::from_str::<EmailAnalysis>(json_str).map_err(|e| {
            format!(
                "Failed to parse analysis JSON: {}. Content: {}",
                e, response_text
            )
        })
    }

    pub async fn generate_search_query(
        &self,
        request: GenerateSearchQueryRequest,
    ) -> Result<String, String> {
        if !self.is_enabled().await {
            return Err(
                "AI service is not enabled. Please configure an API key or activate a license."
                    .to_string(),
            );
        }

        log::debug!("Processing search query generation request");

        let client = self.get_client().await?;
        let model = self.get_model("fast")?;
        let system_prompt = self.get_prompt("generateSearchQuery")?;

        let prompt = format!(
            "Convert this natural language query into a Tantivy search query:\n\n{}\n\nCurrent DateTime: {}",
            request.natural_language_query,
            chrono::Utc::now().to_rfc3339()
        );

        let messages = vec![
            OpenRouterChatMessage::new(Role::System, &*system_prompt),
            OpenRouterChatMessage::new(Role::User, &*prompt),
        ];

        let chat_request = ChatRequest::builder()
            .model(model.clone())
            .messages(messages)
            .build()
            .map_err(|e| format!("Failed to build chat request: {}", e))?;

        let response = client
            .send_chat_completion(&chat_request)
            .await
            .map_err(|e| format!("OpenRouter API request failed: {}", e))?;

        Ok(response.choices[0].content().unwrap().to_string())
    }

    pub async fn get_available_models(&self) -> Result<Vec<AvailableModel>, String> {
        if !self.is_enabled().await {
            return Err(
                "AI service is not enabled. Please configure an API key or activate a license."
                    .to_string(),
            );
        }

        log::debug!("Fetching available models");

        let client = self.get_client().await?;

        let mut result = client
            .list_models()
            .await
            .map_err(|e| format!("Failed to fetch models: {}", e))?;

        result.sort_by(|a, b| a.name.cmp(&b.name));

        result
            .into_iter()
            .map(|model| {
                Ok(AvailableModel {
                    id: model.id,
                    name: model.name,
                    description: model.description,
                    context_length: model.context_length,
                    pricing: ModelPricing {
                        prompt: model.pricing.prompt.parse().unwrap_or(0.0),
                        completion: model.pricing.completion.parse().unwrap_or(0.0),
                    },
                })
            })
            .collect()
    }

    fn build_autocomplete_prompt(&self, context: &EmailCompletionRequest) -> String {
        let mut message = String::new();

        message.push_str(&format!(
            "Current Datetime: {}\nSender: {}\nSubject: {}\nIs Reply: {}\nRecipients: {}\n\n",
            chrono::Utc::now().to_rfc3339(),
            context.metadata.sender,
            context.metadata.subject,
            context.metadata.is_reply,
            context.metadata.recipients.join(", ")
        ));

        if let Some(prior_content) = &context.prior_email {
            message.push_str("Reply to this email:\n");

            let max_prior_chars = MAX_PRIOR_EMAIL_TOKENS * APPROX_CHARS_PER_TOKEN;
            if prior_content.len() > max_prior_chars {
                let start_chars = max_prior_chars / 2;
                let end_chars = max_prior_chars / 2;

                let start_text = &prior_content[..start_chars];
                let end_text = &prior_content[prior_content.len() - end_chars..];

                message.push_str(start_text);
                message.push_str("\n[...]\n");
                message.push_str(end_text);
            } else {
                message.push_str(&prior_content);
            }

            message.push_str("\n\n");
        }

        message.push_str("Current email (up to cursor position):\n");

        let cursor_pos = context.cursor_position.min(context.current_text.len());
        let current_text = &context.current_text[..cursor_pos];

        let max_current_chars = MAX_CURRENT_TEXT_TOKENS * APPROX_CHARS_PER_TOKEN;
        if current_text.len() > max_current_chars {
            let start_pos = cursor_pos.saturating_sub(max_current_chars);
            message.push_str("[...]\n");
            message.push_str(&current_text[start_pos..]);
        } else {
            message.push_str(current_text);
        }

        message
    }
}
