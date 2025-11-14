use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum JsonSchema {
    Boolean(bool),
    Object(Box<SchemaObject>),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SchemaObject {
    // JSON Schema metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$schema")]
    pub schema: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$id")]
    pub id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub type_: Option<SchemaType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, JsonSchema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<JsonSchema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "enum")]
    pub enum_: Option<Vec<serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub const_: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<JsonSchema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<JsonSchema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<JsonSchema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<Box<JsonSchema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$ref")]
    pub ref_: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub definitions: Option<HashMap<String, JsonSchema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$defs")]
    pub defs: Option<HashMap<String, JsonSchema>>,

    // Number constraints (not directly supported in Luau, will use comments)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f64>,

    // String constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    // Array constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,

    // Object constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<AdditionalProperties>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern_properties: Option<HashMap<String, JsonSchema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    // Additional JSON Schema Draft-07 fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub contains: Option<Box<JsonSchema>>,

    // Conditional schemas
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "if")]
    pub if_: Option<Box<JsonSchema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "then")]
    pub then_: Option<Box<JsonSchema>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "else")]
    pub else_: Option<Box<JsonSchema>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AdditionalProperties {
    Boolean(bool),
    Schema(Box<JsonSchema>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SchemaType {
    Single(SingleType),
    Multiple(Vec<SingleType>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SingleType {
    Null,
    Boolean,
    Object,
    Array,
    Number,
    String,
    Integer,
}
