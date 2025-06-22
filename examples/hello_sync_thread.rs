// examples/hello_sync_threads.rs
use rust_llm::{Client, ClientApi, Message};
use std::thread;
use std::time::Instant;
use std::sync::Arc;

fn main() {
    // 确保已设置环境变量 DEESEEK_API_KEY
    let client = Client::new("deepseek-chat".to_string())
        .with_max_tokens(500)
        .with_temperature(0.7);
    // Arc 包裹以便多线程共享
    let client = Arc::new(client);

    let prompts = vec![
        "你好，请介绍一下你自己",
        "写一首关于夜晚星空的诗",
        "解释 Rust 的所有权机制",
        "给我推荐几本好书",
        "讲个笑话",
    ];

    println!("启动同步多线程并发，共 {} 条请求", prompts.len());
    let start = Instant::now();

    let mut handles = vec![];
    for prompt in prompts {
        let client = Arc::clone(&client);
        let prompt_str = prompt.to_string();
        let handle = thread::spawn(move || {
            // 在同步线程中调用 client.response，会创建临时 runtime block_on
            let dialog = vec![ Message { role: "user".to_string(), content: prompt_str.clone() } ];
            match client.response(dialog) {
                Ok(resp) => println!("Prompt: '{}'\nResponse: {}\n", prompt_str, resp),
                Err(e) => eprintln!("Prompt '{}' 错误: {}", prompt_str, e),
            }
        });
        handles.push(handle);
    }
    // 等待所有线程完成
    for h in handles {
        let _ = h.join();
    }

    let duration = start.elapsed();
    println!("全部线程请求完成，用时: {:.2?}", duration);
}
