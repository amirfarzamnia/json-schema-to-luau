# JSON Schema to Luau

[<img alt="Crates.io" src="https://img.shields.io/crates/v/json-schema-to-luau?style=flat-square&logo=rust" height="20">](https://crates.io/crates/json-schema-to-luau)
[<img alt="docs.rs" src="https://img.shields.io/docsrs/json-schema-to-luau?style=flat-square&logo=docs.rs" height="20">](https://docs.rs/json-schema-to-luau)

**Convert JSON Schema to Luau type definitions with full support for constraints and advanced schema features.**

---

## ✨ Features

- ✅ **Full JSON Schema support** (objects, arrays, primitives, enums, const)
- ✅ Handles **`$ref`, `definitions`, and `$defs`**
- ✅ Composition support (`allOf`, `anyOf`, `oneOf`)
- ✅ **Constraints preserved** as Luau comments (ranges, string limits, patterns, array bounds)
- ✅ Required/optional property handling
- ✅ CLI and library usage
- ✅ Type-safe conversion with clear errors

---

## ⬇️ Installation

You can install `json-schema-to-luau` via **Cargo** or download a pre-built binary from **GitHub Releases**.

### Cargo (Library)

To use as a Rust library in your project:

```bash
cargo add json-schema-to-luau
```

### Cargo (CLI)

To install the command-line interface globally:

```bash
cargo install json-schema-to-luau
```

### GitHub Releases (CLI Binaries)

Pre-compiled binaries for various platforms (Linux, macOS, Windows) are available on the [**GitHub Releases page**](https://github.com/amirfarzamnia/json-schema-to-luau/releases). This is the fastest way to get the CLI without needing a Rust toolchain.

---

## 🚀 Usage

### Command Line Interface (CLI)

```bash
# Convert a file and output to another file
json-schema-to-luau schema.json -o types.luau

# Read schema from standard input
cat schema.json | json-schema-to-luau - -o types.luau

# Specify a custom type name (defaults to 'Root')
json-schema-to-luau schema.json --type-name MyCustomType
```

### Rust Library

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

**Output:**

```lua
export type Root = {
    --- @minimum 0
    --- @maximum 120
    age: number?,
    name: string,
}
```

---

## 🛠️ Luau Type Mapping & Behavior

The converter maps JSON Schema concepts to the closest viable Luau types. This is crucial as **Luau is not TypeScript** and has different type system limitations.

### Primitive Mapping

| JSON Schema             | Luau Output  | Notes                                       |
| :---------------------- | :----------- | :------------------------------------------ |
| `"string"`              | `string`     |                                             |
| `"number"`, `"integer"` | `number`     |                                             |
| `"boolean"`             | `boolean`    |                                             |
| `"null"`                | `nil`        | Often combined: `string \| nil`             |
| enum (strings)          | `"a" \| "b"` | Uses union of literal strings               |
| enum (numbers)          | `number`     | Luau cannot represent numeric literal types |

### Complex Type Mapping

| JSON Schema       | Luau Output                             | Description                                                       |
| :---------------- | :-------------------------------------- | :---------------------------------------------------------------- |
| `array`           | `{ T }`                                 | General array type                                                |
| object map        | `{ [string]: T }`                       | Objects with `additionalProperties` or no `properties`            |
| `anyOf` / `oneOf` | union (`A \| B`)                        | `oneOf` exclusivity cannot be enforced in Luau                    |
| `allOf`           | intersection (`A & B`) or merged object | Intersection for standalone `allOf`, otherwise merged into parent |
| `$ref`, `$defs`   | `export type Name = ...`                | Always exports named types; never inlines                         |

---

## 🧩 Composition Handling

### `anyOf` / `oneOf` (Union)

Both are converted to a Luau union type, as Luau does not enforce the exclusivity of `oneOf`.

```lua
export type T = A | B
```

### `allOf` (Intersection / Merging)

1.  **If the parent schema defines properties**, `allOf` members are **merged** into the parent object type.
2.  **Otherwise**, it is converted to a Luau intersection: `export type T = A & B`.

---

## 📝 Examples

### Object with Constraints

```lua
export type Root = {
    --- @minimum 0
    --- @maximum 100
    age: number?,
    name: string,
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

### Definitions (`$ref` / `$defs`)

```lua
export type Root = {
    person: Person?,
}

export type Person = {
    age: number?,
    name: string?,
}
```

_Note: Referenced types like `Person` are always exported as named types._

---

## 🛑 Limitations

Luau has a simpler type system than JSON Schema. The following features degrade gracefully (i.e., they are ignored or simplified):

- **Tuple schemas** (`items: [A, B, C]`) → _Not supported_.
- **Conditionals** (`if` / `then` / `else`) → _Ignored_.
- **Dependencies** (`dependencies`, `dependentSchemas`, `dependentRequired`) → _Ignored_.
- **Pattern matching** (`patternProperties`, `propertyNames`) → _Ignored/Simplified_.
- **Remote `$ref` resolution** → Only local fragments (`#/...`) are supported.
- **Number literal enums** → Collapse to `number`.
- **Exclusive constraints** → Cannot be enforced, only documented via comments.

---

## 💡 Troubleshooting & FAQ

#### “Why is my numeric enum turned into `number`?”

Luau does not support numeric literal types (e.g., `1 | 2 | 3`). Numeric enums from JSON Schema must degrade to the base type `number`.

#### “Why does my object turn into `{ [string]: any }`?”

This typically happens when the schema is an object that allows arbitrary properties but does not explicitly declare any of its own (`properties` is absent or empty, and `additionalProperties` is the default `true`).

#### “Why is a type inlined instead of exported?”

Only types resolved via a `$ref` to a root-level definition (`#/definitions/Name` or `#/$defs/Name`) are exported as named types. All other complex types (like nested objects or arrays) are intentionally inlined for conciseness.

---

## 📦 API Reference (Rust)

### `convert_schema(&str) -> Result<String>`

The simplest function. Parses the JSON Schema string and returns the resulting Luau type definitions.

### `SchemaConverter`

For advanced usage (e.g., reusing definitions across multiple calls):

```rust
let mut converter = SchemaConverter::new();
let luau = converter.convert(&schema)?;
let luau = converter.convert_with_name(&schema, "MyType")?;
```

---

## ⚡ Performance Notes

- The converter is designed for **codegeneration**, not high-frequency runtime use.
- `$ref` resolution is single-pass and only supports local fragments.

---

## 📄 License

[MIT](https://www.google.com/search?q=LICENSE) License
