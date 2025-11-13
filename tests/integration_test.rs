use json_schema_to_luau::{convert_schema, convert_schema_with_name};
use std::fs;
use std::path::Path;
use std::process::Command;

const TEST_SCHEMA_PATH: &str = "tests/test_schema.json";
const EXPECTED_OUTPUT_PATH: &str = "tests/expected_output.luau";

/// Test the library function convert_schema
#[test]
fn test_library_convert_schema() {
    // Read the test schema
    let schema_content =
        fs::read_to_string(TEST_SCHEMA_PATH).expect("Failed to read test schema file");

    // Read the expected output
    let expected_output =
        fs::read_to_string(EXPECTED_OUTPUT_PATH).expect("Failed to read expected output file");

    // Convert using the library function
    let result = convert_schema(&schema_content).expect("Failed to convert schema using library");

    // Compare the results
    assert_eq!(
        result.trim(),
        expected_output.trim(),
        "Library output doesn't match expected output"
    );
}

/// Test the library function convert_schema_with_name
#[test]
fn test_library_convert_schema_with_custom_name() {
    // Read the test schema
    let schema_content =
        fs::read_to_string(TEST_SCHEMA_PATH).expect("Failed to read test schema file");

    // Convert using the library function with custom name
    let result = convert_schema_with_name(&schema_content, "CustomName")
        .expect("Failed to convert schema with custom name");

    // The result should contain the custom type name (converted to PascalCase)
    assert!(
        result.contains("export type CustomName = {"),
        "Custom type name not found in output"
    );

    // Should still contain the same structure
    assert!(result.contains("email: string"));
    assert!(result.contains("isActive: boolean"));
    assert!(result.contains("status: \"pending\" | \"active\" | \"suspended\" | \"terminated\"?"));
}

/// Test the CLI by running the binary
#[test]
fn test_cli_basic_conversion() {
    // Build the project first to ensure the binary exists
    let build_output = Command::new("cargo")
        .args(&["build", "--bin", "json-schema-to-luau"])
        .output()
        .expect("Failed to build the project");

    assert!(
        build_output.status.success(),
        "Failed to build project: {}",
        String::from_utf8_lossy(&build_output.stderr)
    );

    // Run the CLI tool
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "json-schema-to-luau",
            "--",
            TEST_SCHEMA_PATH,
        ])
        .output()
        .expect("Failed to execute CLI command");

    assert!(
        output.status.success(),
        "CLI command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Read the expected output
    let expected_output =
        fs::read_to_string(EXPECTED_OUTPUT_PATH).expect("Failed to read expected output file");

    // Compare the CLI output with expected output
    let cli_output = String::from_utf8(output.stdout).expect("CLI output is not valid UTF-8");

    assert_eq!(
        cli_output.trim(),
        expected_output.trim(),
        "CLI output doesn't match expected output"
    );
}

/// Test the CLI with custom type name
#[test]
fn test_cli_with_custom_type_name() {
    // Run the CLI tool with custom type name
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "json-schema-to-luau",
            "--",
            TEST_SCHEMA_PATH,
            "--type-name",
            "CustomTypeName",
        ])
        .output()
        .expect("Failed to execute CLI command with custom type name");

    assert!(
        output.status.success(),
        "CLI command with custom type name failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let cli_output = String::from_utf8(output.stdout).expect("CLI output is not valid UTF-8");

    // The result should contain the custom type name (converted to PascalCase)
    assert!(
        cli_output.contains("export type CustomTypeName = {"),
        "Custom type name not found in CLI output"
    );
}

/// Test the CLI with output file
#[test]
fn test_cli_with_output_file() {
    let output_file = "tests/cli_output_test.luau";

    // Clean up any existing output file
    let _ = fs::remove_file(output_file);

    // Run the CLI tool with output file
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "json-schema-to-luau",
            "--",
            TEST_SCHEMA_PATH,
            "--output",
            output_file,
        ])
        .output()
        .expect("Failed to execute CLI command with output file");

    assert!(
        output.status.success(),
        "CLI command with output file failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check that the output file was created
    assert!(
        Path::new(output_file).exists(),
        "Output file was not created"
    );

    // Read the output file and compare with expected
    let file_content = fs::read_to_string(output_file).expect("Failed to read CLI output file");

    let expected_output =
        fs::read_to_string(EXPECTED_OUTPUT_PATH).expect("Failed to read expected output file");

    assert_eq!(
        file_content.trim(),
        expected_output.trim(),
        "CLI output file doesn't match expected output"
    );

    // Clean up
    let _ = fs::remove_file(output_file);
}

/// Test error handling for invalid JSON schema
#[test]
fn test_library_invalid_schema() {
    let invalid_schema = r#"{ "type": "invalid_type" }"#;

    let result = convert_schema(invalid_schema);
    assert!(result.is_err(), "Should return error for invalid schema");
}

/// Test error handling for malformed JSON
#[test]
fn test_library_malformed_json() {
    let malformed_json = r#"{ "type": "object", "properties": { "name": }"#;

    let result = convert_schema(malformed_json);
    assert!(result.is_err(), "Should return error for malformed JSON");
}
