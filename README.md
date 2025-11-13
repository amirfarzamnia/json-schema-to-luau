# json-schema-to-luau

Convert JSON Schema to Luau type definitions with full support for constraints and advanced schema features.

## Features

- ✅ Full JSON Schema support (objects, arrays, primitives, enums, const)
- ✅ Handles `$ref`, `definitions`, and `$defs`
- ✅ Supports composition (`allOf`, `anyOf`, `oneOf`)
- ✅ Converts constraints to comments (number ranges, string patterns, array limits)
- ✅ Handles required/optional properties
- ✅ CLI and library usage
- ✅ Type-safe with proper error handling

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
json-schema-to-luau = "0.1"
```

Or install the CLI tool:

```bash
cargo install json-schema-to-luau
```

## Usage

### CLI

```bash
# Convert from file
json-schema-to-luau schema.json -o types.luau

# Convert from stdin
cat schema.json | json-schema-to-luau - -o types.luau

# Custom root type name
json-schema-to-luau schema.json --type-name MyType
```

### Library

```rust
use json_schema_to_luau::convert_schema;

fn main() {
    let json_schema = r#"{
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": {
                "type": "number",
                "minimum": 0,
                "maximum": 120
            }
        },
        "required": ["name"]
    }"#;

    let luau_types = convert_schema(json_schema).unwrap();
    println!("{}", luau_types);
}
```

Output:

```lua
export type Root = {
    name: string,
    -- minimum: 0, maximum: 120
    age?: number,
}
```

## Examples

### Object with Nested Types

JSON Schema:

```json
{
  "type": "object",
  "properties": {
    "user": {
      "type": "object",
      "properties": {
        "id": { "type": "integer" },
        "email": {
          "type": "string",
          "format": "email"
        }
      },
      "required": ["id"]
    }
  }
}
```

Luau Output:

```lua
export type Root = {
    user?: { id: number, email?: string },
}
```

### Arrays

JSON Schema:

```json
{
  "type": "array",
  "items": { "type": "string" },
  "minItems": 1,
  "maxItems": 10
}
```

Luau Output:

```lua
-- minItems: 1, maxItems: 10
export type Root = { string }
```

### Enums and Unions

JSON Schema:

```json
{
  "type": "string",
  "enum": ["red", "green", "blue"]
}
```

Luau Output:

```lua
export type Root = "red" | "green" | "blue"
```

### Using $ref and definitions

JSON Schema:

```json
{
  "type": "object",
  "properties": {
    "person": { "$ref": "#/definitions/Person" }
  },
  "definitions": {
    "Person": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "age": { "type": "number" }
      }
    }
  }
}
```

Luau Output:

```lua
export type Root = {
    person?: Person,
}

export type Person = {
    name?: string,
    age?: number,
}
```

### Number Constraints (as comments)

JSON Schema:

```json
{
  "type": "number",
  "minimum": 0,
  "maximum": 100,
  "multipleOf": 5
}
```

Luau Output:

```lua
-- minimum: 0, maximum: 100, multipleOf: 5
export type Root = number
```

### Composition Types

JSON Schema:

```json
{
  "anyOf": [{ "type": "string" }, { "type": "number" }]
}
```

Luau Output:

```lua
-- Union type (any of these types)
export type Root = string | number
```

## Unsupported Features

Since Luau has limited type system capabilities compared to JSON Schema, some features are converted to comments:

- Number ranges (`minimum`, `maximum`, `exclusiveMinimum`, `exclusiveMaximum`)
- String patterns and length constraints (`pattern`, `minLength`, `maxLength`)
- Array constraints (`minItems`, `maxItems`, `uniqueItems`)
- Object property counts (`minProperties`, `maxProperties`)
- Format specifications (`format`)

These constraints are preserved as comments in the generated Luau types for documentation purposes.

## License

MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
