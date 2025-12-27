use serde::{Deserialize, Serialize};

/// Domain-driven error classification for message processing.
/// 
/// This enforces explicit error routing:
/// - `Transient`: Temporary failures that should be retried (network issues, rate limits, etc.)
/// - `Permanent`: Fatal errors that should go directly to DLQ (validation failures, schema errors, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProcessingError {
    /// Transient error that should be retried.
    /// Examples: Network timeout, service unavailable, rate limiting
    Transient { reason: String },
    
    /// Permanent error that should go to DLQ immediately.
    /// Examples: Invalid schema, validation failure, unsupported version
    Permanent { reason: String },
}

impl ProcessingError {
    pub fn transient(reason: impl Into<String>) -> Self {
        Self::Transient {
            reason: reason.into(),
        }
    }

    pub fn permanent(reason: impl Into<String>) -> Self {
        Self::Permanent {
            reason: reason.into(),
        }
    }

    pub fn reason(&self) -> &str {
        match self {
            Self::Transient { reason } => reason,
            Self::Permanent { reason } => reason,
        }
    }

    pub fn is_transient(&self) -> bool {
        matches!(self, Self::Transient { .. })
    }

    pub fn is_permanent(&self) -> bool {
        matches!(self, Self::Permanent { .. })
    }

    pub fn error_type(&self) -> &str {
        match self {
            Self::Transient { .. } => "transient",
            Self::Permanent { .. } => "permanent",
        }
    }
}

impl std::fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transient { reason } => write!(f, "Transient error: {}", reason),
            Self::Permanent { reason } => write!(f, "Permanent error: {}", reason),
        }
    }
}

impl std::error::Error for ProcessingError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transient_error() {
        let err = ProcessingError::transient("Network timeout");
        assert!(err.is_transient());
        assert!(!err.is_permanent());
        assert_eq!(err.reason(), "Network timeout");
        assert_eq!(err.error_type(), "transient");
    }

    #[test]
    fn test_permanent_error() {
        let err = ProcessingError::permanent("Invalid schema");
        assert!(err.is_permanent());
        assert!(!err.is_transient());
        assert_eq!(err.reason(), "Invalid schema");
        assert_eq!(err.error_type(), "permanent");
    }
}
