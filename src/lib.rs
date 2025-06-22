mod config;
mod api;

use config::{DEFAULT_MODEL, DEFAULT_MAX_TOKENS, DEFAULT_TEMPERATURE, BASE_DELAY, MAX_RETRIES};
use api::{ModelApi, ApiError};
// use std::collections::VecDeque;

use api::{Message, DeepseekModel};

pub trait ClientApi {
    fn response(&self, dialog: &[Message]) -> Result<Option<Message>, ApiError>;
}

// 对话历史记录
#[derive(Debug, Clone)]
pub struct Conversation(Vec<Message>);

impl Conversation {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add_message(&mut self, message: Message) {
        self.0.push(message);
    }

    pub fn get_messages(&self) -> &[Message] {
        &self.0
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

// Communicate between Client and Model
#[derive(Debug, Clone)]
pub struct Communicate<T: ClientApi>{
    model: DeepseekModel,
    client: T,
    conversation: Conversation,
}

impl<T: ClientApi> Communicate<T> {
    pub fn communicate(api_key: String, client: T, model: Option<String>, max_tokens: Option<u32>, temperature: Option<f32>) 
    -> Result<Self, String> {
        let mut communicate = Self {
            model: DeepseekModel::new(api_key)
                .set_model(model.unwrap_or(DEFAULT_MODEL.to_string()))
                .set_max_tokens(max_tokens.unwrap_or(DEFAULT_MAX_TOKENS))
                .set_temperature(temperature.unwrap_or(DEFAULT_TEMPERATURE)),
            client: client,
            conversation: Conversation::new(),
        };

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            'round_loop: loop {
                let mut retry_count = 0;
                let max_retries = MAX_RETRIES;
                let base_delay = BASE_DELAY;

                'retry_loop: loop {
                    let client_message = communicate.client.response(communicate.conversation.get_messages());
                    match client_message {
                        Ok(Some(message)) => {
                            communicate.conversation.add_message(message);
                            break 'retry_loop;
                        }
                        Ok(None) => {
                            break 'round_loop;
                        }
                        Err(ApiError::Unrecoverable(e)) => {
                            return Err(format!("Client unrecoverable error: {}", e));
                        }
                        Err(ApiError::Recoverable(e)) => {
                            if retry_count < max_retries {
                                let delay = base_delay * 2_u32.pow(retry_count);
                                tokio::time::sleep(delay).await;
                                retry_count += 1;
                                continue;
                            }
                            return Err(format!("Client recoverable error after {} retries: {}", max_retries, e));
                        }
                    }
                }

                let model_request = communicate.model.generate_request(communicate.conversation.get_messages());
                let model_message = communicate.model.chat(model_request);
                match model_message {
                    Ok(message) => {
                        communicate.conversation.add_message(Message {
                            role: "assistant".to_string(),
                            content: message,
                        });
                    }
                    Err(e) => {
                        return Err(format!("Model error: {}", e));
                    }
                }
            }
            Ok(())
        })?;
        
        Ok(communicate)
    }
}