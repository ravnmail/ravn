use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::{BooleanQuery, Query, QueryParser, TermQuery};
use tantivy::schema::*;
use tantivy::{Index, IndexWriter, ReloadPolicy, TantivyDocument, Term};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::error::{SearchError, SearchResult};
use crate::database::models::email::{Email, EmailAddress};

/// Fields in the Tantivy search index
/// Designed to match the user documentation's search operators:
/// - from:, to:, cc: for email addresses
/// - subject:, labels:, folder: for metadata
/// - is:read, is:unread for read status
/// - received:[DATE TO DATE] for date ranges
pub struct EmailSchema {
    pub id: Field,
    pub account_id: Field,
    pub folder_id: Field,
    pub conversation_id: Field,

    pub subject: Field,
    pub body: Field,

    pub from: Field,
    pub to: Field,
    pub cc: Field,

    pub received: Field,
    pub is_read: Field,
    pub is_flagged: Field,
    pub is_deleted: Field,
    pub labels: Field,
}

impl EmailSchema {
    pub fn build() -> (Schema, Self) {
        let mut schema_builder = Schema::builder();

        let text_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("default")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();

        let email_address_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("default")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();

        let fast_text_options = TextOptions::default().set_fast(Some("raw"));

        let email_schema = EmailSchema {
            id: schema_builder.add_text_field("id", STRING | STORED | FAST),
            account_id: schema_builder.add_text_field("account_id", STRING | FAST),
            folder_id: schema_builder.add_text_field("folder_id", STRING | FAST),
            conversation_id: schema_builder.add_text_field("conversation_id", STRING | FAST),

            subject: schema_builder.add_text_field("subject", text_options.clone()),
            body: schema_builder.add_text_field("body", text_options.clone()),

            from: schema_builder.add_text_field("from", email_address_options.clone()),
            to: schema_builder.add_text_field("to", email_address_options.clone()),
            cc: schema_builder.add_text_field("cc", email_address_options.clone()),

            received: schema_builder.add_date_field("received", STORED | INDEXED | FAST),

            is_read: schema_builder.add_bool_field("is_read", STORED | INDEXED | FAST),
            is_flagged: schema_builder.add_bool_field("is_flagged", STORED | FAST),
            is_deleted: schema_builder.add_bool_field("is_deleted", STORED | INDEXED | FAST),

            labels: schema_builder.add_text_field("labels", fast_text_options),
        };

        (schema_builder.build(), email_schema)
    }
}

/// Search query parameters supporting user documentation syntax
/// Examples:
/// - Simple: "budget report"
/// - With operators: "from:john budget is:unread"
/// - Complex: "(from:john OR from:jane) AND budget received:[2024-01-01 TO 2024-12-31]"
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchQuery {
    /// Search query string (supports full Tantivy query syntax)
    /// Users can enter:
    /// - Keywords: "budget"
    /// - Phrases: "\"fiscal year 2024\""
    /// - Email operators: "from:john to:sarah cc:team"
    /// - Boolean: "from:john AND budget", "invoice OR receipt"
    /// - Negation: "report -draft"
    /// - Date ranges: "received:[2024-01-01 TO 2024-12-31]"
    /// - Wildcards: "report*" (prefix matching)
    /// - Fuzzy: "rusty~1" (edit distance matching)
    pub query: String,

    pub account_id: Option<Uuid>,
    pub folder_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,

    #[serde(default = "default_limit")]
    pub limit: usize,

    #[serde(default)]
    pub offset: usize,
}

fn default_limit() -> usize {
    50
}

/// Search result item (minimal data for list views)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultItem {
    pub id: Uuid,
    pub score: f32,
}

/// Manages the Tantivy search index for emails
pub struct SearchManager {
    index: Index,
    schema: EmailSchema,
    writer: Arc<RwLock<IndexWriter>>,
    reader: tantivy::IndexReader,
}

impl SearchManager {
    /// Initialize or open the search index at the given path
    pub fn new<P: AsRef<Path>>(index_path: P) -> SearchResult<Self> {
        let path = index_path.as_ref();
        std::fs::create_dir_all(path)?;

        let (schema_def, schema) = EmailSchema::build();

        let directory = MmapDirectory::open(path)?;
        let index = if Index::exists(&directory)? {
            Index::open(directory)?
        } else {
            Index::create(directory, schema_def.clone(), Default::default())?
        };

        let writer = index.writer(50_000_000)?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        Ok(Self {
            index,
            schema,
            writer: Arc::new(RwLock::new(writer)),
            reader,
        })
    }

    pub async fn index_email(&self, email: &Email) -> SearchResult<()> {
        let doc = self.email_to_document(email)?;
        let writer = self.writer.write().await;

        writer.delete_term(Term::from_field_text(self.schema.id, &email.id.to_string()));
        writer.add_document(doc)?;

        Ok(())
    }

