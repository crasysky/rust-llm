use rust_llm::{Message, DeepseekModel, ModelApi};


#[tokio::main]
async fn main() {
    // let model = DeepseekModel::new(MY_API_KEY.to_string());
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .expect("DEEPSEEK_API_KEY environment variable not set");

    let model = DeepseekModel::new(api_key);
    
    let request_message = "Hello, how are you?";
    let request = model.generate_request(&[Message { role: "user".to_string(), content: request_message.to_string() }]);
    let response = model.chat(request).await.unwrap();

    println!("request: {}", request_message);
    println!("response: {}", response);
}

// request: Hello, how are you?
// response: Hello! I'm just a computer program, so I don't have feelings, but I'm here and ready to help you with anything you need. How about you? How are you doing today? 😊
