use crate::config::Settings;
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessage,
    CreateChatCompletionRequestArgs, CreateCompletionRequestArgs,
};
use async_openai::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const MAX_PRIOR_EMAIL_TOKENS: usize = 500;
const MAX_CURRENT_TEXT_TOKENS: usize = 300;
const APPROX_CHARS_PER_TOKEN: usize = 4;

pub struct CorvusService {
    settings: Arc<Settings>,
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
pub struct AvailableModel {
    pub id: String,
    pub name: String,
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

impl CorvusService {
    pub fn new(settings: Arc<Settings>) -> Self {
        Self { settings }
    }

    fn get_api_key(&self) -> Result<String, String> {
        self.settings.get::<String>("ai.api.key").map_err(|_| {
            "API key not configured in settings. Please set ai.api.key in settings.json".to_string()
        })
    }

    fn get_base_url(&self) -> Result<String, String> {
        self.settings
            .get::<String>("ai.api.baseUrl")
            .or_else(|_| Ok("https://api.openai.com/v1".to_string()))
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

    fn get_client(&self) -> Result<Client<OpenAIConfig>, String> {
        let api_key = self.get_api_key()?;
        let base_url = self.get_base_url()?;
        let config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(base_url);
        Ok(Client::with_config(config))
    }

    pub async fn ask_ai(&self, request: AskAiRequest) -> Result<String, String> {
        log::debug!(
            "Processing ask_ai request with {} messages",
            request.history.len()
        );

        let client = self.get_client()?;
        let model = self.get_model("normal")?;
        let mut system_prompt = self.get_prompt("askAi")?;
        system_prompt.push_str(&self.build_writing_style_context());

        let messages: Vec<ChatCompletionRequestMessage> = request
            .history
            .into_iter()
            .map(|msg| {
                if msg.role == "user" {
                    ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                        content: msg.content.into(),
                        name: None,
                    })
                } else {
                    ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                        content: msg.content.into(),
                        name: None,
                    })
                }
            })
            .collect();

        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder.model(model);
        request_builder.messages(messages);

        let request = request_builder
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let response = client
            .chat()
            .create(request)
            .await
            .map_err(|e| format!("OpenAI API request failed: {}", e))?;

