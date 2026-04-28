#[derive(Debug)]
pub struct TTSError {
    pub message: String,
}

impl std::fmt::Display for TTSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TTSError {}

impl From<std::io::Error> for TTSError {
    fn from(e: std::io::Error) -> Self {
        TTSError {
            message: e.to_string(),
        }
    }
}

impl From<serde_json::Error> for TTSError {
    fn from(e: serde_json::Error) -> Self {
        TTSError {
            message: e.to_string(),
        }
    }
}

impl From<ort::Error> for TTSError {
    fn from(e: ort::Error) -> Self {
        TTSError {
            message: e.to_string(),
        }
    }
}

pub type TTSResult<T> = Result<T, TTSError>;
