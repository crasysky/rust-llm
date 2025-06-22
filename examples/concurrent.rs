use rust_llm::{Client, AsyncClientApi, Message};
use futures::future::join_all;
use std::time::Instant;

#[tokio::main]
async fn main() {
    // 确保已设置环境变量 DEESEEK_API_KEY
    let client = Client::new("deepseek-chat".to_string())
        .with_max_tokens(500)
        .with_temperature(0.7);

    // 准备多个 prompt，比如从命令行参数、文件或硬编码
    let prompts = vec![
        "你好，请介绍一下你自己",
        "请用诗歌形式描述春天",
        "给我一个 Rust 并发示例的解释",
        "解释量子力学中的叠加原理",
        "写一段关于人工智能未来的短文",
    ];

    println!("准备并发发送 {} 个请求", prompts.len());
    let start = Instant::now();

    // 方法一：用 tokio::spawn
    
    let mut handles = Vec::with_capacity(prompts.len());
    for prompt in prompts.iter() {
        let client = client.clone();
        let prompt_str = prompt.to_string();
        let handle = tokio::spawn(async move {
            // send_message 内部会 spawn_blocking 执行同步 chat
            match client.send_message(prompt_str.clone()).await {
                Ok(resp) => {
                    println!("Prompt: '{}'\nResponse: {}\n", prompt_str, resp);
                    Some((prompt_str, resp))
                }
                Err(e) => {
                    eprintln!("Prompt '{}' 错误: {}", prompt_str, e);
                    None
                }
            }
        });
        handles.push(handle);
    }
    // 等待所有任务完成
    let results = join_all(handles).await;
    // results: Vec<Result<Option<(String, String)>, JoinError>>
    

    // 方法二：用 futures::future::join_all 直接收集 Future
    /* 
    let futures = prompts.iter().map(|prompt| {
        let client = client.clone();
        let prompt_str = prompt.to_string();
        async move {
            match client.send_message(prompt_str.clone()).await {
                Ok(resp) => {
                    println!("Prompt: '{}'\nResponse: {}\n", prompt_str, resp);
                    Some((prompt_str, resp))
                }
                Err(e) => {
                    eprintln!("Prompt '{}' 错误: {}", prompt_str, e);
                    None
                }
            }
        }
    });
    let results = join_all(futures).await;

    let duration = start.elapsed();
    println!("全部请求完成，用时: {:.2?}", duration);

    // 处理结果
    for res in results {
        if let Some((prompt, resp)) = res {
            // 这里已经在上面打印过
        }
    }
    */
}
