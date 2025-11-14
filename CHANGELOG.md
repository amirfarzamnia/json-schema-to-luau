# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.2] - 2025-11-14

### Fixed

- Pure Luau compatibility: all generated `.luau` schema files now include a trailing `return {}` to satisfy the standalone Luau runtime’s requirement that modules return exactly one value.

## [1.0.1] - 2025-11-14

### Fixed

- Typo and formatting issues in the README (`README.md`).

## [1.0.0] - 2025-11-13

### Added

- Stable 1.0 release
- Enhanced type inference for complex nested schemas
- Improved handling of `$ref` resolution and circular references
- Extended CLI options (input/output path, stdout, formatting control)
- Better Luau code formatting and comment alignment
- Support for custom type name prefixes and namespace output
- Performance optimizations for large schemas
- Improved error messages and validation feedback
- Updated documentation and examples for 1.0 features

### Changed

- Refactored core converter for maintainability and extensibility
- Unified internal schema traversal logic
- Improved enum and const handling for mixed-type values
- CLI now exits with non-zero code on validation or conversion errors

### Fixed

- Corrected handling of nested `allOf` and `oneOf` compositions
- Fixed missing comments for constraints in nested definitions
- Resolved edge cases with empty `properties` or `items` arrays

## [0.1.0] - 2025-11-13

### Added

- Initial release
- Convert JSON Schema to Luau type definitions
- Support for basic types (string, number, integer, boolean, null, object, array)
- Support for object properties with required/optional fields
- Support for array types with items
- Support for enum types (string, number, boolean literals)
- Support for const values
- Support for $ref references to definitions
- Support for definitions and $defs
- Support for composition types (allOf, anyOf, oneOf)
- Support for nested objects and arrays
- Constraint documentation in comments:
  - Number constraints (minimum, maximum, exclusiveMinimum, exclusiveMaximum, multipleOf)
  - String constraints (minLength, maxLength, pattern, format)
  - Array constraints (minItems, maxItems, uniqueItems)
  - Object constraints (minProperties, maxProperties)
- Support for additionalProperties
- Support for pattern properties
- Support for descriptions as comments
- CLI tool for command-line conversion
- Library API for programmatic usage
- Comprehensive test suite
- Full documentation and examples

[Unreleased]: https://github.com/amirfarzamnia/json-schema-to-luau/compare/v1.0.2...HEAD
[1.0.2]: https://github.com/amirfarzamnia/json-schema-to-luau/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/amirfarzamnia/json-schema-to-luau/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/amirfarzamnia/json-schema-to-luau/compare/v0.1.0...v1.0.0
[0.1.0]: https://github.com/amirfarzamnia/json-schema-to-luau/releases/tag/v0.1.0
