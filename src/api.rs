use reqwest::Client;
use serde::Serialize;
use crate::config::{API_KEY, MAX_RETRIES, BASE_DELAY};

// error type of request
#[derive(Debug, Clone)]
pub enum ApiError {
    NetworkError(String),
    ApiError(String),
    RateLimitError,
    OtherError(String),
}

// LLM API request struct
#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

// request struct of LLM API
#[derive(Debug, Clone, Serialize)]
pub struct ModelRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

pub type ModelResponse = String;

pub trait ModelApi {
    // create a new model
    fn new() -> Self;

    // send a request and get a response
    fn chat(&self, request: ModelRequest) -> Result<ModelResponse, ApiError>;
}

// LLM model
pub struct DeepseekModel {
    http_client: Client,
    api_key: String,
    base_url: String,
}

impl ModelApi for DeepseekModel {
    fn new() -> Self {
        Self {
            http_client: Client::new(),
            api_key: API_KEY.to_string(),
            base_url: "https://api.deepseek.com/v1/chat/completions".to_string(),
        }
    }

    fn chat(&self, request: ModelRequest) -> Result<ModelResponse, ApiError> {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let client = &self.http_client;
            let url = &self.base_url;
            let api_key = &self.api_key;
            let req_body = &request;

            let mut retry_count = 0;
            let max_retries = MAX_RETRIES;
            let base_delay = BASE_DELAY;

            loop {
                let resp = client
                    .post(url)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(req_body)
                    .send()
                    .await;

                match resp {
                    Ok(r) => {
                        if r.status().is_success() {
                            let text = r.text().await.map_err(|e| ApiError::OtherError(e.to_string()))?;
                            return Ok(text);
                        } else if r.status().as_u16() == 429 || r.status().as_u16() == 503 {
                            // 429: Rate limit
                            // 503: Service unavailable
                            if retry_count < max_retries {
                                let delay = base_delay * 2_u32.pow(retry_count);
                                tokio::time::sleep(delay).await;
                                retry_count += 1;
                                continue;
                            } else {
                                return Err(ApiError::RateLimitError);
                            }
                        } else {
                            let err_text = r.text().await.unwrap_or_default();
                            return Err(ApiError::ApiError(err_text));
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
                            return Err(ApiError::NetworkError(e.to_string()));
                        }
                    }
                }
            }
        })
    }
}