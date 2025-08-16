use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiteError {
    pub status: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub error_type: String,
}

impl fmt::Display for KiteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Kite API Error: {} ({})", self.message, self.error_type)
    }
}

impl std::error::Error for KiteError {}

#[derive(Debug)]
pub struct KiteConnectError {
    pub kind: KiteConnectErrorKind,
    pub backtrace: std::backtrace::Backtrace,
}

#[derive(Debug)]
pub enum KiteConnectErrorKind {
    ApiError(KiteError),
    HttpError(reqwest::Error),
    SerializationError(serde_json::Error),
    InvalidHeader(reqwest::header::InvalidHeaderValue),
    Other(String),
}

impl fmt::Display for KiteConnectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            KiteConnectErrorKind::ApiError(e) => write!(f, "{}", e),
            KiteConnectErrorKind::HttpError(e) => write!(f, "HTTP Error: {}", e),
            KiteConnectErrorKind::SerializationError(e) => write!(f, "Serialization Error: {}", e),
            KiteConnectErrorKind::InvalidHeader(e) => write!(f, "Invalid Header: {}", e),
            KiteConnectErrorKind::Other(e) => write!(f, "Error: {}", e),
        }
    }
}

impl std::error::Error for KiteConnectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            KiteConnectErrorKind::ApiError(e) => Some(e),
            KiteConnectErrorKind::HttpError(e) => Some(e),
            KiteConnectErrorKind::SerializationError(e) => Some(e),
            KiteConnectErrorKind::InvalidHeader(e) => Some(e),
            KiteConnectErrorKind::Other(_) => None,
        }
    }
}

impl KiteConnectError {
    /// Create a new error with the given kind
    pub fn new(kind: KiteConnectErrorKind) -> Self {
        KiteConnectError {
            kind,
            backtrace: std::backtrace::Backtrace::capture(),
        }
    }

    /// Create a new Other error with captured backtrace
    pub fn other(msg: impl Into<String>) -> Self {
        Self::new(KiteConnectErrorKind::Other(msg.into()))
    }

    /// Get the backtrace for this error
    pub fn backtrace(&self) -> &std::backtrace::Backtrace {
        &self.backtrace
    }

    pub fn print_backtrace(&self) {
        use std::backtrace::BacktraceStatus;

        match self.backtrace.status() {
            BacktraceStatus::Disabled | BacktraceStatus::Unsupported => {
                eprintln!("Backtrace disabled (run with RUST_BACKTRACE=1)");
                return;
            }
            BacktraceStatus::Captured => {}
            _ => return,
        }

        let trace = self.backtrace.to_string();
        let mut count = 0;

        for line in trace.lines() {
            // Print frame number and function
            if let Some(pos) = line.find(": ") {
                let frame_num = line[..pos].trim();
                let content = &line[pos + 2..];
                eprintln!("  {} → {}", frame_num, content);
                count += 1;
            } else if line.trim().starts_with("at ") {
                eprintln!("       {}", line.trim());
            }
        }

        if count == 0 {
            eprintln!("  (no frames to display)");
        }
    }
}

impl From<reqwest::Error> for KiteConnectError {
    fn from(error: reqwest::Error) -> Self {
        Self::new(KiteConnectErrorKind::HttpError(error))
    }
}

impl From<serde_json::Error> for KiteConnectError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(KiteConnectErrorKind::SerializationError(error))
    }
}

impl From<reqwest::header::InvalidHeaderValue> for KiteConnectError {
    fn from(error: reqwest::header::InvalidHeaderValue) -> Self {
        Self::new(KiteConnectErrorKind::InvalidHeader(error))
    }
}

impl From<KiteError> for KiteConnectError {
    fn from(error: KiteError) -> Self {
        Self::new(KiteConnectErrorKind::ApiError(error))
    }
}
