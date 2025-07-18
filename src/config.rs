use std::time::Duration;

pub const MAX_RETRIES: u32 = 5;
pub const BASE_DELAY: Duration = Duration::from_millis(500);

pub const DEFAULT_MODEL: &str = "deepseek-chat";
pub const DEFAULT_MAX_TOKENS: u32 = 2048;
pub const DEFAULT_TEMPERATURE: f32 = 0.7;