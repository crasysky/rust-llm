use reqwest::Client;
use serde::{Serialize, Deserialize};
use std::future::Future;
use crate::config::{MAX_RETRIES, BASE_DELAY, DEFAULT_MODEL, DEFAULT_MAX_TOKENS, DEFAULT_TEMPERATURE};
use crate::error::ApiError;

/// LLM API request struct
#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Request struct of LLM API
#[derive(Debug, Clone, Serialize)]
pub struct ModelRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

/// API response structures
#[derive(Debug, Deserialize)]
struct ApiResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

/// Just a string (parse from json response)
pub type ModelResponse = String;

/// Can be implemented for different models (e.g. Deepseek, OpenAI, etc.)
pub trait ModelApi {
    /// create a new model
    fn new(api_key: impl Into<String>) -> Self; 

    /// set model
    fn set_model(self, model: String) -> Self;

    /// set max tokens
    fn set_max_tokens(self, max_tokens: u32) -> Self;

    /// set temperature
    fn set_temperature(self, temperature: f32) -> Self;

    /// generate a request
    fn generate_request(&self, messages: &[Message]) -> ModelRequest;

    /// send a request and get a response
    /// always return response or unrecoverable error
    /// this is an async function that should be awaited
    fn chat(&self, request: ModelRequest) -> impl Future<Output = Result<ModelResponse, ApiError>> + Send;
}

/// Deepseek LLM model
/// 
/// # example
/// 
/// ```no_run
/// use rust_llm::{Message, DeepseekModel};
/// 
/// #[tokio::main]
/// async fn main() {
///     let model = DeepseekModel::new("your_api_key".to_string());
/// 
///     let request = model.generate_request(&[Message { role: "user".to_string(), content: "Hello, how are you?".to_string() }]);
///     let response = model.chat(request).await.unwrap();
/// 
///     println!("response: {}", response);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct DeepseekModel {
    http_client: Client,
    api_key: String,
    base_url: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
}

impl ModelApi for DeepseekModel {
    // fn new(api_key: String) -> Self {
    fn new(api_key: impl Into<String>) -> Self {
        Self {
            http_client: Client::new(),
            // api_key: api_key.to_string(),
            api_key: api_key.into(),
            base_url: "https://api.deepseek.com/v1/chat/completions".to_string(),
            model: DEFAULT_MODEL.to_string(),
            max_tokens: DEFAULT_MAX_TOKENS,
            temperature: DEFAULT_TEMPERATURE,
        }
    }

    fn set_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    fn set_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    fn set_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    fn generate_request(&self, messages: &[Message]) -> ModelRequest {
        ModelRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            max_tokens: Some(self.max_tokens),
            temperature: Some(self.temperature),
        }
    }

    fn chat(&self, request: ModelRequest) -> impl Future<Output = Result<ModelResponse, ApiError>> + Send {
        let client = self.http_client.clone();
        let url = self.base_url.clone();
        let api_key = self.api_key.clone();
        
        async move {
            let req_body = &request;
            let mut retry_count = 0;
            let max_retries = MAX_RETRIES;
            let base_delay = BASE_DELAY;

            loop {
                let resp = client
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(req_body)
                    .send()
                    .await;

                match resp {
                    Ok(r) => {
                        if r.status().is_success() {
                            // Parse the JSON response and extract content
                            let text = r.text().await.map_err(|e| ApiError::Unrecoverable(e.to_string()))?;
                            let api_response: ApiResponse = serde_json::from_str(&text)
                                .map_err(|e| ApiError::Unrecoverable(format!("Failed to parse response: {}", e)))?;
                            
                            // Extract content from the first choice
                            if let Some(first_choice) = api_response.choices.first() {
                                return Ok(first_choice.message.content.clone());
                            } else {
                                return Err(ApiError::Unrecoverable("No choices in response".to_string()));
                            }
                        } else if r.status().as_u16() == 429 || r.status().as_u16() == 503 {
                            // 429: Rate limit, 503: Service unavailable
                            if retry_count < max_retries {
                                let delay = base_delay * 2_u32.pow(retry_count);
                                tokio::time::sleep(delay).await;
                                retry_count += 1;
                                continue;
                            } else {
                                return Err(ApiError::Unrecoverable(format!("Recoverable error after {} retries", max_retries)));
                            }
                        } else {
                            let err_text = r.text().await.unwrap_or_default();
                            return Err(ApiError::Unrecoverable(err_text));
                        }
                    }
                    Err(e) => {
                        // Network error
                        if retry_count < max_retries {
                            let delay = base_delay * 2_u32.pow(retry_count);
                            tokio::time::sleep(delay).await;
                            retry_count += 1;
                            continue;
                        } else {
                            return Err(ApiError::Unrecoverable(format!("Network error after {} retries: {}", max_retries, e)));
                        }
                    }
                }
            }
        }
    }
}