use rust_llm::{Client, ClientApi, Message};

fn main() {
    // 确保已设置环境变量 DEESEEK_API_KEY
    let client = Client::new("deepseek-chat".to_string())
        .with_max_tokens(1000)
        .with_temperature(0.7);

    // 同步调用单条消息
    println!("=== 同步单条消息示例 ===");
    let dialog = vec![
        Message {
            role: "user".to_string(),
            content: "你好，请介绍一下你自己".to_string(),
        }
    ];
    match client.response(dialog) {
        Ok(response) => {
            // response 内部会打印 Raw response JSON
            println!("Parsed content: {}", response);
        }
        Err(e) => {
            eprintln!("同步调用错误: {}", e);
        }
    }

    // 同步对话
    println!("=== 同步对话示例 ===");
    let conversation = vec![
        Message {
            role: "user".to_string(),
            content: "你好".to_string(),
        },
        Message {
            role: "assistant".to_string(),
            content: "你好！我是DeepSeek，一个AI助手。".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "你能做什么？".to_string(),
        }
    ];
    match client.response(conversation) {
        Ok(response) => {
            println!("Parsed content: {}", response);
        }
        Err(e) => {
            eprintln!("同步对话错误: {}", e);
        }
    }
}