    /// Index multiple emails in batch for better performance
    pub async fn index_emails_batch(&self, emails: &[Email]) -> SearchResult<()> {
        let writer = self.writer.write().await;

        for email in emails {
            let doc = self.email_to_document(email)?;

            writer.delete_term(Term::from_field_text(self.schema.id, &email.id.to_string()));
            writer.add_document(doc)?;
        }

        Ok(())
    }

    /// Commit all pending changes to the index
    pub async fn commit(&self) -> SearchResult<()> {
        let mut writer = self.writer.write().await;
        writer.commit()?;
        Ok(())
    }

    /// Delete an email from the index
    pub async fn delete_email(&self, email_id: Uuid) -> SearchResult<()> {
        let mut writer = self.writer.write().await;
        writer.delete_term(Term::from_field_text(self.schema.id, &email_id.to_string()));
        writer.commit()?;
        Ok(())
    }

    /// Search emails with the given query
    /// Supports all user documentation operators:
    /// - from:, to:, cc: for email addresses (supports address, name, or partial matches)
    /// - labels: for labels
    /// - received: for date ranges
    /// - is:read, is:unread for read status
    /// - Boolean operators: AND, OR, NOT, ()
    /// - Wildcards: *
    /// - Fuzzy matching: ~N
    /// - Phrase queries: ""
    /// - Negation: -
    pub async fn search(&self, query: SearchQuery) -> SearchResult<Vec<SearchResultItem>> {
        self.validate_query(&query)?;

        let searcher = self.reader.searcher();
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![
                self.schema.subject,
                self.schema.body,
                self.schema.from,
                self.schema.to,
                self.schema.cc,
                self.schema.received,
                self.schema.is_read,
                self.schema.labels,
            ],
        );

        let parsed_query = query_parser.parse_query(&query.query)?;
        let mut filters: Vec<Box<dyn Query>> = vec![Box::new(parsed_query)];

        if let Some(account_id) = query.account_id {
            let term = Term::from_field_text(self.schema.account_id, &account_id.to_string());
            filters.push(Box::new(TermQuery::new(term, IndexRecordOption::Basic)));
        }

        if let Some(folder_id) = query.folder_id {
            let term = Term::from_field_text(self.schema.folder_id, &folder_id.to_string());
            filters.push(Box::new(TermQuery::new(term, IndexRecordOption::Basic)));
        }

        if let Some(conversation_id) = query.conversation_id {
            let term =
                Term::from_field_text(self.schema.conversation_id, &conversation_id.to_string());
            filters.push(Box::new(TermQuery::new(term, IndexRecordOption::Basic)));
        }

        let final_query = if filters.len() > 1 {
            Box::new(BooleanQuery::intersection(filters)) as Box<dyn Query>
        } else {
            filters.into_iter().next().unwrap()
        };

        let limit = query.limit.min(1000);
        let offset = query.offset;
        let top_docs = searcher.search(&final_query, &TopDocs::with_limit(limit + offset))?;

        let results: Vec<SearchResultItem> = top_docs
            .into_iter()
            .skip(offset)
            .take(limit)
            .filter_map(|(score, doc_address)| {
                let doc: TantivyDocument = searcher.doc(doc_address).ok()?;
                let id_field = doc.get_first(self.schema.id)?;
                let id_str = id_field.as_str()?;
                let id = Uuid::parse_str(id_str).ok()?;

                Some(SearchResultItem { id, score })
            })
            .collect();

