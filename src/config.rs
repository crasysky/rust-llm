use std::time::Duration;

/// 从环境变量读取 Deepseek API Key，环境变量名 DEESEEK_API_KEY
pub fn get_api_key() -> String {
    std::env::var("DEESEEK_API_KEY")
        .expect("请设置环境变量 DEESEEK_API_KEY 为你的 Deepseek API Key")
}
// pub const API_KEY: &str = "sk-b5b8c29284304fa6a1895b8257e5741f";

#[cfg(not(test))]
pub const MAX_RETRIES: u32 = 5;
#[cfg(test)]
pub const MAX_RETRIES: u32 = 2;

#[cfg(not(test))]
pub const BASE_DELAY: Duration = Duration::from_millis(500);
#[cfg(test)]
pub const BASE_DELAY: Duration = Duration::from_millis(10);

