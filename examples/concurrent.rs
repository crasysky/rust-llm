use rust_llm::{Client, AsyncClientApi};
use futures::future::join_all;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let client = Client::new("deepseek-chat".to_string())
        .with_max_tokens(500)
        .with_temperature(0.7);

    let prompts = vec![
        "你好，请介绍一下你自己",
        "写一首关于夜晚星空的诗",
        "解释 Rust 的所有权机制",
    ];

    let start = Instant::now();
    let futures = prompts.into_iter().map(|p| client.send_message(p.to_string()));
    let results = join_all(futures).await;

    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(resp) => println!("请求 {}: {}", i, resp),
            Err(e) => eprintln!("请求 {} 失败: {}", i, e),
        }
    }
    println!("总耗时: {:.2?}", start.elapsed());
}