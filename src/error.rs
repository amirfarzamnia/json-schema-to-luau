use thiserror::Error;

pub type Result<T> = std::result::Result<T, ConversionError>;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Failed to parse JSON Schema: {0}")]
    ParseError(String),

    #[error("Unsupported schema type: {0}")]
    UnsupportedType(String),
}
