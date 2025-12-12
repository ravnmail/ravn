use crate::database::models::email::{Email, EmailAddress};
use crate::database::{error::DatabaseError, repositories::ContactRepository};
use std::sync::Arc;
use uuid::Uuid;

pub struct ContactExtractor {
    contact_repo: Arc<dyn ContactRepository + Send + Sync>,
}

impl ContactExtractor {
    pub fn new(contact_repo: Arc<dyn ContactRepository + Send + Sync>) -> Self {
        Self { contact_repo }
    }

    pub async fn extract_from_sender(&self, from: &EmailAddress) -> Result<Uuid, DatabaseError> {
        self.contact_repo
            .increment_receive_count(&from.address, from.name.as_deref())
            .await
    }

    pub async fn extract_from_recipients(
        &self,
        _account_id: Uuid,
        recipients: &[EmailAddress],
        sent_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<Uuid>, DatabaseError> {
        let mut contact_ids = Vec::new();

        for recipient in recipients {
            let contact_id = self
                .contact_repo
                .increment_send_count(&recipient.address, recipient.name.as_deref(), sent_at)
                .await?;
            contact_ids.push(contact_id);
        }

        Ok(contact_ids)
    }

    pub async fn extract_and_store_from_received_email(
        &self,
        email: &Email,
    ) -> Result<(), DatabaseError> {
        self.extract_from_sender(email.from()).await?;

        for addr in email
            .to()
            .iter()
            .chain(email.cc().iter())
            .chain(email.bcc().iter())
        {
            let _ = self
                .contact_repo
                .increment_receive_count(&addr.address, addr.name.as_deref())
                .await;
        }

        Ok(())
    }

    pub async fn extract_and_store_from_sent_email(
        &self,
        to: &[EmailAddress],
        cc: &[EmailAddress],
        bcc: &[EmailAddress],
        sent_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), DatabaseError> {
        for addr in to.iter().chain(cc.iter()).chain(bcc.iter()) {
            let _ = self
                .contact_repo
                .increment_send_count(&addr.address, addr.name.as_deref(), sent_at)
                .await;
        }

        Ok(())
    }
}
