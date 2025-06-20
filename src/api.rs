use reqwest::Client;
use serde::Serialize;
use async_retry::{retry, strategies::ExponentialBackoff};
use std::time::Duration;

// error type of request
#[derive(Debug, Clone)]
pub enum ApiError {
    NetworkError(String),
    ApiError(String),
    TimeoutError,
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

// LLM model
pub struct LLMModel {
    http_client: Client,
    api_key: String,
    base_url: String,
}

pub trait ModelApi {
    // create a new model
    fn new() -> Self;

    // send a request and get a response
    fn chat(&self, request: ModelRequest) -> Result<ModelResponse, ApiError>;
}

pub type DeepSeekModel = LLMModel;

impl ModelApi for DeepSeekModel {
    fn new() -> Self {
        Self {
            http_client: Client::new(),
            api_key: "sk-b5b8c29284304fa6a1895b8257e5741f".to_string(),
            base_url: "https://api.deepseek.com/v1/chat/completions".to_string(),
        }
    }

    fn chat(&self, request: ModelRequest) -> Result<ModelResponse, ApiError> {
        todo!();
        /*
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let retry_strategy = ExponentialBackoff::from_millis(500).factor(2).max_delay(Duration::from_secs(30)).take(5);
            let client = &self.http_client;
            let url = &self.base_url;
            let api_key = &self.api_key;
            let req_body = &request;

            let result = retry(retry_strategy, || async {
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
                            let text = r.text().await.map_err(|e| async_retry::Error::Permanent(ApiError::OtherError(e.to_string())))?;
                            Ok(text)
                        } else if r.status().as_u16() == 429 {
                            // Rate limit
                            Err(async_retry::Error::Transient(ApiError::RateLimitError))
                        } else {
                            let err_text = r.text().await.unwrap_or_default();
                            Err(async_retry::Error::Permanent(ApiError::ApiError(err_text)))
                        }
                    }
                    Err(e) => Err(async_retry::Error::Transient(ApiError::NetworkError(e.to_string()))),
                }
            }).await;

            match result {
                Ok(text) => Ok(text),
                Err(e) => match e {
                    ApiError::RateLimitError => Err(ApiError::RateLimitError),
                    ApiError::NetworkError(msg) => Err(ApiError::NetworkError(msg)),
                    ApiError::ApiError(msg) => Err(ApiError::ApiError(msg)),
                    ApiError::TimeoutError => Err(ApiError::TimeoutError),
                    ApiError::OtherError(msg) => Err(ApiError::OtherError(msg)),
                },
            }
        })
        */
    }
}