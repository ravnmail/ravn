use crate::database::repositories::{EmailRepository, RepositoryFactory};
use crate::services::corvus::{
    AskAiRequest, AvailableModel, ChatMessage, CorvusService, EmailAnalysis,
    EmailCompletionRequest, EmailMetadata, GenerateSearchQueryRequest, GenerateSubjectRequest,
};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::{command, Emitter, State};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct AskAiHistory {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct AskAiContext {
    pub history: Vec<AskAiHistory>,
}

#[derive(Debug, Deserialize)]
pub struct EmailContextRequest {
    pub metadata: EmailMetadataRequest,
    pub prior_email: Option<String>,
    pub current_text: String,
    pub cursor_position: usize,
}

#[derive(Debug, Deserialize)]
pub struct EmailMetadataRequest {
    pub sender: String,
    pub subject: String,
    pub is_reply: bool,
    pub recipients: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateSubjectContextRequest {
    pub body_content: String,
    pub sender: String,
    pub recipients: Vec<String>,
    pub is_reply: bool,
    pub current_subject: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateSearchQueryContextRequest {
    pub natural_language_query: String,
}

#[derive(Debug, Serialize)]
pub struct GenerateSearchQueryResult {
    pub query: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AutoCompletionResult {
    pub completion: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EmailAnalysisResponse {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct EmailAnalysisResult {
    pub analysis: Option<EmailAnalysis>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AvailableModelsResult {
    pub models: Vec<AvailableModel>,
    pub error: Option<String>,
}

fn get_ai_service(state: &State<'_, AppState>) -> std::sync::Arc<CorvusService> {
    std::sync::Arc::clone(&state.ai_service)
}

#[command]
pub async fn ask_ai(
    state: State<'_, AppState>,
    context: AskAiContext,
) -> Result<AutoCompletionResult, String> {
    log::debug!(
        "Received ask_ai request with {} messages",
        context.history.len()
    );

    let ai_service = get_ai_service(&state);
    let request = AskAiRequest {
        history: context
            .history
            .into_iter()
            .map(|h| ChatMessage {
                role: h.role,
                content: h.content,
            })
            .collect(),
    };

    match ai_service.ask_ai(request).await {
        Ok(completion) => Ok(AutoCompletionResult {
            completion,
            error: None,
        }),
        Err(e) => {
            log::error!("ask_ai error: {}", e);
            Ok(AutoCompletionResult {
                completion: String::new(),
                error: Some(e),
            })
        }
    }
}

#[command]
pub async fn generate_email_completion(
    state: State<'_, AppState>,
    context: EmailContextRequest,
) -> Result<AutoCompletionResult, String> {
    log::debug!("Received generate_email_completion request");

    let ai_service = get_ai_service(&state);
    let request = EmailCompletionRequest {
        metadata: EmailMetadata {
            sender: context.metadata.sender,
            subject: context.metadata.subject,
            is_reply: context.metadata.is_reply,
            recipients: context.metadata.recipients,
        },
        prior_email: context.prior_email,
        current_text: context.current_text,
        cursor_position: context.cursor_position,
    };

    match ai_service.generate_email_completion(request).await {
        Ok(completion) => Ok(AutoCompletionResult {
            completion,
            error: None,
        }),
        Err(e) => {
            log::error!("generate_email_completion error: {}", e);
            Ok(AutoCompletionResult {
                completion: String::new(),
                error: Some(e),
            })
        }
    }
}

#[command]
pub async fn generate_subject(
    state: State<'_, AppState>,
    context: GenerateSubjectContextRequest,
) -> Result<AutoCompletionResult, String> {
    log::debug!("Received generate_subject request");

    let ai_service = get_ai_service(&state);
    let request = GenerateSubjectRequest {
        body_content: context.body_content,
        sender: context.sender,
        recipients: context.recipients,
        is_reply: context.is_reply,
        current_subject: context.current_subject,
    };

    match ai_service.generate_subject(request).await {
        Ok(subject) => Ok(AutoCompletionResult {
            completion: subject,
            error: None,
        }),
        Err(e) => {
            log::error!("generate_subject error: {}", e);
            Ok(AutoCompletionResult {
                completion: String::new(),
                error: Some(e),
            })
        }
    }
}

#[command]
pub async fn generate_search_query(
    state: State<'_, AppState>,
    natural_language_query: String,
) -> Result<GenerateSearchQueryResult, String> {
    log::debug!("Received generate_search_query request");

    let ai_service = get_ai_service(&state);
    let request = GenerateSearchQueryRequest {
        natural_language_query,
    };

    match ai_service.generate_search_query(request).await {
        Ok(query) => Ok(GenerateSearchQueryResult { query, error: None }),
        Err(e) => {
            log::error!("generate_search_query error: {}", e);
            Ok(GenerateSearchQueryResult {
                query: String::new(),
                error: Some(e),
            })
        }
    }
}

#[command]
pub async fn analyze_email_with_ai(
    state: State<'_, AppState>,
    email_id: Uuid,
) -> Result<EmailAnalysisResult, String> {
    log::debug!("Analyzing email with ID: {}", email_id);

    let repo_factory = RepositoryFactory::new(state.db_pool.clone());
    let email_repo = repo_factory.email_repository();

    let email = email_repo
        .find_by_id(email_id)
        .await
        .map_err(|e| format!("Failed to fetch email: {}", e))?
        .ok_or_else(|| "Email not found".to_string())?;

    if let Some(ref cache) = email.ai_cache {
        if let Ok(cached_analysis) = serde_json::from_str::<EmailAnalysis>(cache) {
            log::debug!("Returning cached analysis for email {}", email_id);
            return Ok(EmailAnalysisResult {
                analysis: Some(cached_analysis),
                error: None,
            });
        }
    }

    let email_content = email
        .body_plain
        .clone()
        .or_else(|| email.body_html.clone())
        .ok_or_else(|| "Email has no content".to_string())?;

    let subject = email
        .subject
        .clone()
        .unwrap_or_else(|| "(No subject)".to_string());

    let ai_service = get_ai_service(&state);

    let received_at = Some(email.received_at.to_string());

    match ai_service
        .analyze_email(subject, email_content, received_at)
        .await
    {
        Ok(analysis) => {
            let analysis_json = serde_json::to_string(&analysis)
                .map_err(|e| format!("Failed to serialize analysis: {}", e))?;

            let mut updated_email = email;
            updated_email.ai_cache = Some(analysis_json);

            email_repo
                .update(&updated_email)
                .await
                .map_err(|e| format!("Failed to update email with analysis: {}", e))?;

            log::debug!("Analysis stored for email {}", email_id);

            if let Err(e) = state
                .app_handle
                .emit("email:ai-analysis-complete", email_id.to_string())
            {
                log::error!("Failed to emit event: {}", e);
            }

            Ok(EmailAnalysisResult {
                analysis: Some(analysis),
                error: None,
            })
        }
        Err(e) => {
            log::error!("analyze_email error: {}", e);
            Ok(EmailAnalysisResult {
                analysis: None,
                error: Some(e),
            })
        }
    }
}

#[command]
pub async fn get_available_models(
    state: State<'_, AppState>,
) -> Result<AvailableModelsResult, String> {
    log::debug!("Fetching available models");

    let ai_service = get_ai_service(&state);

    match ai_service.get_available_models().await {
        Ok(models) => Ok(AvailableModelsResult {
            models,
            error: None,
        }),
        Err(e) => {
            log::error!("get_available_models error: {}", e);
            Ok(AvailableModelsResult {
                models: vec![],
                error: Some(e),
            })
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WritingStyleResult {
    pub style: Option<String>,
    pub error: Option<String>,
}

#[command]
pub fn get_writing_style(state: State<'_, AppState>) -> Result<WritingStyleResult, String> {
    log::debug!("Fetching writing style settings");

    let style = state
        .settings
        .get::<Option<String>>("ai.writingStyle")
        .ok()
        .flatten();

    Ok(WritingStyleResult { style, error: None })
}

#[derive(Debug, Deserialize)]
pub struct SetWritingStyleRequest {
    pub style: Option<String>,
}

#[command]
pub async fn set_writing_style(
    state: State<'_, AppState>,
    request: SetWritingStyleRequest,
) -> Result<WritingStyleResult, String> {
    log::debug!("Setting writing style");

    if let Some(style_str) = &request.style {
        state
            .settings
            .set("ai.writingStyle", style_str.clone().into())
            .map_err(|e| format!("Failed to set writing style: {}", e))?;
    }

    Ok(WritingStyleResult {
        style: request.style,
        error: None,
    })
}

#[command]
pub async fn ask_ai_streaming(
    state: State<'_, AppState>,
    context: AskAiContext,
) -> Result<String, String> {
    log::debug!(
        "Received ask_ai_streaming request with {} messages",
        context.history.len()
    );

    let ai_service = get_ai_service(&state);
    let app_handle = state.app_handle.clone();
    let request_id = uuid::Uuid::new_v4().to_string();
    let request_id_clone = request_id.clone();

    let request = AskAiRequest {
        history: context
            .history
            .into_iter()
            .map(|h| ChatMessage {
                role: h.role,
                content: h.content,
            })
            .collect(),
    };

    tokio::spawn(async move {
        match ai_service.ask_ai_streaming(request).await {
            Ok(mut rx) => {
                while let Some(chunk) = rx.recv().await {
                    let _ = app_handle
                        .emit(&format!("corvus:ask-ai-chunk-{}", request_id_clone), chunk);
                }
                let _ = app_handle.emit(
                    &format!("corvus:ask-ai-complete-{}", request_id_clone),
                    "done",
                );
            }
            Err(e) => {
                log::error!("ask_ai_streaming error: {}", e);
                let _ = app_handle.emit(&format!("corvus:ask-ai-error-{}", request_id_clone), e);
            }
        }
    });

    Ok(request_id)
}

#[command]
pub async fn generate_email_completion_streaming(
    state: State<'_, AppState>,
    context: EmailContextRequest,
) -> Result<String, String> {
    log::debug!("Received generate_email_completion_streaming request");

    let ai_service = get_ai_service(&state);
    let app_handle = state.app_handle.clone();
    let request_id = uuid::Uuid::new_v4().to_string();
    let request_id_clone = request_id.clone();

    let request = EmailCompletionRequest {
        metadata: EmailMetadata {
            sender: context.metadata.sender,
            subject: context.metadata.subject,
            is_reply: context.metadata.is_reply,
            recipients: context.metadata.recipients,
        },
        prior_email: context.prior_email,
        current_text: context.current_text,
        cursor_position: context.cursor_position,
    };

    tokio::spawn(async move {
        match ai_service
            .generate_email_completion_streaming(request)
            .await
        {
            Ok(mut rx) => {
                while let Some(chunk) = rx.recv().await {
                    let _ = app_handle.emit(
                        &format!("corvus:completion-chunk-{}", request_id_clone),
                        chunk,
                    );
                }
                let _ = app_handle.emit(
                    &format!("corvus:completion-complete-{}", request_id_clone),
                    "done",
                );
            }
            Err(e) => {
                log::error!("generate_email_completion_streaming error: {}", e);
                let _ =
                    app_handle.emit(&format!("corvus:completion-error-{}", request_id_clone), e);
            }
        }
    });

    Ok(request_id)
}

#[command]
pub async fn generate_subject_streaming(
    state: State<'_, AppState>,
    context: GenerateSubjectContextRequest,
) -> Result<String, String> {
    log::debug!("Received generate_subject_streaming request");

    let ai_service = get_ai_service(&state);
    let app_handle = state.app_handle.clone();
    let request_id = uuid::Uuid::new_v4().to_string();
    let request_id_clone = request_id.clone();

    let request = GenerateSubjectRequest {
        body_content: context.body_content,
        sender: context.sender,
        recipients: context.recipients,
        is_reply: context.is_reply,
        current_subject: context.current_subject,
    };

    tokio::spawn(async move {
        match ai_service.generate_subject_streaming(request).await {
            Ok(mut rx) => {
                while let Some(chunk) = rx.recv().await {
                    let _ = app_handle
                        .emit(&format!("corvus:subject-chunk-{}", request_id_clone), chunk);
                }
                let _ = app_handle.emit(
                    &format!("corvus:subject-complete-{}", request_id_clone),
                    "done",
                );
            }
            Err(e) => {
                log::error!("generate_subject_streaming error: {}", e);
                let _ = app_handle.emit(&format!("corvus:subject-error-{}", request_id_clone), e);
            }
        }
    });

    Ok(request_id)
}