        response
            .choices
            .first()
            .ok_or_else(|| "No choices in response".to_string())
            .and_then(|choice| {
                choice
                    .message
                    .content
                    .clone()
                    .ok_or_else(|| "No content in response".to_string())
            })
    }

    pub async fn ask_ai_streaming(
        &self,
        request: AskAiRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<String>, String> {
        log::debug!(
            "Processing ask_ai streaming request with {} messages",
            request.history.len()
        );

        let client = self.get_client()?;
        let model = self.get_model("normal")?;
        let mut system_prompt = self.get_prompt("askAi")?;
        system_prompt.push_str(&self.build_writing_style_context());

        let messages: Vec<ChatCompletionRequestMessage> = request
            .history
            .into_iter()
            .map(|msg| {
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: msg.content.into(),
                    name: None,
                })
            })
            .collect();

        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder.model(model);
        request_builder.messages(messages);
        request_builder.stream(true);

        let request = request_builder
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            match client.chat().create_stream(request).await {
                Ok(mut stream) => {
                    use futures::StreamExt;
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(response) => {
                                if let Some(choice) = response.choices.first() {
                                    if let Some(content) = &choice.delta.content {
                                        let _ = tx.send(content.clone()).await;
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Streaming error: {}", e);
                                let _ = tx.send(format!("ERROR: {}", e)).await;
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to create stream: {}", e);
                    let _ = tx.send(format!("ERROR: {}", e)).await;
                }
            }
        });

        Ok(rx)
    }

    pub async fn generate_email_completion(
        &self,
        request: EmailCompletionRequest,
    ) -> Result<String, String> {
        log::debug!("Processing email completion request");

        let client = self.get_client()?;
        let model = self.get_model("fast")?;

        let user_message = self.build_autocomplete_prompt(&request);
        let mut system_prompt = self.get_prompt("generateCompletion")?;
        system_prompt.push_str(&self.build_writing_style_context());

        let prompt = format!("{}\n\n{}", system_prompt, user_message);

        let mut request_builder = CreateCompletionRequestArgs::default();
        request_builder.model(model);
        request_builder.prompt(prompt);

        let request = request_builder
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let response = client
            .completions()
            .create(request)
            .await
            .map_err(|e| format!("OpenAI API request failed: {}", e))?;

        response
            .choices
            .first()
            .ok_or_else(|| "No choices in response".to_string())
            .map(|choice| choice.text.clone())
    }

    pub async fn generate_email_completion_streaming(
        &self,
        request: EmailCompletionRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<String>, String> {
        log::debug!("Processing email completion streaming request");

        let client = self.get_client()?;
        let model = self.get_model("fast")?;

        let user_message = self.build_autocomplete_prompt(&request);
        let mut system_prompt = self.get_prompt("generateCompletion")?;
        system_prompt.push_str(&self.build_writing_style_context());

        let prompt = format!("{}\n\n{}", system_prompt, user_message);

        let mut request_builder = CreateCompletionRequestArgs::default();
        request_builder.model(model);
        request_builder.prompt(prompt);
        request_builder.stream(true);

        let request = request_builder
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            match client.completions().create_stream(request).await {
                Ok(mut stream) => {
                    use futures::StreamExt;
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(response) => {
                                if let Some(choice) = response.choices.first() {
                                    let _ = tx.send(choice.text.clone()).await;
                                }
                            }
                            Err(e) => {
                                log::error!("Streaming error: {}", e);
                                let _ = tx.send(format!("ERROR: {}", e)).await;
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to create stream: {}", e);
                    let _ = tx.send(format!("ERROR: {}", e)).await;
                }
            }
        });

        Ok(rx)
    }

    pub async fn generate_subject(
        &self,
        request: GenerateSubjectRequest,
    ) -> Result<String, String> {
        log::debug!("Processing generate subject request");

        let client = self.get_client()?;
        let model = self.get_model("normal")?;
        let mut system_prompt = self.get_prompt("generateSubject")?;
        system_prompt.push_str(&self.build_writing_style_context());

        let prompt = format!(
            "Generate a concise, professional email subject line based on the following email content. The subject should be clear, specific, and engaging. Email content: {}\n\nCurrent subject (if any): {}\n\nGenerate only the subject line, nothing else.",
            request.body_content,
            request.current_subject.unwrap_or_else(|| "None".to_string())
        );

        let messages = vec![ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: prompt.into(),
                name: None,
            },
        )];

        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder.model(model);
        request_builder.messages(messages);

        let request = request_builder
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let response = client
            .chat()
            .create(request)
            .await
            .map_err(|e| format!("OpenAI API request failed: {}", e))?;

        response
            .choices
            .first()
            .ok_or_else(|| "No choices in response".to_string())
            .and_then(|choice| {
                choice
                    .message
                    .content
                    .clone()
                    .ok_or_else(|| "No content in response".to_string())
            })
    }

    pub async fn generate_subject_streaming(
        &self,
        request: GenerateSubjectRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<String>, String> {
        log::debug!("Processing generate subject streaming request");

        let client = self.get_client()?;
        let model = self.get_model("normal")?;
        let mut system_prompt = self.get_prompt("generateSubject")?;
        system_prompt.push_str(&self.build_writing_style_context());

        let prompt = format!(
            "Generate a concise, professional email subject line based on the following email content. The subject should be clear, specific, and engaging. Email content: {}\n\nCurrent subject (if any): {}\n\nGenerate only the subject line, nothing else.",
            request.body_content,
            request.current_subject.unwrap_or_else(|| "None".to_string())
        );

        let messages = vec![ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: prompt.into(),
                name: None,
            },
        )];

        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder.model(model);
        request_builder.messages(messages);
        request_builder.stream(true);

        let request = request_builder
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            match client.chat().create_stream(request).await {
                Ok(mut stream) => {
                    use futures::StreamExt;
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(response) => {
                                if let Some(choice) = response.choices.first() {
                                    if let Some(content) = &choice.delta.content {
                                        let _ = tx.send(content.clone()).await;
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Streaming error: {}", e);
                                let _ = tx.send(format!("ERROR: {}", e)).await;
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to create stream: {}", e);
                    let _ = tx.send(format!("ERROR: {}", e)).await;
                }
            }
        });

        Ok(rx)
    }

    pub async fn analyze_email(
        &self,
        subject: String,
        content: String,
        received_at: Option<String>,
    ) -> Result<EmailAnalysis, String> {
        log::debug!("Processing email analysis request");

        let client = self.get_client()?;
        let model = self.get_model("normal")?;
        let mut system_prompt = self.get_prompt("analyzeEmail")?;
        system_prompt.push_str(&self.build_writing_style_context());

        let current_datetime = chrono::Utc::now().to_rfc3339();
        let received_at = received_at.unwrap_or_else(|| current_datetime.clone());

        let prompt = format!(
            r#"Analyze the following email and provide:
1. A concise gist (1-2 sentences) summarizing the key point or request
2. 1-4 appropriate response suggestions with titles and content

Current DateTime: {}

Email Details:
Subject: {}
Received At: {}
Content:
{}

Respond with ONLY a valid JSON response in this exact format:
{{
  "gist": "Brief summary of the email",
  "responses": [
    {{
      "title": "Response option title",
      "content": "Full response content in plain text"
    }}
  ]
}}

Important:
- You must respond in valid JSON object only! No additional text, markup or explanations
- Gist should be actionable and highlight what requires attention
- Responses should be professional, contextual, and ready to send
- Match the tone and language of the original email
- Provide 1-4 response options depending on the email context
- Response content should be complete, well-structured messages"#,
            current_datetime, subject, received_at, content,
        );

        let messages = vec![ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: prompt.into(),
                name: None,
            },
        )];

        let mut request_builder = CreateChatCompletionRequestArgs::default();
        request_builder.model(model);
        request_builder.messages(messages);

        let request = request_builder
            .build()
            .map_err(|e| format!("Failed to build request: {}", e))?;

        let response = client
            .chat()
            .create(request)
            .await
            .map_err(|e| format!("OpenAI API request failed: {}", e))?;

        let response_text = response
            .choices
            .first()
            .ok_or_else(|| "No choices in response".to_string())?
            .message
            .content
            .clone()
            .ok_or_else(|| "No content in response".to_string())?;

        serde_json::from_str::<EmailAnalysis>(&response_text).map_err(|e| {
            format!(
                "Failed to parse analysis JSON: {}. Content: {}",
                e, response_text
            )
        })
    }

    pub async fn get_available_models(&self) -> Result<Vec<AvailableModel>, String> {
        log::debug!("Fetching available models");
        Ok(vec![
            AvailableModel {
                id: "gpt-4o".to_string(),
                name: "GPT-4 Omni".to_string(),
            },
            AvailableModel {
                id: "gpt-4o-mini".to_string(),
                name: "GPT-4 Omni Mini".to_string(),
            },
            AvailableModel {
                id: "gpt-4-turbo".to_string(),
                name: "GPT-4 Turbo".to_string(),
            },
            AvailableModel {
                id: "gpt-4".to_string(),
                name: "GPT-4".to_string(),
            },
            AvailableModel {
                id: "gpt-3.5-turbo".to_string(),
                name: "GPT-3.5 Turbo".to_string(),
            },
        ])
    }

    fn build_autocomplete_prompt(&self, context: &EmailCompletionRequest) -> String {
        let mut message = String::new();

        message.push_str(&format!(
            "Sender: {}\nSubject: {}\nIs Reply: {}\nRecipients: {}\n\n",
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
