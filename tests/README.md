# Integration Tests

This directory contains comprehensive integration tests for the json-schema-to-luau converter, testing both the library API and CLI functionality.

## Test Files

### `test_schema.json`
A comprehensive JSON Schema that includes:
- **Basic types**: string, number, integer, boolean
- **Complex types**: object, array
- **Constraints**: minimum/maximum values, string length limits, array size limits
- **Validation**: email format, regex patterns
- **Enums**: string literal types
- **Required/optional fields**
- **Nested objects**
- **Descriptions and metadata**

### `expected_output.luau`
The expected Luau type definition output when converting `test_schema.json`. This serves as the reference for validating both library and CLI output.

### `integration_test.rs`
Comprehensive test suite that validates:

#### Library Tests
- **`test_library_convert_schema()`**: Tests the basic `convert_schema()` function
- **`test_library_convert_schema_with_custom_name()`**: Tests `convert_schema_with_name()` with custom type names
- **`test_library_invalid_schema()`**: Tests error handling for invalid schema types
- **`test_library_malformed_json()`**: Tests error handling for malformed JSON

#### CLI Tests
- **`test_cli_basic_conversion()`**: Tests basic CLI usage with input file
- **`test_cli_with_custom_type_name()`**: Tests CLI with `--type-name` flag
- **`test_cli_with_output_file()`**: Tests CLI with `--output` flag for file output

## Running Tests

```bash
# Run all tests
cargo test

# Run only integration tests
cargo test integration_test

# Run a specific test
cargo test test_library_convert_schema
```

## Test Coverage

The tests validate:
- ✅ **Library API**: Both `convert_schema()` and `convert_schema_with_name()` functions
- ✅ **CLI functionality**: Input files, output files, custom type names
- ✅ **Error handling**: Invalid schemas and malformed JSON
- ✅ **Output correctness**: Exact matching against expected Luau output
- ✅ **Type name conversion**: PascalCase conversion behavior

## Example Usage

The test schema demonstrates a real-world use case - a user profile with various data types and constraints that would be common in web applications. The generated Luau types include:

- Type safety for required vs optional fields
- String literal unions for enums
- Nested object types
- Array types with constraints
- Validation metadata as comments

This provides a comprehensive validation that the converter works correctly for both simple and complex JSON Schema structures.
