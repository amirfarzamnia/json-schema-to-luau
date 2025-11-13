use json_schema_to_luau::{convert_schema, convert_schema_with_name};

#[test]
fn test_simple_object() {
    let schema = r#"{
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "number" }
        },
        "required": ["name"]
    }"#;

    let result = convert_schema(schema).unwrap();
    assert!(result.contains("export type Root = {"));
    assert!(result.contains("name: string"));
    assert!(result.contains("age?: number"));
}

#[test]
fn test_array_type() {
    let schema = r#"{
        "type": "array",
        "items": { "type": "string" }
    }"#;

    let result = convert_schema(schema).unwrap();
    assert!(result.contains("export type Root = { string }"));
}

#[test]
fn test_enum() {
    let schema = r#"{
        "type": "string",
        "enum": ["red", "green", "blue"]
    }"#;

    let result = convert_schema(schema).unwrap();
    assert!(result.contains("\"red\" | \"green\" | \"blue\""));
}

#[test]
fn test_number_constraints() {
    let schema = r#"{
        "type": "number",
        "minimum": 0,
        "maximum": 100
    }"#;

    let result = convert_schema(schema).unwrap();
    assert!(result.contains("minimum: 0"));
    assert!(result.contains("maximum: 100"));
}

#[test]
fn test_ref_definition() {
    let schema = r#"{
        "type": "object",
        "properties": {
            "user": { "$ref": "#/definitions/User" }
        },
        "definitions": {
            "User": {
                "type": "object",
                "properties": {
                    "id": { "type": "integer" },
                    "name": { "type": "string" }
                }
            }
        }
    }"#;

    let result = convert_schema(schema).unwrap();
    assert!(result.contains("user?: User"));
    assert!(result.contains("export type User = {"));
    assert!(result.contains("id?: number"));
    assert!(result.contains("name?: string"));
}

#[test]
fn test_any_of_union() {
    let schema = r#"{
        "anyOf": [
            { "type": "string" },
            { "type": "number" }
        ]
    }"#;

    let result = convert_schema(schema).unwrap();
    assert!(result.contains("string | number"));
}

#[test]
fn test_nested_object() {
    let schema = r#"{
        "type": "object",
        "properties": {
            "address": {
                "type": "object",
                "properties": {
                    "street": { "type": "string" },
                    "city": { "type": "string" }
                }
            }
        }
    }"#;

    let result = convert_schema(schema).unwrap();
    assert!(result.contains("address?:"));
    assert!(result.contains("street?: string"));
    assert!(result.contains("city?: string"));
}

#[test]
fn test_custom_type_name() {
    let schema = r#"{
        "type": "object",
        "properties": {
            "value": { "type": "string" }
        }
    }"#;

    let result = convert_schema_with_name(schema, "CustomType").unwrap();
    assert!(result.contains("export type CustomType = {"));
}

#[test]
fn test_string_constraints() {
    let schema = r#"{
        "type": "string",
        "minLength": 5,
        "maxLength": 50,
        "pattern": "^[a-z]+$"
    }"#;

    let result = convert_schema(schema).unwrap();
    assert!(result.contains("minLength: 5"));
    assert!(result.contains("maxLength: 50"));
    assert!(result.contains("pattern: ^[a-z]+$"));
}

#[test]
fn test_const_value() {
    let schema = r#"{
        "const": "fixed-value"
    }"#;

    let result = convert_schema(schema).unwrap();
    assert!(result.contains("\"fixed-value\""));
}

#[test]
fn test_additional_properties() {
    let schema = r#"{
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        },
        "additionalProperties": { "type": "number" }
    }"#;

    let result = convert_schema(schema).unwrap();
    assert!(result.contains("name?: string"));
    assert!(result.contains("[string]: number"));
}

#[test]
fn test_array_with_constraints() {
    let schema = r#"{
        "type": "array",
        "items": { "type": "integer" },
        "minItems": 1,
        "maxItems": 10,
        "uniqueItems": true
    }"#;

    let result = convert_schema(schema).unwrap();
    assert!(result.contains("minItems: 1"));
    assert!(result.contains("maxItems: 10"));
    assert!(result.contains("uniqueItems: true"));
}

#[test]
fn test_all_of() {
    let schema = r#"{
        "allOf": [
            { "type": "object", "properties": { "a": { "type": "string" } } },
            { "type": "object", "properties": { "b": { "type": "number" } } }
        ]
    }"#;

    let result = convert_schema(schema).unwrap();
    assert!(result.contains("&"));
}

#[test]
fn test_description_as_comment() {
    let schema = r#"{
        "type": "object",
        "description": "User profile information",
        "properties": {
            "name": {
                "type": "string",
                "description": "Full name of the user"
            }
        }
    }"#;

    let result = convert_schema(schema).unwrap();
    assert!(result.contains("-- User profile information"));
    assert!(result.contains("-- Full name of the user"));
}