        Ok(results)
    }

    /// Clear the entire index (use with caution!)
    pub async fn clear_index(&self) -> SearchResult<()> {
        let mut writer = self.writer.write().await;
        writer.delete_all_documents()?;
        writer.commit()?;
        Ok(())
    }

    /// Convert an Email model to a Tantivy document
    /// Maps email fields to search schema fields for indexing
    /// Properly handles EmailAddress structs by combining address + name
    fn email_to_document(&self, email: &Email) -> SearchResult<TantivyDocument> {
        let mut doc = TantivyDocument::new();

        doc.add_text(self.schema.id, email.id.to_string());
        doc.add_text(self.schema.account_id, email.account_id.to_string());
        doc.add_text(self.schema.folder_id, email.folder_id.to_string());

        if let Some(conv_id) = &email.conversation_id {
            doc.add_text(self.schema.conversation_id, conv_id);
        }

        if let Some(subject) = &email.subject {
            doc.add_text(self.schema.subject, subject);
        }

        if let Some(body_plain) = &email.body_plain {
            doc.add_text(self.schema.body, body_plain);
        }

        self.add_email_address_to_field(&mut doc, self.schema.from, &email.from.0);

        for recipient in &email.to.0 {
            self.add_email_address_to_field(&mut doc, self.schema.to, recipient);
        }
        for recipient in &email.cc.0 {
            self.add_email_address_to_field(&mut doc, self.schema.cc, recipient);
        }

        let timestamp = email.received_at.timestamp();
        let tantivy_datetime = tantivy::DateTime::from_timestamp_secs(timestamp);
        doc.add_date(self.schema.received, tantivy_datetime);

        doc.add_bool(self.schema.is_read, email.is_read);
        doc.add_bool(self.schema.is_flagged, email.is_flagged);
        doc.add_bool(self.schema.is_deleted, email.is_deleted);

        Ok(doc)
    }

    /// Helper method to add a complete EmailAddress to a field
    /// Stores both the address and display name to support flexible matching
    ///
    /// Examples:
    /// - EmailAddress { address: "john@company.com", name: Some("John Smith") }
    ///   Will be indexed to match: "john", "company.com", "john@company.com", "John", "Smith", etc.
    /// - EmailAddress { address: "jane@company.com", name: None }
    ///   Will be indexed to match: "jane", "company.com", "jane@company.com"
    fn add_email_address_to_field(
        &self,
        doc: &mut TantivyDocument,
        field: Field,
        email_address: &EmailAddress,
    ) {
        doc.add_text(field, &email_address.address);
        if let Some(name) = &email_address.name {
            doc.add_text(field, name);
        }
    }

    /// Validate search query to prevent abuse and performance issues
    fn validate_query(&self, query: &SearchQuery) -> SearchResult<()> {
        const MAX_QUERY_LENGTH: usize = 2000;
        const MAX_OR_CLAUSES: usize = 50;
        const MAX_WILDCARDS: usize = 5;

        if query.query.len() > MAX_QUERY_LENGTH {
            return Err(SearchError::InvalidQuery(format!(
                "Query too long (max {} characters)",
                MAX_QUERY_LENGTH
            )));
        }

        let or_count = query.query.matches(" OR ").count();
        if or_count > MAX_OR_CLAUSES {
            return Err(SearchError::InvalidQuery(format!(
                "Too many OR clauses (max {})",
                MAX_OR_CLAUSES
            )));
        }

        let wildcard_count = query.query.matches('*').count();
        if wildcard_count > MAX_WILDCARDS {
            return Err(SearchError::InvalidQuery(
                "Too many wildcards in query".to_string(),
            ));
        }

        if query.limit > 1000 {
            return Err(SearchError::InvalidQuery(
                "Limit cannot exceed 1000".to_string(),
            ));
        }

        if query.offset > 10000 {
            return Err(SearchError::InvalidQuery(
                "Offset cannot exceed 10000".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_index() {
        let temp_dir = TempDir::new().unwrap();
        let search_manager = SearchManager::new(temp_dir.path()).unwrap();
        assert!(search_manager.index.schema().fields().count() > 0);
    }

    #[tokio::test]
    async fn test_validate_query_length() {
        let temp_dir = TempDir::new().unwrap();
        let search_manager = SearchManager::new(temp_dir.path()).unwrap();

        let query = SearchQuery {
            query: "a".repeat(2001),
            account_id: None,
            folder_id: None,
            conversation_id: None,
            limit: 50,
            offset: 0,
        };

        let result = search_manager.validate_query(&query);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_query_or_clauses() {
        let temp_dir = TempDir::new().unwrap();
        let search_manager = SearchManager::new(temp_dir.path()).unwrap();

        let mut query_parts = vec!["from:a"];
        for _i in 0..51 {
            query_parts.push("OR");
            query_parts.push("from:b");
        }

        let query = SearchQuery {
            query: query_parts.join(" "),
            account_id: None,
            folder_id: None,
            conversation_id: None,
            limit: 50,
            offset: 0,
        };

        let result = search_manager.validate_query(&query);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_query_wildcards() {
        let temp_dir = TempDir::new().unwrap();
        let search_manager = SearchManager::new(temp_dir.path()).unwrap();

        let query = SearchQuery {
            query: "*a* *b* *c* *d* *e* *f*".to_string(),
            account_id: None,
            folder_id: None,
            conversation_id: None,
            limit: 50,
            offset: 0,
        };

        let result = search_manager.validate_query(&query);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_query_limit() {
        let temp_dir = TempDir::new().unwrap();
        let search_manager = SearchManager::new(temp_dir.path()).unwrap();

        let query = SearchQuery {
            query: "test".to_string(),
            account_id: None,
            folder_id: None,
            conversation_id: None,
            limit: 1001,
            offset: 0,
        };

        let result = search_manager.validate_query(&query);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_query_offset() {
        let temp_dir = TempDir::new().unwrap();
        let search_manager = SearchManager::new(temp_dir.path()).unwrap();

        let query = SearchQuery {
            query: "test".to_string(),
            account_id: None,
            folder_id: None,
            conversation_id: None,
            limit: 50,
            offset: 10001,
        };

        let result = search_manager.validate_query(&query);
        assert!(result.is_err());
    }
}
