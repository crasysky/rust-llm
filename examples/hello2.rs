use rust_llm::{Message, ClientApi, ApiError, Communicate};

struct MyClient;

impl ClientApi for MyClient {
    fn response(&self, dialog: &[Message]) -> impl std::future::Future<Output = Result<Option<Message>, ApiError>> + Send {
        async move {
            if dialog.is_empty() {
                Ok(Some(Message { role: "user".to_string(), content: "Hello, how are you?".to_string() }))
            } else if dialog.len() == 2 && dialog[0].role == "user" && dialog[1].role == "assistant" {
                Ok(None)
            } else {
                Err(ApiError::Unrecoverable("Invalid dialog".to_string()))
            }
        }
    }
}


#[tokio::main]
async fn main() {
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .expect("DEEPSEEK_API_KEY environment variable not set");

    let client = MyClient;

    // let communicate = Communicate::communicate(client, MY_API_KEY.to_string(), None, None, None).await.unwrap();
    let communicate = Communicate::communicate(client, Some(api_key), None, None, None)
         .await
         .unwrap();

    println!("{}", communicate.get_messages());
}

// Conversation:
// user: Hello, how are you?
// assistant: Hello! I'm just a computer program, so I don't have feelings, but I'm here and ready to help you with anything you need. 😊 How about you? How are you doing today?