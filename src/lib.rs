mod config;
mod api;

use api::*;
use crate::config::get_api_key;
pub use api::{Message, ModelRequest, ApiError};

pub type ClientResponse = String;

pub trait ClientApi {
    fn response(&self, dialog: Vec<Message>) -> Result<ClientResponse, String>;
}

pub trait AsyncClientApi {
    fn send_message(&self, message: String) -> impl std::future::Future<Output = Result<ClientResponse, String>> + Send;
    fn send_conversation(&self, messages: Vec<Message>) -> impl std::future::Future<Output = Result<ClientResponse, String>> + Send;
}

#[derive(Debug, Clone)]
pub struct Conversation(Vec<Message>);

impl Conversation {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add_message(&mut self, role: String, content: String) {
        self.0.push(Message { role, content });
    }

    pub fn get_messages(&self) -> Vec<Message> {
        self.0.iter().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

#[derive(Clone)]
pub struct Client {
    model_name: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    model: DeepseekModel,
}

impl Client {
    pub fn new(model_name: String) -> Self {
        Self {
            model_name,
            max_tokens: None,
            temperature: None,
            model: DeepseekModel::new(),
        }
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    fn build_request(&self, messages: Vec<Message>) -> ModelRequest {
        ModelRequest {
            model: self.model_name.clone(),
            messages,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        }
    }

    fn parse_response(&self, response: &str) -> Result<String, String> {
        match serde_json::from_str::<serde_json::Value>(response) {
            Ok(json) => {
                if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                    if let Some(choice) = choices.get(0) {
                        if let Some(message) = choice.get("message") {
                            if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                                return Ok(content.to_string());
                            }
                        }
                    }
                }
                Err("无法解析响应内容".to_string())
            }
            Err(_) => Ok(response.to_string()),
        }
    }

    /// 测试时注入 base_url（通常指向 MockServer）
    pub fn with_base_url(mut self, base_url: String) -> Self {
        let http_client = reqwest::Client::new();
        let api_key = get_api_key();
        self.model = DeepseekModel::with_base(http_client, base_url, api_key);
        self
    }
}

impl ClientApi for Client {
    fn response(&self, dialog: Vec<Message>) -> Result<ClientResponse, String> {
        let request = self.build_request(dialog);
        match self.model.chat(request) {
            Ok(raw) => {
                // 打印原始 JSON
                println!("Raw response JSON: {}", raw);
                // 解析后返回内容
                self.parse_response(&raw).map_err(|e| e)
            }
            Err(e) => Err(format!("API调用失败: {:?}", e)),
        }
    }
}

impl AsyncClientApi for Client {
    async fn send_message(&self, message: String) -> Result<ClientResponse, String> {
        let dialog = vec![Message {
            role: "user".to_string(),
            content: message,
        }];
        let request = self.build_request(dialog);
        let raw = tokio::task::spawn_blocking({
            let model = self.model.clone();
            move || model.chat(request)
        })
        .await
        .map_err(|e| format!("任务执行失败: {:?}", e))?
        .map_err(|e| format!("API调用失败: {:?}", e))?;
        // 打印原始 JSON
        println!("Raw async response JSON: {}", raw);
        self.parse_response(&raw).map_err(|e| e)
    }

    async fn send_conversation(&self, messages: Vec<Message>) -> Result<ClientResponse, String> {
        let request = self.build_request(messages);
        let raw = tokio::task::spawn_blocking({
            let model = self.model.clone();
            move || model.chat(request)
        })
        .await
        .map_err(|e| format!("任务执行失败: {:?}", e))?
        .map_err(|e| format!("API调用失败: {:?}", e))?;
        println!("Raw async response JSON: {}", raw);
        self.parse_response(&raw).map_err(|e| e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::MockServer;
    use httpmock::Method::POST;
    use serde_json::json;

    // 同步测试：普通 #[test] 避免在 Tokio runtime 内 block_on
    #[test]
    fn test_sync_response() {
        let server = MockServer::start();
        let path = "/v1/chat/completions";
        let response_body = json!({
            "choices": [
                { "message": { "content": "Hello, I am DeepSeek." } }
            ]
        })
        .to_string();
        let m = server.mock(|when, then| {
            when.method(POST)
                .path(path)
                .header("Content-Type", "application/json");
            then.status(200)
                .header("Content-Type", "application/json")
                .body(response_body.clone());
        });

        let base_url = format!("{}{}", server.url(""), path);
        let client = Client::new("deepseek-chat".to_string())
            .with_max_tokens(1000)
            .with_temperature(0.7)
            .with_base_url(base_url);

        let dialog = vec![ Message { role: "user".into(), content: "你好".into() } ];
        let response = client.response(dialog).expect("Failed to get response");
        assert_eq!(response, "Hello, I am DeepSeek.");
        m.assert_hits(1);
    }

    #[test]
    fn test_sync_response_error() {
        let server = MockServer::start();
        let path = "/v1/chat/completions";
        let m = server.mock(|when, then| {
            when.method(POST).path(path);
            then.status(400).body("Bad request");
        });

        let base_url = format!("{}{}", server.url(""), path);
        let client = Client::new("deepseek-chat".to_string())
            .with_base_url(base_url);

        let dialog = vec![ Message { role: "user".into(), content: "你好".into() } ];
        let result = client.response(dialog);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API调用失败"));
        m.assert_hits(1);
    }

    #[tokio::test]
    async fn test_async_send_message() {
        let server = MockServer::start();
        let path = "/v1/chat/completions";
        let response_body = json!({
            "choices": [
                { "message": { "content": "Async reply" } }
            ]
        })
        .to_string();
        let m = server.mock(|when, then| {
            when.method(POST).path(path);
            then.status(200)
                .header("Content-Type", "application/json")
                .body(response_body.clone());
        });

        let base_url = format!("{}{}", server.url(""), path);
        let client = Client::new("deepseek-chat".to_string())
            .with_base_url(base_url);

        let resp = client.send_message("你好".to_string()).await.expect("Failed");
        assert_eq!(resp, "Async reply");
        m.assert_hits(1);
    }

    #[tokio::test]
    async fn test_retry_logic_exhausted() {
        let server = MockServer::start();
        let path = "/v1/chat/completions";
        let m = server.mock(|when, then| {
            when.method(POST).path(path);
            then.status(503).body("Service Unavailable");
        });

        let base_url = format!("{}{}", server.url(""), path);
        let client = Client::new("deepseek-chat".to_string())
            .with_base_url(base_url);

        let resp = client.send_message("触发重试".to_string()).await;
        assert!(resp.is_err());
        assert!(m.hits() >= 1);
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let server = MockServer::start();
        let path = "/v1/chat/completions";
        let response_body = json!({
            "choices": [
                { "message": { "content": "Concurrent reply" } }
            ]
        })
        .to_string();
        let m = server.mock(|when, then| {
            when.method(POST).path(path);
            then.status(200)
                .header("Content-Type", "application/json")
                .body(response_body.clone());
        });

        let base_url = format!("{}{}", server.url(""), path);
        let client = Client::new("deepseek-chat".to_string())
            .with_base_url(base_url);

        let mut handles = vec![];
        for _ in 0..5 {
            let client = client.clone();
            let handle = tokio::spawn(async move {
                let rep = client.send_message("并发测试".to_string()).await.expect("Failed");
                assert_eq!(rep, "Concurrent reply");
            });
            handles.push(handle);
        }
        for h in handles {
            h.await.expect("Task failed");
        }
        assert!(m.hits() >= 5);
    }
}
