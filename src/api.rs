use reqwest::Client;
use serde::Serialize;
use crate::config::{get_api_key, MAX_RETRIES, BASE_DELAY};

/// 请求过程中的可能错误
#[derive(Debug, Clone)]
pub enum ApiError {
    NetworkError(String),
    ApiError(String),
    RateLimitError,
    OtherError(String),
}

/// LLM API 请求消息结构
#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// LLM API 请求体结构
#[derive(Debug, Clone, Serialize)]
pub struct ModelRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

pub type ModelResponse = String;

/// 抽象模型接口
pub trait ModelApi {
    #[allow(dead_code)]
    fn new() -> Self;
    fn chat(&self, request: ModelRequest) -> Result<ModelResponse, ApiError>;
}

/// Deepseek 模型实现
#[derive(Clone)]
pub struct DeepseekModel {
    http_client: Client,
    api_key: String,
    base_url: String,
}

impl DeepseekModel {
    /// 默认构造，生产环境使用，从环境变量读取 Key
    pub fn new() -> Self {
        let api_key = get_api_key();
        Self {
            http_client: Client::new(),
            api_key,
            base_url: "https://api.deepseek.com/v1/chat/completions".to_string(),
        }
    }
    /// 测试或自定义场景使用：注入 http_client、base_url、api_key
    pub fn with_base(client: Client, base_url: String, api_key: String) -> Self {
        Self {
            http_client: client,
            api_key,
            base_url,
        }
    }
}

impl ModelApi for DeepseekModel {
    fn new() -> Self {
        DeepseekModel::new()
    }

    fn chat(&self, request: ModelRequest) -> Result<ModelResponse, ApiError> {
        // 同步封装：block_on 在非 Tokio 运行时上下文使用
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
                            if retry_count < max_retries {
                                let delay = base_delay * 2_u32.pow(retry_count);
                                tokio::time::sleep(delay).await;
                                retry_count += 1;
                                continue;
                            } else {
                                return Err(ApiError::RateLimitError);
                            }
                        } else {
                            // 其他 HTTP 错误，返回 body 作为错误信息
                            let err_text = r.text().await.unwrap_or_default();
                            return Err(ApiError::ApiError(err_text));
                        }
                    }
                    Err(e) => {
                        // 网络错误重试
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
