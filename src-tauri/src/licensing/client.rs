use super::types::*;
use reqwest::Client;
use std::time::Duration;

pub struct ActivationClient {
    base_url: String,
    client: Client,
}

impl ActivationClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Self { base_url, client }
    }

    pub async fn activate(
        &self,
        instance_name: String,
        license_key: String,
    ) -> Result<ActivationResponse, ActivationError> {
        let url = format!("{}/v1/activate", self.base_url);
        let request = ActivationRequest {
            instance_name,
            license_key,
        };

        log::info!("Activating license at {}", url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                log::error!("Failed to send activation request: {}", e);
                ActivationError::RequestFailed(e)
            })?;

        let status = response.status();
        let body = response.text().await.map_err(|e| {
            log::error!("Failed to read activation response: {}", e);
            ActivationError::RequestFailed(e)
        })?;

        if status.is_success() {
            log::info!("License activated successfully");
            serde_json::from_str(&body).map_err(|e| {
                log::error!(
                    "Failed to parse activation response: {} - Body: {}",
                    e,
                    body
                );
                ActivationError::InvalidResponse(format!(
                    "Failed to parse response: {} - Body: {}",
                    e, body
                ))
            })
        } else {
            log::error!("License activation failed: {} - {}", status, body);
            Err(ActivationError::from_response(status.as_u16(), &body))
        }
    }

    pub async fn validate(
        &self,
        license_key: String,
    ) -> Result<ActivationResponse, ActivationError> {
        let url = format!("{}/v1/validate", self.base_url);
        let request = ValidationRequest { license_key };

        log::debug!("Validating license at {}", url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                log::error!("Failed to send validation request: {}", e);
                ActivationError::RequestFailed(e)
            })?;

        let status = response.status();
        let body = response.text().await.map_err(|e| {
            log::error!("Failed to read validation response: {}", e);
            ActivationError::RequestFailed(e)
        })?;

        if status.is_success() {
            log::debug!("License validated successfully");
            serde_json::from_str(&body).map_err(|e| {
                log::error!(
                    "Failed to parse validation response: {} - Body: {}",
                    e,
                    body
                );
                ActivationError::InvalidResponse(format!(
                    "Failed to parse response: {} - Body: {}",
                    e, body
                ))
            })
        } else {
            log::error!("License validation failed: {} - {}", status, body);
            Err(ActivationError::from_response(status.as_u16(), &body))
        }
    }

    pub async fn start_trial(
        &self,
        instance_name: String,
        email: String,
    ) -> Result<ActivationResponse, ActivationError> {
        let url = format!("{}/v1/trial", self.base_url);
        let request = TrialRequest {
            instance_name,
            email,
        };

        log::info!("Starting trial at {}", url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                log::error!("Failed to send trial request: {}", e);
                ActivationError::RequestFailed(e)
            })?;

        let status = response.status();
        let body = response.text().await.map_err(|e| {
            log::error!("Failed to read trial response: {}", e);
            ActivationError::RequestFailed(e)
        })?;

        if status.is_success() {
            log::info!("Trial started successfully");
            serde_json::from_str(&body).map_err(|e| {
                log::error!("Failed to parse trial response: {} - Body: {}", e, body);
                ActivationError::InvalidResponse(format!(
                    "Failed to parse response: {} - Body: {}",
                    e, body
                ))
            })
        } else {
            log::error!("Trial activation failed: {} - {}", status, body);
            Err(ActivationError::from_response(status.as_u16(), &body))
        }
    }

    pub async fn is_service_reachable(&self) -> bool {
        let url = format!("{}/v1/validate", self.base_url);
        match self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(_) => true,
            Err(e) => {
                log::debug!("Activation service not reachable: {}", e);
                false
            }
        }
    }
}
