use thiserror::Error;

pub type Result<T> = std::result::Result<T, ConversionError>;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Failed to parse JSON Schema: {0}")]
    ParseError(String),

    #[error("Unsupported schema type: {0}")]
    UnsupportedType(String),

    #[error("Invalid schema: {0}")]
    InvalidSchema(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
