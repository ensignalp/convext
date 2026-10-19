use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum ConvextError {
    #[error("I/O operation failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration serialization or parsing error: {0}")]
    Config(String),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Archive processing error: {0}")]
    Archive(String),

    #[error("External tool '{binary}' is missing or failed: {reason}")]
    ExternalBinary { binary: String, reason: String },

    #[error("Media processing failed on '{path}': {details}")]
    MediaProcessing { path: PathBuf, details: String },

    #[error("PDF manipulation failed: {0}")]
    Pdf(String),

    #[error("Document conversion failed: {0}")]
    Document(String),

    #[error("QR code or metadata operation failed: {0}")]
    PrivacyQr(String),

    #[error("Desktop integration or D-Bus error: {0}")]
    Desktop(String),

    #[error("Single-instance lock error: {0}")]
    InstanceLock(String),

    #[error("Invalid input parameter: {0}")]
    InvalidInput(String),
}

pub type ConvextResult<T> = Result<T, ConvextError>;
