use rust_llm::{Client, ClientApi, AsyncClientApi, Message};

#[tokio::main]
async fn main() {
    // 创建一个Client实例
    let client = Client::new("deepseek-chat".to_string())
        .with_max_tokens(1000)
        .with_temperature(0.7);

    // 同步方式调用
    let dialog = vec![
        Message {
            role: "user".to_string(),
            content: "你好，请介绍一下你自己".to_string(),
        }
    ];

    match client.response(dialog) {
        Ok(response) => {
            println!("同步调用 - AI回复: {}", response);
        }
        Err(e) => {
            println!("同步调用错误: {}", e);
        }
    }

    // 异步方式调用
    match client.send_message("你好，请介绍一下你自己".to_string()).await {
        Ok(response) => {
            println!("异步调用 - AI回复: {}", response);
        }
        Err(e) => {
            println!("异步调用错误: {}", e);
        }
    }

    // 异步发送完整对话
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

    match client.send_conversation(conversation).await {
        Ok(response) => {
            println!("对话 - AI回复: {}", response);
        }
        Err(e) => {
            println!("对话错误: {}", e);
        }
    }
}
