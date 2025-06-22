//! Rust LLM library
//! 
//! This library provides a simple interface for interacting with LLM models.
//! 
//! There are two ways to use the library:
//! 
//! 1. Use the `DeepseekModel` struct to generate requests and responses directly
//! 2. Use the `ClientApi` trait to implement how to communicate with the model, 
//! and use the `Communicate` struct to generate the conversation.
//! 
//! Notice that our library is designed for concurrency, so all the functions are async.

mod config;
mod api;
mod error;

use config::{DEFAULT_MODEL, DEFAULT_MAX_TOKENS, DEFAULT_TEMPERATURE, BASE_DELAY, MAX_RETRIES};
pub use api::{Message, DeepseekModel, ModelApi, ModelRequest, ModelResponse};
pub use error::ApiError;
use std::future::Future;
use std::fmt::Display;

/// Client trait, used to implement how to communicate with the model
/// 
/// # example
/// 
/// ```
/// use rust_llm::{Message, ClientApi, ApiError};
/// 
/// struct MyClient;
/// 
/// impl ClientApi for MyClient {
///     fn response(&self, dialog: &[Message]) -> impl std::future::Future<Output = Result<Option<Message>, ApiError>> + Send {
///         async move {
///             if dialog.is_empty() {
///                 Ok(Some(Message { role: "user".to_string(), content: "Hello, how are you?".to_string() }))
///             } else if dialog.len() == 2 && dialog[0].role == "user" && dialog[1].role == "assistant" {
///                 Ok(None)
///             } else {
///                 Err(ApiError::Unrecoverable("Invalid dialog".to_string()))
///             }
///         }
///     }
/// }
/// ```
pub trait ClientApi: Send {
    fn response(&self, dialog: &[Message]) -> impl Future<Output = Result<Option<Message>, ApiError>> + Send;
}

/// Dialog history (display implemented)
#[derive(Debug, Clone)]
pub struct Conversation(Vec<Message>);

impl Display for Conversation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "Conversation:\n{}", 
            self.0.iter()
                .map(|m| format!("{}: {}", m.role, m.content))
                .collect::<Vec<String>>().join("\n")
        )?;
        Ok(())
    }
}

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

/// Communicate between Client and Model
/// 
/// # example
/// 
/// ```no_run
/// use rust_llm::{Message, ClientApi, ApiError, Communicate};
/// 
/// struct MyClient;
/// 
/// impl ClientApi for MyClient {
///     fn response(&self, dialog: &[Message]) -> impl std::future::Future<Output = Result<Option<Message>, ApiError>> + Send {
///         async move {
///             if dialog.is_empty() {
///                 Ok(Some(Message { role: "user".to_string(), content: "Hello, how are you?".to_string() }))
///             } else if dialog.len() == 2 && dialog[0].role == "user" && dialog[1].role == "assistant" {
///                 Ok(None)
///             } else {
///                 Err(ApiError::Unrecoverable("Invalid dialog".to_string()))
///             }
///         }
///     }
/// }
/// 
/// #[tokio::main]
/// async fn main() {
///     let client = MyClient;
/// 
///     let communicate = Communicate::communicate(client, "your_api_key".to_string(), None, None, None).await.unwrap();
/// 
///     println!("{}", communicate.get_messages());
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Communicate<T: ClientApi>{
    model: DeepseekModel,
    client: T,
    conversation: Conversation,
}

impl<T: ClientApi> Communicate<T> {
    pub fn communicate(client: T, api_key: String, model: Option<String>, max_tokens: Option<u32>, temperature: Option<f32>) 
    -> impl Future<Output = Result<Self, String>> + Send 
    where T: Send + 'static {
        async move {
            let mut communicate = Self {
                model: DeepseekModel::new(api_key)
                    .set_model(model.unwrap_or(DEFAULT_MODEL.to_string()))
                    .set_max_tokens(max_tokens.unwrap_or(DEFAULT_MAX_TOKENS))
                    .set_temperature(temperature.unwrap_or(DEFAULT_TEMPERATURE)),
                client: client,
                conversation: Conversation::new(),
            };

            'round_loop: loop {
                let mut retry_count = 0;
                let max_retries = MAX_RETRIES;
                let base_delay = BASE_DELAY;

                'retry_loop: loop {
                    let client_message = communicate.client.response(communicate.conversation.get_messages()).await;
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
                let model_message = communicate.model.chat(model_request).await;
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
            
            Ok(communicate)
        }
    }

    pub fn get_messages(&self) -> &Conversation {
        &self.conversation
    }
}