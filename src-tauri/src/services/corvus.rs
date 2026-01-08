use crate::config::Settings;
use crate::database::models::email::Email;
use crate::licensing::LicenseManager;
use openrouter_rs::api::chat::{
    ChatCompletionRequest as ChatRequest, Message as OpenRouterChatMessage,
};
use openrouter_rs::client::OpenRouterClient;
use openrouter_rs::types::{ProviderPreferences, ProviderSortBy, Role};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const MAX_PRIOR_EMAIL_TOKENS: usize = 500;
const MAX_CURRENT_TEXT_TOKENS: usize = 300;
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
}

#[derive(Debug, Clone)]
pub struct GenerateSearchQueryRequest {
    pub natural_language_query: String,
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

    pub async fn analyze_email(&self, email: &Email) -> Result<EmailAnalysis, String> {
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

        // Extract email fields
        let from = format!(
            "{} <{}>",
            email.from().name.clone().unwrap_or_default(),
            email.from().address
        )
        .trim_start_matches(" <")
        .trim_end_matches(">")
        .to_string();

        let to = email
            .to()
            .iter()
            .map(|addr| {
                format!(
                    "{} <{}>",
                    addr.name.clone().unwrap_or_default(),
                    addr.address
                )
                .trim_start_matches(" <")
                .trim_end_matches(">")
                .to_string()
            })
            .collect::<Vec<_>>()
            .join(", ");

        let cc = email
            .cc()
            .iter()
            .map(|addr| {
                format!(
                    "{} <{}>",
                    addr.name.clone().unwrap_or_default(),
                    addr.address
                )
                .trim_start_matches(" <")
                .trim_end_matches(">")
                .to_string()
            })
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

        let current_datetime = chrono::Utc::now().to_rfc3339();
        let received_at = email.received_at.to_rfc3339();

        let user_prompt = format!(
            r#"Current DateTime: {}

Email Details:
From: {}
To: {}
Cc: {}
Subject: {}
Received At: {}
Content:
{}"#,
            current_datetime, from, to, cc, subject, received_at, content,
        );

        let system_with_style = if writing_style.is_empty() {
            system_prompt
        } else {
            format!("{}\n{}", system_prompt, writing_style)
        };

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

        serde_json::from_str::<EmailAnalysis>(&response_text).map_err(|e| {
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
