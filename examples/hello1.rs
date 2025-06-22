use rust_llm::{Message, DeepseekModel, ModelApi};

const MY_API_KEY: &str = "sk-b5b8c29284304fa6a1895b8257e5741f";

#[tokio::main]
async fn main() {
    let model = DeepseekModel::new(MY_API_KEY.to_string());
    let request_message = "Hello, how are you?";
    let request = model.generate_request(&[Message { role: "user".to_string(), content: request_message.to_string() }]);
    let response = model.chat(request).await.unwrap();

    println!("request: {}", request_message);
    println!("response: {}", response);
}