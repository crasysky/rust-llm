# Rust LLM Client for Deepseek API

本项目提供一个用 Rust 实现的 Deepseek LLM 客户端库，支持同步和异步两种调用方式，内置重试、指数退避、并发支持等功能，方便在 Rust 程序中集成 Deepseek AI 服务。

## 目录结构
rust-llm/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── examples/
│ ├── hello.rs
│ ├── hello_sync.rs
│ ├── hello_sync_thread.rs
│ └── concurrent.rs
├── src/
│ ├── api.rs
│ ├── config.rs
│ └── lib.rs
└── test/
└── integration_test.rs


- `src/api.rs`：DeepseekModel 的实现，封装 HTTP 请求、重试逻辑、错误类型等。
- `src/config.rs`：配置管理，从环境变量读取 API Key (`DEESEEK_API_KEY`)，以及重试次数/延迟常量（测试模式下自动缩短）。
- `src/lib.rs`：对外提供的 `Client` 结构体，包含同步接口 `ClientApi::response` 和异步接口 `AsyncClientApi::send_message/send_conversation`，并打印原始 JSON、解析返回内容。
- `examples/`：多个示例程序：
  - `hello.rs`：异步示例，使用 `#[tokio::main]`，演示 `send_message`/`send_conversation` 用法。
  - `hello_sync.rs`：同步示例，普通 `fn main`，演示 `client.response(...)` 同步调用。
  - `hello_sync_thread.rs`：同步多线程示例，演示用 `std::thread::spawn` 并行多个同步调用。
  - `concurrent.rs`：异步并发示例，演示在 Tokio 运行时下并发多条 `send_message` 请求。
- `test/integration_test.rs`：集成测试示例，使用 `httpmock` 模拟 Deepseek API，测试并发、多次调用等场景。
- `README.md`：项目说明、使用文档等。

## 功能特性

- **同步 & 异步调用**  
  - 同步接口：`Client::response(dialog: Vec<Message>) -> Result<String, String>`  
    内部会启动临时 Tokio runtime 发起请求，适用于简单脚本或同步场景。  
  - 异步接口：`Client::send_message(msg: String) -> Future<Output=Result<String,String>>`、`Client::send_conversation(messages: Vec<Message>)`  
    适用于异步上下文，使用 Tokio runtime + spawn_blocking（或可改造为全异步），支持高并发调用。  
- **环境变量配置 API Key**  
  - 从环境变量 `DEESEEK_API_KEY` 读取 Deepseek API Key，不再硬编码，增强安全性。  
- **请求参数定制**  
  - 可通过 `.with_max_tokens(u32)`、`.with_temperature(f32)` 链式设置 max_tokens、temperature 等请求参数。  
- **重试与指数退避**  
  - 对 HTTP 429/503 及网络错误自动重试，重试次数和基准延迟在 `src/config.rs` 中配置，测试模式下自动缩短延迟。  
- **并发 & 多线程安全**  
  - `reqwest::Client` 本身 `Send + Sync`，可在多个 async 任务或线程中共享。  
  - 异步接口通过 `tokio::task::spawn_blocking` 将同步调用放入 blocking 池，避免阻塞主 async 任务。可以并发地调用 `send_message`。  
  - 同步示例通过 `std::thread::spawn` 并行调用 `client.response`。  
- **原始 JSON 打印 &解析**  
  - 在调用返回后，先 `println!("Raw response JSON: {}", raw)`，再解析提取 `choices[0].message.content` 并返回。便于调试和观察 Deepseek 实际返回结构。  
- **测试覆盖**  
  - 使用 `httpmock` 启动本地 MockServer，在单元/集成测试中注入自定义 `base_url`，模拟各种 HTTP 场景（成功、错误、重试耗尽、并发多次调用等），保证客户端逻辑正确。  

## 环境准备

1. **Rust 工具链**  
   - 安装 Rust（建议 stable 1.XX+），并确保 `cargo` 可用。  
2. **设置 Deepseek API Key**  
   - 在终端环境变量中设置：  
     ```bash
     export DEESEEK_API_KEY="你的真实 Deepseek API Key"
     ```  
   - Windows PowerShell：  
     ```powershell
     $env:DEESEEK_API_KEY="你的真实 Deepseek API Key"
     ```  
   - 项目运行时会通过 `std::env::var("DEESEEK_API_KEY")` 读取。

## 构建与测试

- **编译项目**  
  ```bash
  cargo build
  ```

- **运行所有测试**
  测试基于 `httpmock` 模拟 Deepseek API，放在 `src/lib.rs` 的 `#[cfg(test)] mod tests` 和 `tests/integration_test.rs` 中，自动启动本地 MockServer 并注入自定义 `base_url`。可通过以下命令运行。
  ```bash
  cargo test
  ```
  测试会覆盖同步接口、异步接口、错误处理、重试耗尽、并发调用等多种场景，无需真实 API Key。

## 使用示例
1. 异步示例 (`examples/hello.rs`)
   运行
    ```bash
    cargo run --example hello
    ```
2. 同步示例 (`examples/hello_sync.rs`)
   运行
    ```bash
    cargo run --example hello_sync
    ```
3. 同步多线程示例 (`examples/hello_sync_thread.rs`)
   运行
    ```bash
    cargo run --example hello_sync_thread
    ```
4. 异步并发示例 (`examples/concurrent.rs`)
   运行
    ```bash
    cargo run --example cargo run --example concurrent
    ```
   
## 配置与扩展

- API Key 管理：目前通过环境变量 `DEESEEK_API_KEY` 读取。如需更多配置方式，可在项目中集成 `dotenv`、`config` 等库，支持 `.env` 文件或其他配置源。
- 全异步改造：当前 `DeepseekModel.chat` 是同步封装，异步接口使用 `spawn_blocking`。若希望更高效，可将 `chat` 改成 `async fn chat_async(...)`，直接 `await reqwest::Client` 的异步请求，省去 blocking 池开销。
- 限流/排队：若 Deepseek API 有并发或速率限制，可在调用层使用 `tokio::sync::Semaphore`、令牌桶等策略，控制最大并发数或请求速率。
- 流式返回：若 Deepseek 支持流式返回，可在 `DeepseekModel` 中对流式响应进行处理，并在 `Client` 中提供 `Stream` 接口或回调，以增量方式消费内容。
- 日志与监控：可用 `tracing`、`log` 等库记录请求、响应、重试、错误等信息；集成指标（如 Prometheus）监控请求数量、延迟、错误率。
- 命令行工具或 HTTP 服务：基于本库构建 CLI（如用 `clap`）或 HTTP 微服务（如用 `axum`、`warp`、`actix-web`），为其他应用提供 Deepseek API 代理或中间层。
- 多后端支持：若将来要支持多个 LLM 后端，可定义抽象 trait（如已有的`ModelApi/chat_async`），实现不同后端（OpenAI、Anthropic、其他内部模型），根据配置选择。
- 缓存与去重：对相似请求或近期请求结果做缓存，减少重复调用，提升效率。

