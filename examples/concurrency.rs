use rust_llm::{Message, ApiError, ClientApi, Communicate};
use std::time::Instant;

// const MY_API_KEY: &str = "sk-b5b8c29284304fa6a1895b8257e5741f";

/// Simple client, used for concurrency example
struct SimpleClient {
    id: u32,
    prompts: Vec<String>,
}

impl SimpleClient {
    fn new(id: u32, prompts: Vec<String>) -> Self {
        Self { id, prompts }
    }
}

impl ClientApi for SimpleClient {
    fn response(&self, dialog: &[Message]) -> impl std::future::Future<Output = Result<Option<Message>, ApiError>> + Send {
        let id = self.id;
        let prompts = self.prompts.clone();
        
        async move {
            let round = dialog.len() / 2;
            if round >= prompts.len() {
                Ok(None)
            } else {
                Ok(Some(Message {
                    role: "user".to_string(),
                    content: format!("Client {}: {}", id, prompts[round]),
                }))
            }
        }
    }
}

/// Example: Concurrent multiple conversations
async fn concurrent_conversations() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .expect("DEEPSEEK_API_KEY environment variable not set");

    println!("=== Concurrent multiple conversations ===");
    let start = Instant::now();
    
    let clients = vec![
        SimpleClient::new(1, vec!["What is Rust?".to_string(), "How does async work?".to_string()]),
        SimpleClient::new(2, vec!["Explain concurrency".to_string(), "What are futures?".to_string()]),
        SimpleClient::new(3, vec!["Tell me about tokio".to_string()]),
    ];
    
    let tasks: Vec<_> = clients
        .into_iter()
        .map(|client| {
            let client_id = client.id;
            let api_key = api_key.clone();
            tokio::spawn(async move {
                let result = Communicate::communicate(
                    client, 
                    // MY_API_KEY.to_string(), 
                    Some(api_key), 
                    None, None, None
                ).await;
                
                match result {
                    Ok(communicate) => format!("Client {}: {} messages", client_id, communicate.get_messages().get_messages().len()),
                    Err(e) => format!("Client {} error: {}", client_id, e),
                }
            })
        })
        .collect();
    
    let results = futures::future::join_all(tasks).await;
    for result in results {
        println!("{}", result.unwrap());
    }
    
    println!("time: {:?}\n", start.elapsed());
    Ok(())
}


#[tokio::main]
async fn main() {
    concurrent_conversations().await.unwrap();
}

// === Concurrent multiple conversations ===
// Client 1: 4 messages
// Client 2: 4 messages
// Client 3: 2 messages
// time: 58.531016458s