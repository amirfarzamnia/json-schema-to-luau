pub mod converter;
pub mod error;
pub mod schema;

pub use converter::SchemaConverter;
pub use error::{ConversionError, Result};
pub use schema::JsonSchema;

/// Convert a JSON Schema string to Luau type definitions
pub fn convert_schema(json_schema: &str) -> Result<String> {
    let schema: JsonSchema = serde_json::from_str(json_schema)
        .map_err(|e| ConversionError::ParseError(e.to_string()))?;

    let converter = SchemaConverter::new();
    converter.convert(&schema)
}

/// Convert a JSON Schema string to Luau with a custom type name
pub fn convert_schema_with_name(json_schema: &str, type_name: &str) -> Result<String> {
    let schema: JsonSchema = serde_json::from_str(json_schema)
        .map_err(|e| ConversionError::ParseError(e.to_string()))?;

    let converter = SchemaConverter::new();
    converter.convert_with_name(&schema, type_name)
}
