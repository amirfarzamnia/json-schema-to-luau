# json-schema-to-luau

Convert JSON Schema to Luau type definitions with full support for constraints and advanced schema features.

## Features

- ✅ Full JSON Schema support (objects, arrays, primitives, enums, const)
- ✅ Handles `$ref`, `definitions`, and `$defs`
- ✅ Composition support (`allOf`, `anyOf`, `oneOf`)
- ✅ Constraints preserved as Luau comments (ranges, string limits, patterns, array bounds)
- ✅ Required/optional property handling
- ✅ CLI and library usage
- ✅ Type-safe conversion with clear errors

---

## Installation

```bash
cargo add json-schema-to-luau
```

CLI:

```bash
cargo install json-schema-to-luau
```

---

## Usage

### CLI

```bash
json-schema-to-luau schema.json -o types.luau
cat schema.json | json-schema-to-luau - -o types.luau
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
    --- @minimum 0
    --- @maximum 120
    age?: number,
}
```

---

# Luau Type Mapping

The converter follows Luau’s actual type model. This is important because Luau is **not TypeScript**, and JSON Schema cannot always be represented directly.

| JSON Schema             | Luau Output                                            |
| ----------------------- | ------------------------------------------------------ |
| `"string"`              | `string`                                               |
| `"number"`, `"integer"` | `number`                                               |
| `"boolean"`             | `boolean`                                              |
| `"null"`                | `nil`                                                  |
| `array`                 | `{ T }`                                                |
| object map              | `{ [string]: T }`                                      |
| enum (strings)          | `"a" \| "b"`                                           |
| enum (numbers)          | `number` (Luau cannot represent numeric literal types) |
| anyOf / oneOf           | union (`A \| B`)                                       |
| allOf                   | intersection (`A & B`) or merged object                |

---

# Composition Handling

### `anyOf`

Converted to union:

```lua
export type T = A | B
```

### `oneOf`

Also converted to a union (Luau cannot enforce exclusivity):

```lua
export type T = A | B
```

### `allOf`

Two behaviors:

1. **If the parent schema defines properties**, it keeps them and merges with its `allOf` members.
2. **Otherwise**, converted to an intersection:

```lua
export type T = A & B
```

---

# Definition Resolution

The converter recognizes:

- `#/definitions/Name`
- `#/$defs/Name`

Definitions are collected from the root schema if it’s an object. Each definition becomes:

```
export type PascalName = ...
```

`$ref` never inlines referenced types.

---

# Examples

### Objects

### Nested types

```lua
export type Root = {
    user: { email: string?, id: number }?,
}
```

### Arrays

```lua
--- @minItems 1
--- @maxItems 10
export type Root = { string }
```

### Enum

```lua
export type Root = "red" | "green" | "blue"
```

### `$ref` and definitions

```lua
export type Root = {
    person: Person?,
}

export type Person = {
    age: number?,
    name: string?,
}
```

### Number constraints

```lua
--- @minimum 0
--- @maximum 100
--- @multipleOf 5
export type Root = number
```

### Composition

```lua
--- Union type (any of these types)
export type Root = string | number
```

---

# Advanced Examples

### allOf merging

JSON Schema:

```json
{
  "type": "object",
  "properties": { "id": { "type": "number" } },
  "allOf": [
    { "type": "object", "properties": { "name": { "type": "string" } } }
  ]
}
```

Output:

```lua
export type Root = {
    id: number?,
    name: string?,
}
```

### Inline vs Exported Types

Properties become inline types unless they come from `$ref`. Example:

```lua
user: { id: number, name: string }?
```

Definitions always get a named export.

---

# Limitations

Luau has a simpler type system than JSON Schema, so some features degrade gracefully:

- Tuple schemas (`items: [A, B, C]`) → not supported
- `if` / `then` / `else` → ignored
- `dependencies`, `dependentSchemas`, `dependentRequired` → ignored
- `patternProperties` → ignored (falls back to additionalProperties)
- `propertyNames` → ignored
- No remote `$ref` resolution (only local fragments)
- Number literal enums collapse to `number`
- Regex patterns are preserved as comments only
- Exclusive constraints cannot be enforced in Luau, only documented

---

# Troubleshooting

### “Why is my enum turned into `number`?”

Luau does not support numeric literal types. JSON Schema numeric enums degrade to `number`.

### “Why does my object turn into `{ [string]: any }`?”

This happens when the schema does not declare properties, but allows objects.

### “Why isn’t my referenced type exported?”

Only root-level `definitions` / `$defs` are collected.

### “Why is a type inlined instead of exported?”

Only `$ref` produces named types. Everything else is intentionally inline.

---

# API Reference

### `convert_schema(&str) -> Result<String>`

Parses JSON Schema text and returns Luau types.

### `SchemaConverter`

```rust
let mut converter = SchemaConverter::new();
let luau = converter.convert(&schema)?;
let luau = converter.convert_with_name(&schema, "MyType")?;
```

Useful for advanced embedding or reusing definitions.

---

# Performance Notes

- The converter is designed for codegen, not high-frequency runtime use.
- Converting large schemas repeatedly may allocate many intermediate objects.
- `$ref` resolution is single-pass and local only.

---

# License

[MIT](LICENSE.md) License
