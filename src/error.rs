use std::fmt::{self, Display};

/// Error type of request: including recoverable and unrecoverable errors
#[derive(Debug, Clone)]
pub enum ApiError {
    // recoverable error, should retry
    Recoverable(String),
    // unrecoverable error, should not retry
    Unrecoverable(String),
}

impl Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Recoverable(e) => write!(f, "(Recoverable error) {}", e),
            ApiError::Unrecoverable(e) => write!(f, "(Unrecoverable error) {}", e),
        }
    }
}