mod config;
mod api;

use api::*;
// use std::collections::VecDeque;

// 重新导出常用的类型
pub use api::{Message, ModelRequest, ApiError};

pub type ClientResponse = String;

pub trait ClientApi {
    fn response(&self, dialog: Vec<Message>) -> Result<ClientResponse, String>;
}

// pub trait AsyncClientApi {
//     async fn response_async(&self, dialog: Vec<Message>) -> Result<ClientResponse, String>;
// }

// 对话历史记录
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

// 主要的客户端结构体
pub struct Communicate;

impl Communicate {
//     pub fn new() -> Self {
//         Self {}
//     }

//     pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
//         self.max_tokens = Some(max_tokens);
//         self
//     }

//     pub fn with_temperature(mut self, temperature: f32) -> Self {
//         self.temperature = Some(temperature);
//         self
//     }

//     // 解析API响应
//     fn parse_response(&self, response: &str) -> Result<String, String> {
//         // 尝试解析JSON响应
//         match serde_json::from_str::<serde_json::Value>(response) {
//             Ok(json) => {
//                 // 提取choices[0].message.content
//                 if let Some(choices) = json.get("choices") {
//                     if let Some(choice) = choices[0].as_object() {
//                         if let Some(message) = choice.get("message") {
//                             if let Some(content) = message.get("content") {
//                                 if let Some(content_str) = content.as_str() {
//                                     return Ok(content_str.to_string());
//                                 }
//                             }
//                         }
//                     }
//                 }
//                 Err("无法解析响应内容".to_string())
//             }
//             Err(_) => {
//                 // 如果不是JSON，直接返回原始响应
//                 Ok(response.to_string())
//             }
//         }
//     }

//     // 异步发送消息
//     pub async fn send_message(&self, message: String) -> Result<String, String> {
//         let dialog = vec![
//             Message {
//                 role: "user".to_string(),
//                 content: message,
//             }
//         ];
//         self.response_async(dialog).await
//     }

//     // 异步发送对话
//     pub async fn send_conversation(&self, messages: Vec<Message>) -> Result<String, String> {
//         self.response_async(messages).await
//     }

//     // 构建请求
//     fn build_request(&self, messages: Vec<Message>) -> ModelRequest {
//         ModelRequest {
//             model: self.model_name.clone(),
//             messages,
//             max_tokens: self.max_tokens,
//             temperature: self.temperature,
//         }
//     }
// }

// impl ClientApi for Client {
//     fn response(&self, dialog: Vec<Message>) -> Result<ClientResponse, String> {
//         // 构建请求
//         let request = self.build_request(dialog);

//         // 调用API
//         match self.model.chat(request) {
//             Ok(response) => self.parse_response(&response),
//             Err(e) => Err(format!("API调用失败: {:?}", e)),
//         }
//     }
// }

// impl AsyncClientApi for Client {
//     async fn response_async(&self, dialog: Vec<Message>) -> Result<ClientResponse, String> {
//         // 使用tokio::task::spawn_blocking来异步执行同步代码
//         let request = self.build_request(dialog);
        
//         tokio::task::spawn_blocking(move || {
//             // 这里需要重新创建model实例，因为self不能传递
//             let model = DeepseekModel::new();
//             model.chat(request)
//         }).await.map_err(|e| format!("任务执行失败: {:?}", e))?
//         .map_err(|e| format!("API调用失败: {:?}", e))
//         .and_then(|response_text| {
//             // 解析响应
//             match serde_json::from_str::<serde_json::Value>(&response_text) {
//                 Ok(json) => {
//                     if let Some(choices) = json.get("choices") {
//                         if let Some(choice) = choices[0].as_object() {
//                             if let Some(message) = choice.get("message") {
//                                 if let Some(content) = message.get("content") {
//                                     if let Some(content_str) = content.as_str() {
//                                         return Ok(content_str.to_string());
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                     Err("无法解析响应内容".to_string())
//                 }
//                 Err(_) => Ok(response_text),
//             }
//         })
//     }
}

