# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-05-02

### Fixed
- Fixed serialization issues with string and numeric values:
  - Numeric values are now properly serialized without quotes
  - String values with trailing commas now have the commas removed during serialization
  - The `requires_quotes` method now correctly determines when quotes are needed
- Fixed compatibility issues with the `from_file` function:
  - Updated documentation to clarify that two generic parameters are required for the function
  - Added example using `FromConf::from_file()` method directly as an alternative
- Added comprehensive test suite for serialization

## [0.1.0] - 2025-04-18

### Added
- Initial implementation of the Confetti configuration language parser
- Core parser functionality with lexer and AST builder
- Configuration mapping system for conversion between configuration files and Rust structs
- Derive macros for automatic implementation of FromConf and ToConf traits
- Support for various configuration syntax features:
  - Block directives
  - Quoted and triple-quoted strings
  - Line continuations
  - Comments (single-line and multi-line)
- Added comprehensive documentation and examples
- GitHub Actions workflow for continuous integration

### Changed
- Converted original code from C11 to pure Rust implementation
- Enhanced error handling with descriptive error messages
- Improved configuration options for parser customization

### Documentation
- Added detailed README with examples and API documentation
- Included examples for common use cases:
  - Configuration loading and serialization
  - File-based configuration
  - Custom type mapping 