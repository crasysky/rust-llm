use rust_llm::{Client, ClientApi, AsyncClientApi, Message};
use httpmock::MockServer;
use httpmock::Method::POST;
use serde_json::json;

#[tokio::test]
async fn test_concurrent_requests_integration() {
    let server = MockServer::start();
    let path = "/v1/chat/completions";
    let response_body = json!({
        "choices": [
            { "message": { "content": "Hello, I am DeepSeek." } }
        ]
    })
    .to_string();
    let m = server.mock(|when, then| {
        when.method(POST).path(path);
        then.status(200)
            .header("Content-Type", "application/json")
            .body(response_body.clone());
    });

    let base_url = format!("{}{}", server.url(""), path);
    let client = Client::new("deepseek-chat".to_string())
        .with_max_tokens(1000)
        .with_temperature(0.7)
        .with_base_url(base_url);

    let mut handles = vec![];
    for _ in 0..5 {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            let response = client.send_message("你好".to_string())
                .await
                .expect("Failed to get response");
            assert_eq!(response, "Hello, I am DeepSeek.");
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await.expect("Task failed");
    }
    assert!(m.hits() >= 5);
}

#[tokio::test]
async fn test_full_conversation_integration() {
    let server = MockServer::start();
    let path = "/v1/chat/completions";
    let response_body = json!({
        "choices": [
            { "message": { "content": "Hello, I can help you!" } }
        ]
    })
    .to_string();
    let m = server.mock(|when, then| {
        when.method(POST).path(path);
        then.status(200)
            .header("Content-Type", "application/json")
            .body(response_body.clone());
    });

    let base_url = format!("{}{}", server.url(""), path);
    let client = Client::new("deepseek-chat".to_string())
        .with_max_tokens(1000)
        .with_temperature(0.7)
        .with_base_url(base_url);

    let conversation = vec![
        Message { role: "user".to_string(), content: "你好".to_string() },
        Message { role: "assistant".to_string(), content: "你好！我是DeepSeek。".to_string() },
        Message { role: "user".to_string(), content: "你能做什么？".to_string() },
    ];
    let response = client.send_conversation(conversation)
        .await
        .expect("Failed to get response");
    assert_eq!(response, "Hello, I can help you!");
    assert!(m.hits() >= 1);
}
