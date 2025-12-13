use crate::database::{
    error::DatabaseError,
    models::contact::{Contact, ContactSummary},
};
use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

#[async_trait]
pub trait ContactRepository {
    // Core CRUD operations
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Contact>, DatabaseError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Contact>, DatabaseError>;
    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<Contact>, DatabaseError>;
    async fn create(&self, contact: &Contact) -> Result<Uuid, DatabaseError>;
    async fn update(&self, contact: &Contact) -> Result<(), DatabaseError>;
    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError>;

    async fn increment_send_count(
        &self,
        email: &str,
        name: Option<&str>,
        sent_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Uuid, DatabaseError>;
    async fn increment_receive_count(
        &self,
        email: &str,
        name: Option<&str>,
    ) -> Result<Uuid, DatabaseError>;
    async fn reset_counters(&self) -> Result<(), DatabaseError>;

    async fn search_contacts(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ContactSummary>, DatabaseError>;
    async fn get_top_contacts(&self, limit: i64) -> Result<Vec<ContactSummary>, DatabaseError>;

    async fn update_avatar(
        &self,
        id: Uuid,
        avatar_type: &str,
        avatar_path: Option<String>,
    ) -> Result<(), DatabaseError>;
    async fn find_contacts_without_avatars(
        &self,
        limit: i64,
    ) -> Result<Vec<Contact>, DatabaseError>;
}

pub struct SqliteContactRepository {
    pool: SqlitePool,
}

impl SqliteContactRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    async fn get_or_create_by_email(
        &self,
        email: &str,
        name: Option<&str>,
    ) -> Result<Uuid, DatabaseError> {
        if let Some(contact) = self.find_by_email(email).await? {
            return Ok(contact.id);
        }

        let contact = Contact {
            id: Uuid::now_v7(),
            email: email.to_lowercase(),
            display_name: name.map(ToString::to_string),
            first_name: None,
            last_name: None,
            company: None,
            source: "observed".to_string(),
            avatar_type: "unprocessed".to_string(),
            avatar_path: None,
            send_count: 0,
            receive_count: 0,
            last_used_at: Some(Utc::now()),
            first_seen_at: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.create(&contact).await
    }
}

#[async_trait]
impl ContactRepository for SqliteContactRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Contact>, DatabaseError> {
        sqlx::query_as::<_, Contact>("SELECT * FROM contacts WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<Contact>, DatabaseError> {
        sqlx::query_as::<_, Contact>("SELECT * FROM contacts WHERE email = ?")
            .bind(email.to_lowercase())
            .fetch_optional(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<Contact>, DatabaseError> {
        sqlx::query_as::<_, Contact>(
            "SELECT * FROM contacts ORDER BY display_name, email LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    async fn create(&self, contact: &Contact) -> Result<Uuid, DatabaseError> {
        let id = contact.id.to_string();
        let email_lowercase = contact.email.to_lowercase();

        sqlx::query!(
            r#"
            INSERT INTO contacts (
                id, email, display_name, first_name, last_name, company,
                source, avatar_type, avatar_path, send_count, receive_count,
                last_used_at, first_seen_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            id,
            email_lowercase,
            contact.display_name,
            contact.first_name,
            contact.last_name,
            contact.company,
            contact.source,
            contact.avatar_type,
            contact.avatar_path,
            contact.send_count,
            contact.receive_count,
            contact.last_used_at,
            contact.first_seen_at
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(contact.id)
    }

    async fn update(&self, contact: &Contact) -> Result<(), DatabaseError> {
        let id = contact.id.to_string();

        sqlx::query!(
            r#"
            UPDATE contacts
            SET display_name = ?, first_name = ?, last_name = ?, company = ?,
                source = ?, avatar_type = ?, avatar_path = ?, send_count = ?,
                receive_count = ?, last_used_at = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            contact.display_name,
            contact.first_name,
            contact.last_name,
            contact.company,
            contact.source,
            contact.avatar_type,
            contact.avatar_path,
            contact.send_count,
            contact.receive_count,
            contact.last_used_at,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError> {
        let id = id.to_string();
        sqlx::query!("DELETE FROM contacts WHERE id = ?", id)
            .execute(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn increment_send_count(
        &self,
        email: &str,
        name: Option<&str>,
        sent_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Uuid, DatabaseError> {
        let contact_id = self.get_or_create_by_email(email, name).await?;
        let contact_id_str = contact_id.to_string();

        if let Some(sent_timestamp) = sent_at {
            sqlx::query!(
                r#"
                UPDATE contacts
                SET send_count = send_count + 1,
                    last_used_at = MAX(COALESCE(last_used_at, ?), ?),
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = ?
                "#,
                sent_timestamp,
                sent_timestamp,
                contact_id_str
            )
            .execute(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)?;
        } else {
            sqlx::query!(
                r#"
                UPDATE contacts
                SET send_count = send_count + 1,
                    last_used_at = CURRENT_TIMESTAMP,
                    updated_at = CURRENT_TIMESTAMP
                WHERE id = ?
                "#,
                contact_id_str
            )
            .execute(&self.pool)
            .await
            .map_err(DatabaseError::ConnectionError)?;
        }

        Ok(contact_id)
    }

    async fn increment_receive_count(
        &self,
        email: &str,
        name: Option<&str>,
    ) -> Result<Uuid, DatabaseError> {
        // First ensure contact exists
        let contact_id = self.get_or_create_by_email(email, name).await?;
        let contact_id_str = contact_id.to_string();

        sqlx::query!(
            r#"
            UPDATE contacts
            SET receive_count = receive_count + 1,
                last_used_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            contact_id_str
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(contact_id)
    }

    async fn search_contacts(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ContactSummary>, DatabaseError> {
        let search_pattern = format!("%{}%", query);
        let alt_search_pattern = format!("{}%", query);

        let results = sqlx::query!(
            r#"
            SELECT
                id, email, display_name, avatar_path, send_count, receive_count, last_used_at
            FROM contacts
            WHERE email LIKE ? OR display_name LIKE ? OR first_name LIKE ? OR last_name LIKE ?
            ORDER BY
                CASE WHEN email LIKE ? THEN 3
                     WHEN display_name LIKE ? THEN 2
                     ELSE 1
                END DESC,
                send_count + receive_count DESC,
                last_used_at DESC
            LIMIT ?
            "#,
            search_pattern,
            search_pattern,
            search_pattern,
            search_pattern,
            alt_search_pattern,
            alt_search_pattern,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        log::info!("Fetched search contacts: {:?}", results);

        let summaries = results
            .into_iter()
            .map(|row| {
                let base_score = row.send_count + row.receive_count;
                let days_boost = if let Some(last_used) = row.last_used_at {
                    let last_used_utc = Utc.from_utc_datetime(&last_used);
                    let days_since = (Utc::now() - last_used_utc).num_days();
                    if days_since < 30 {
                        (30 - days_since as i64) * 5
                    } else {
                        0
                    }
                } else {
                    0
                };

                let last_used_at = row.last_used_at.map(|dt| Utc.from_utc_datetime(&dt));

                ContactSummary {
                    id: Uuid::parse_str(row.id.as_str()).unwrap(),
                    email: row.email,
                    display_name: row.display_name,
                    avatar_path: row.avatar_path,
                    send_count: row.send_count,
                    receive_count: row.receive_count,
                    last_used_at,
                    usage_score: base_score + days_boost,
                }
            })
            .collect();

        Ok(summaries)
    }

    async fn get_top_contacts(&self, limit: i64) -> Result<Vec<ContactSummary>, DatabaseError> {
        let results = sqlx::query!(
            r#"
            SELECT
                id, email, display_name, avatar_path, send_count, receive_count, last_used_at
            FROM contacts
            ORDER BY
                send_count * 2 + receive_count DESC,
                last_used_at DESC
            LIMIT ?
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        log::info!("Fetched top contacts: {:?}", results);

        let summaries = results
            .into_iter()
            .map(|row| {
                let last_used_at = row.last_used_at.map(|dt| Utc.from_utc_datetime(&dt));

                ContactSummary {
                    id: Uuid::parse_str(row.id.as_str()).unwrap(),
                    email: row.email,
                    display_name: row.display_name,
                    avatar_path: row.avatar_path,
                    send_count: row.send_count,
                    receive_count: row.receive_count,
                    last_used_at,
                    usage_score: row.send_count * 2 + row.receive_count,
                }
            })
            .collect();

        Ok(summaries)
    }

    async fn update_avatar(
        &self,
        id: Uuid,
        avatar_type: &str,
        avatar_path: Option<String>,
    ) -> Result<(), DatabaseError> {
        let id = id.to_string();

        sqlx::query!(
            r#"
            UPDATE contacts
            SET avatar_type = ?, avatar_path = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            avatar_type,
            avatar_path,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }

    async fn find_contacts_without_avatars(
        &self,
        limit: i64,
    ) -> Result<Vec<Contact>, DatabaseError> {
        sqlx::query_as::<_, Contact>(
            r#"
            SELECT * FROM contacts
            WHERE avatar_type = 'unprocessed'
            ORDER BY receive_count + send_count DESC, last_used_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)
    }

    async fn reset_counters(&self) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
            UPDATE contacts
            SET send_count = 0, receive_count = 0, last_used_at = NULL
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(DatabaseError::ConnectionError)?;

        Ok(())
    }
}
