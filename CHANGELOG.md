# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/amirfarzamnia/json-schema-to-luau/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/amirfarzamnia/json-schema-to-luau/releases/tag/v0.1.0
