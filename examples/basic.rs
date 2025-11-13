use json_schema_to_luau::convert_schema;

fn main() {
    // Example 1: Simple object
    let schema1 = r#"{
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "number", "minimum": 0, "maximum": 120 },
            "email": { "type": "string", "format": "email" }
        },
        "required": ["name", "email"]
    }"#;

    println!("=== Example 1: Simple Object ===");
    println!("{}\n", convert_schema(schema1).unwrap());

    // Example 2: Array
    let schema2 = r#"{
        "type": "array",
        "items": {
            "type": "object",
            "properties": {
                "id": { "type": "integer" },
                "title": { "type": "string" }
            }
        },
        "minItems": 1,
        "maxItems": 100
    }"#;

    println!("=== Example 2: Array ===");
    println!("{}\n", convert_schema(schema2).unwrap());

    // Example 3: Enum
    let schema3 = r#"{
        "type": "string",
        "enum": ["active", "inactive", "pending"]
    }"#;

    println!("=== Example 3: Enum ===");
    println!("{}\n", convert_schema(schema3).unwrap());

    // Example 4: References
    let schema4 = r##"{
        "type": "object",
        "properties": {
            "user": { "$ref": "#/definitions/User" },
            "posts": {
                "type": "array",
                "items": { "$ref": "#/definitions/Post" }
            }
        },
        "definitions": {
            "User": {
                "type": "object",
                "properties": {
                    "id": { "type": "integer" },
                    "username": { "type": "string" }
                },
                "required": ["id", "username"]
            },
            "Post": {
                "type": "object",
                "properties": {
                    "id": { "type": "integer" },
                    "content": { "type": "string" },
                    "authorId": { "type": "integer" }
                }
            }
        }
    }"##;

    println!("=== Example 4: References ===");
    println!("{}\n", convert_schema(schema4).unwrap());

    // Example 5: Union types (anyOf)
    let schema5 = r#"{
        "anyOf": [
            { "type": "string" },
            { "type": "number" },
            { "type": "boolean" }
        ]
    }"#;

    println!("=== Example 5: Union Types ===");
    println!("{}\n", convert_schema(schema5).unwrap());

    // Example 6: Complex nested object
    let schema6 = r#"{
        "type": "object",
        "properties": {
            "config": {
                "type": "object",
                "properties": {
                    "theme": {
                        "type": "string",
                        "enum": ["light", "dark", "auto"]
                    },
                    "notifications": {
                        "type": "object",
                        "properties": {
                            "email": { "type": "boolean" },
                            "push": { "type": "boolean" }
                        }
                    }
                }
            }
        }
    }"#;

    println!("=== Example 6: Complex Nested Object ===");
    println!("{}\n", convert_schema(schema6).unwrap());
}
