use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::error::SyncResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub csrf_token: String,
    pub pkce_verifier: String,
    pub provider: String,
    pub account_id: Uuid,
    pub redirect_uri: String,
    pub created_at: DateTime<Utc>,
}

impl OAuthState {
    pub fn is_expired(&self) -> bool {
        let expiry = self.created_at + Duration::minutes(10);
        Utc::now() > expiry
    }
}

/// Thread-safe OAuth state manager with automatic expiration
pub struct OAuthStateManager {
    states: Arc<RwLock<HashMap<String, OAuthState>>>,
}

impl OAuthStateManager {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store OAuth state for later retrieval
    pub async fn store(&self, csrf_token: String, state: OAuthState) -> SyncResult<()> {
        let mut states = self.states.write().await;
        states.insert(csrf_token, state);
        Ok(())
    }

    /// Retrieve and remove OAuth state
    pub async fn get_and_remove(&self, csrf_token: &str) -> SyncResult<OAuthState> {
        self.cleanup_expired().await;

        let mut states = self.states.write().await;
        let state = states.remove(csrf_token).ok_or_else(|| {
            super::error::SyncError::OAuth2Error("OAuth state not found or expired".to_string())
        })?;

        if state.is_expired() {
            return Err(super::error::SyncError::OAuth2Error(
                "OAuth state has expired".to_string(),
            ));
        }

        Ok(state)
    }

    /// Clean up expired states
    async fn cleanup_expired(&self) {
        let mut states = self.states.write().await;
        states.retain(|_, state| !state.is_expired());
    }

    /// Get state without removing (for debugging/testing)
    #[allow(dead_code)]
    pub async fn get(&self, csrf_token: &str) -> Option<OAuthState> {
        let states = self.states.read().await;
        states.get(csrf_token).cloned()
    }
}

impl Default for OAuthStateManager {
    fn default() -> Self {
        Self::new()
    }
}
