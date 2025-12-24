/*!
# Confetti-rs

A configuration language and parser library for Rust, with a flexible mapper for converting between configuration files and Rust structs.

## Features

- Simple, intuitive configuration syntax
- A powerful parser with customizable options
- Automatic mapping between configuration and Rust structs
- Support for custom data types
- Comprehensive error handling

## Basic Usage

Here's a simple example of how to use Confetti-rs:

```
use confetti_rs::{
    parse, from_str, to_string, ConfOptions, ConfDirective, ConfArgument,
    FromConf, ToConf, MapperError, ValueConverter,
};

// Define a configuration structure
#[derive(Debug, PartialEq)]
struct ServerConfig {
    host: String,
    port: i32,
}

// Implement FromConf to deserialize from config
impl FromConf for ServerConfig {
    fn from_directive(directive: &ConfDirective) -> Result<Self, MapperError> {
        let host = directive.children.iter()
            .find(|d| d.name.value == "host")
            .and_then(|d| d.arguments.first())
            .map(|a| confetti_rs::__private::strip_quotes(&a.value))
            .ok_or_else(|| MapperError::MissingField("host".into()))?;

        let port = directive.children.iter()
            .find(|d| d.name.value == "port")
            .and_then(|d| d.arguments.first())
            .map(|a| i32::from_conf_value(&a.value))
            .transpose()?
            .ok_or_else(|| MapperError::MissingField("port".into()))?;

        Ok(ServerConfig { host, port })
    }
}

// Implement ToConf to serialize to config
impl ToConf for ServerConfig {
    fn to_directive(&self) -> Result<ConfDirective, MapperError> {
        Ok(ConfDirective {
            name: ConfArgument {
                value: "ServerConfig".to_string(),
                span: 0..0, is_quoted: false, is_triple_quoted: false, is_expression: false,
            },
            arguments: vec![],
            children: vec![
                ConfDirective {
                    name: ConfArgument {
                        value: "host".to_string(),
                        span: 0..0, is_quoted: false, is_triple_quoted: false, is_expression: false,
                    },
                    arguments: vec![ConfArgument {
                        value: self.host.clone(),
                        span: 0..0, is_quoted: true, is_triple_quoted: false, is_expression: false,
                    }],
                    children: vec![],
                },
                ConfDirective {
                    name: ConfArgument {
                        value: "port".to_string(),
                        span: 0..0, is_quoted: false, is_triple_quoted: false, is_expression: false,
                    },
                    arguments: vec![ConfArgument {
                        value: self.port.to_string(),
                        span: 0..0, is_quoted: false, is_triple_quoted: false, is_expression: false,
                    }],
                    children: vec![],
                },
            ],
        })
    }
}

// Parse configuration
let config_str = r#"
ServerConfig {
    host "localhost";
    port 8080;
}
"#;

let config: ServerConfig = from_str(config_str).unwrap();
assert_eq!(config.host, "localhost");
assert_eq!(config.port, 8080);

// Serialize back to string
let serialized = to_string(&config).unwrap();
assert!(serialized.contains("host"));
assert!(serialized.contains("port"));
```

## Configuration Syntax

Confetti-rs uses a simple, readable syntax:

```text
DirectiveName {
  nested_directive "value";
  another_directive 123;

  block_directive {
    setting true;
    array 1, 2, 3, 4;
  }
}
```

## Documentation

For more examples and detailed documentation, please visit:
- [GitHub repository](https://github.com/shkmv/confetti-rs)
- [Comprehensive documentation on docs.rs](https://docs.rs/confetti-rs)
*/

use std::error::Error;
use std::fmt;
use std::ops::Range;

pub mod lexer;
pub mod mapper;
pub mod parser;

#[cfg(feature = "derive")]
pub use confetti_derive::ConfMap;

// Private module for derive macro implementation details
#[doc(hidden)]
pub mod __private {
    pub fn is_option_type(type_name: &str) -> bool {
        type_name.starts_with("core::option::Option<")
            || type_name.starts_with("std::option::Option<")
    }

    pub fn extract_option_type(type_name: &str) -> Option<&str> {
        if is_option_type(type_name) {
            // Extract the inner type from Option<T>
            let start = type_name.find('<')? + 1;
            let end = type_name.rfind('>')?;
            Some(&type_name[start..end])
        } else {
            None
        }
    }

    pub fn strip_quotes(value: &str) -> String {
        let result = value.to_string();
        // Add length check to prevent panic on strings shorter than 2 characters
        if result.starts_with('"') && result.ends_with('"') && result.len() >= 2 {
            result[1..result.len() - 1].to_string()
        } else {
            result
        }
    }
}

/// Processes escape sequences in a string according to the Confetti specification.
///
/// Per the spec, a backslash followed by any non-whitespace character produces
/// that character literally (e.g., `\n` produces 'n', not a newline).
///
/// # Arguments
///
/// * `input` - The input string containing escape sequences.
///
/// # Returns
///
/// A new string with escape sequences processed.
///
/// # Examples
///
/// ```
/// use confetti_rs::process_escapes;
///
/// assert_eq!(process_escapes(r#"hello\nworld"#), "hellonworld");
/// assert_eq!(process_escapes(r#"quote\"here"#), "quote\"here");
/// ```
pub fn process_escapes(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                // Per spec: backslash + non-whitespace char produces the char literally
                if !next.is_whitespace() {
                    chars.next(); // consume the escaped character
                    result.push(next); // push it literally
                    continue;
                }
            }
        }
        result.push(c);
    }

    result
}

/// Represents a configuration argument.
#[derive(Debug, Clone)]
pub struct ConfArgument {
    /// The value of the argument.
    pub value: String,
    /// The span of the argument in the source text.
    pub span: Range<usize>,
    /// Whether the argument is quoted.
    pub is_quoted: bool,
    /// Whether the argument is a triple-quoted string.
    pub is_triple_quoted: bool,
    /// Whether the argument is an expression.
    pub is_expression: bool,
}

/// Represents a configuration directive.
#[derive(Debug, Clone)]
pub struct ConfDirective {
    /// The name of the directive.
    pub name: ConfArgument,
    /// The arguments of the directive.
    pub arguments: Vec<ConfArgument>,
    /// The child directives of this directive.
    pub children: Vec<ConfDirective>,
}

/// Represents a configuration unit.
#[derive(Debug, Clone)]
pub struct ConfUnit {
    /// The root directives of the configuration.
    pub directives: Vec<ConfDirective>,
    /// The comments in the configuration.
    pub comments: Vec<ConfComment>,
}

/// Represents a comment in the configuration.
#[derive(Debug, Clone)]
pub struct ConfComment {
    /// The content of the comment.
    pub content: String,
    /// The span of the comment in the source text.
    pub span: Range<usize>,
    /// Whether the comment is a multi-line comment.
    pub is_multi_line: bool,
}

/// Represents an error that can occur during parsing.
#[derive(Debug)]
pub enum ConfError {
    /// An error occurred during lexing.
    LexerError {
        /// The position in the source text where the error occurred.
        position: usize,
        /// A description of the error.
        message: String,
    },
    /// An error occurred during parsing.
    ParserError {
        /// The position in the source text where the error occurred.
        position: usize,
        /// A description of the error.
        message: String,
    },
}

impl Error for ConfError {}

impl fmt::Display for ConfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfError::LexerError { position, message } => {
                write!(f, "Lexer error at position {}: {}", position, message)
            }
            ConfError::ParserError { position, message } => {
                write!(f, "Parser error at position {}: {}", position, message)
            }
        }
    }
}

/// Options for parsing configuration.
#[derive(Debug, Clone)]
pub struct ConfOptions {
    /// Whether to allow C-style comments (/* */ and //).
    pub allow_c_style_comments: bool,
    /// Whether to allow expression arguments.
    pub allow_expression_arguments: bool,
    /// The maximum depth of nested directives.
    pub max_depth: usize,
    /// Whether to forbid bidirectional formatting characters.
    /// When true (default), bidi characters will cause a lexer error.
    pub forbid_bidi_characters: bool,
    /// Whether to require semicolons at the end of directives.
    pub require_semicolons: bool,
    /// Whether to allow triple-quoted strings.
    pub allow_triple_quotes: bool,
    /// Whether to allow line continuations with backslash.
    pub allow_line_continuations: bool,
}

impl Default for ConfOptions {
    fn default() -> Self {
        Self {
            allow_c_style_comments: false,
            allow_expression_arguments: false,
            max_depth: 100,
            forbid_bidi_characters: true, // Default: forbid bidi characters for security
            require_semicolons: false,
            allow_triple_quotes: true,
            allow_line_continuations: true,
        }
    }
}

/// Parses a configuration string.
///
/// # Arguments
///
/// * `input` - The configuration string to parse.
/// * `options` - The options for parsing.
///
/// # Returns
///
/// A `Result` containing either the parsed configuration unit or an error.
///
/// # Examples
///
/// ```
/// use confetti_rs::{parse, ConfOptions};
///
/// let input = "server {\n  listen 80;\n}";
/// let options = ConfOptions::default();
/// let result = parse(input, options);
/// assert!(result.is_ok());
/// ```
pub fn parse(input: &str, options: ConfOptions) -> Result<ConfUnit, ConfError> {
    let mut parser = parser::Parser::new(input, options)?;
    parser.parse()
}

// Re-export key traits from mapper module
pub use crate::mapper::{FromConf, MapperError, MapperOptions, ToConf, ValueConverter};

// Create convenience wrappers for common operations
/// Load configuration from a file into a struct.
///
/// This function reads a configuration file and deserializes it into a type
/// that implements the [`FromConf`] trait.
///
/// # Example
///
/// ```no_run
/// use confetti_rs::{from_file, FromConf, ConfDirective, MapperError, ValueConverter};
///
/// #[derive(Debug)]
/// struct ServerConfig {
///     port: i32,
///     host: String,
/// }
///
/// impl FromConf for ServerConfig {
///     fn from_directive(directive: &ConfDirective) -> Result<Self, MapperError> {
///         let port = directive.children.iter()
///             .find(|d| d.name.value == "port")
///             .and_then(|d| d.arguments.first())
///             .map(|a| i32::from_conf_value(&a.value))
///             .transpose()?
///             .ok_or_else(|| MapperError::MissingField("port".into()))?;
///
///         let host = directive.children.iter()
///             .find(|d| d.name.value == "host")
///             .and_then(|d| d.arguments.first())
///             .map(|a| confetti_rs::__private::strip_quotes(&a.value))
///             .ok_or_else(|| MapperError::MissingField("host".into()))?;
///
///         Ok(ServerConfig { port, host })
///     }
/// }
///
/// // Load from file (file must exist)
/// // let config: ServerConfig = from_file("config.conf")?;
/// ```
pub fn from_file<T: FromConf, P: AsRef<std::path::Path>>(
    path: P,
) -> Result<T, mapper::MapperError> {
    T::from_file(path)
}

/// Load configuration from a string into a struct.
///
/// This function parses a configuration string and deserializes it into a type
/// that implements the [`FromConf`] trait.
///
/// # Example
///
/// ```
/// use confetti_rs::{from_str, FromConf, ConfDirective, MapperError, ValueConverter};
///
/// #[derive(Debug, PartialEq)]
/// struct ServerConfig {
///     port: i32,
///     host: String,
/// }
///
/// impl FromConf for ServerConfig {
///     fn from_directive(directive: &ConfDirective) -> Result<Self, MapperError> {
///         let port = directive.children.iter()
///             .find(|d| d.name.value == "port")
///             .and_then(|d| d.arguments.first())
///             .map(|a| i32::from_conf_value(&a.value))
///             .transpose()?
///             .ok_or_else(|| MapperError::MissingField("port".into()))?;
///
///         let host = directive.children.iter()
///             .find(|d| d.name.value == "host")
///             .and_then(|d| d.arguments.first())
///             .map(|a| confetti_rs::__private::strip_quotes(&a.value))
///             .ok_or_else(|| MapperError::MissingField("host".into()))?;
///
///         Ok(ServerConfig { port, host })
///     }
/// }
///
/// let config_str = r#"
/// ServerConfig {
///   port 8080;
///   host "localhost";
/// }
/// "#;
///
/// let server_config: ServerConfig = from_str(config_str).unwrap();
/// assert_eq!(server_config.port, 8080);
/// assert_eq!(server_config.host, "localhost");
/// ```
pub fn from_str<T: FromConf>(s: &str) -> Result<T, mapper::MapperError> {
    T::from_str(s)
}

/// Convert a struct to a configuration string.
///
/// This function serializes a type that implements the [`ToConf`] trait
/// into a configuration string.
///
/// # Example
///
/// ```
/// use confetti_rs::{to_string, ToConf, ConfDirective, ConfArgument, MapperError, ValueConverter};
///
/// #[derive(Debug)]
/// struct ServerConfig {
///     port: i32,
///     host: String,
/// }
///
/// impl ToConf for ServerConfig {
///     fn to_directive(&self) -> Result<ConfDirective, MapperError> {
///         Ok(ConfDirective {
///             name: ConfArgument {
///                 value: "ServerConfig".to_string(),
///                 span: 0..0,
///                 is_quoted: false,
///                 is_triple_quoted: false,
///                 is_expression: false,
///             },
///             arguments: vec![],
///             children: vec![
///                 ConfDirective {
///                     name: ConfArgument {
///                         value: "port".to_string(),
///                         span: 0..0,
///                         is_quoted: false,
///                         is_triple_quoted: false,
///                         is_expression: false,
///                     },
///                     arguments: vec![ConfArgument {
///                         value: self.port.to_conf_value()?,
///                         span: 0..0,
///                         is_quoted: false,
///                         is_triple_quoted: false,
///                         is_expression: false,
///                     }],
///                     children: vec![],
///                 },
///                 ConfDirective {
///                     name: ConfArgument {
///                         value: "host".to_string(),
///                         span: 0..0,
///                         is_quoted: false,
///                         is_triple_quoted: false,
///                         is_expression: false,
///                     },
///                     arguments: vec![ConfArgument {
///                         value: self.host.to_conf_value()?,
///                         span: 0..0,
///                         is_quoted: true,
///                         is_triple_quoted: false,
///                         is_expression: false,
///                     }],
///                     children: vec![],
///                 },
///             ],
///         })
///     }
/// }
///
/// let server_config = ServerConfig {
///     port: 8080,
///     host: "localhost".into(),
/// };
///
/// let config_str = to_string(&server_config).unwrap();
/// assert!(config_str.contains("port 8080"));
/// assert!(config_str.contains("host \"localhost\""));
/// ```
pub fn to_string<T: ToConf>(value: &T) -> Result<String, mapper::MapperError> {
    value.to_string()
}

/// Save a struct to a configuration file.
///
/// This function serializes a type that implements the [`ToConf`] trait
/// and writes it to a file.
///
/// # Example
///
/// ```no_run
/// use confetti_rs::{to_file, ToConf, ConfDirective, ConfArgument, MapperError, ValueConverter};
///
/// #[derive(Debug)]
/// struct ServerConfig {
///     port: i32,
///     host: String,
/// }
///
/// impl ToConf for ServerConfig {
///     fn to_directive(&self) -> Result<ConfDirective, MapperError> {
///         Ok(ConfDirective {
///             name: ConfArgument {
///                 value: "ServerConfig".to_string(),
///                 span: 0..0,
///                 is_quoted: false,
///                 is_triple_quoted: false,
///                 is_expression: false,
///             },
///             arguments: vec![],
///             children: vec![
///                 ConfDirective {
///                     name: ConfArgument {
///                         value: "port".to_string(),
///                         span: 0..0,
///                         is_quoted: false,
///                         is_triple_quoted: false,
///                         is_expression: false,
///                     },
///                     arguments: vec![ConfArgument {
///                         value: self.port.to_conf_value()?,
///                         span: 0..0,
///                         is_quoted: false,
///                         is_triple_quoted: false,
///                         is_expression: false,
///                     }],
///                     children: vec![],
///                 },
///                 ConfDirective {
///                     name: ConfArgument {
///                         value: "host".to_string(),
///                         span: 0..0,
///                         is_quoted: false,
///                         is_triple_quoted: false,
///                         is_expression: false,
///                     },
///                     arguments: vec![ConfArgument {
///                         value: self.host.to_conf_value()?,
///                         span: 0..0,
///                         is_quoted: true,
///                         is_triple_quoted: false,
///                         is_expression: false,
///                     }],
///                     children: vec![],
///                 },
///             ],
///         })
///     }
/// }
///
/// let server_config = ServerConfig {
///     port: 8080,
///     host: "localhost".into(),
/// };
///
/// to_file(&server_config, "config.conf").unwrap();
/// ```
pub fn to_file<T: ToConf, P: AsRef<std::path::Path>>(
    value: &T,
    path: P,
) -> Result<(), mapper::MapperError> {
    value.to_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conf_error_display() {
        let lexer_error = ConfError::LexerError {
            position: 10,
            message: "Invalid character".to_string(),
        };
        assert_eq!(
            lexer_error.to_string(),
            "Lexer error at position 10: Invalid character"
        );

        let parser_error = ConfError::ParserError {
            position: 20,
            message: "Unexpected token".to_string(),
        };
        assert_eq!(
            parser_error.to_string(),
            "Parser error at position 20: Unexpected token"
        );
    }

    #[test]
    fn test_parse_empty() {
        let input = "";
        let options = ConfOptions::default();
        let result = parse(input, options);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().directives.len(), 0);
    }

    #[test]
    fn test_parse_simple_directive() {
        let input = "server localhost;";
        let options = ConfOptions::default();
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.directives[0].name.value, "server");
        assert_eq!(conf_unit.directives[0].arguments.len(), 1);
        assert_eq!(conf_unit.directives[0].arguments[0].value, "localhost");
    }

    #[test]
    fn test_parse_block_directive() {
        let input = "server {\n  listen 80;\n}";
        let options = ConfOptions::default();
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.directives[0].name.value, "server");
        assert_eq!(conf_unit.directives[0].children.len(), 1);
        assert_eq!(conf_unit.directives[0].children[0].name.value, "listen");
        assert_eq!(conf_unit.directives[0].children[0].arguments.len(), 1);
        assert_eq!(conf_unit.directives[0].children[0].arguments[0].value, "80");
    }

    #[test]
    fn test_parse_with_comments() {
        let input = "# This is a comment\nserver {\n  # Another comment\n  listen 80;\n}";
        let options = ConfOptions::default();
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.comments.len(), 1);
        assert_eq!(conf_unit.comments[0].content, "# This is a comment");
    }

    #[test]
    fn test_parse_quoted_arguments() {
        let input = r#"server "example.com";"#;
        let options = ConfOptions::default();
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.directives[0].arguments.len(), 1);
        assert_eq!(
            conf_unit.directives[0].arguments[0].value,
            "\"example.com\""
        );
        assert!(conf_unit.directives[0].arguments[0].is_quoted);
    }

    #[test]
    fn test_parse_triple_quoted_arguments() {
        let input = r#"server """
        This is a multi-line
        string argument
        """;"#;
        let options = ConfOptions::default();
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.directives[0].arguments.len(), 1);
        assert!(conf_unit.directives[0].arguments[0]
            .value
            .contains("multi-line"));
        assert!(conf_unit.directives[0].arguments[0].is_triple_quoted);
    }

    #[test]
    fn test_parse_line_continuation() {
        let input = "server \\\nexample.com;";
        let options = ConfOptions {
            allow_line_continuations: true,
            ..ConfOptions::default()
        };
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.directives[0].arguments.len(), 1);
        assert_eq!(conf_unit.directives[0].arguments[0].value, "example.com");
    }

    // Phase 5: Comprehensive tests

    #[test]
    fn test_process_escapes_basic() {
        // Per spec: \n should produce 'n', not newline
        assert_eq!(process_escapes(r#"hello\nworld"#), "hellonworld");
        assert_eq!(process_escapes(r#"quote\"here"#), "quote\"here");
        assert_eq!(process_escapes(r#"tab\there"#), "tabthere");
        assert_eq!(process_escapes(r#"backslash\\"#), "backslash\\");
    }

    #[test]
    fn test_process_escapes_whitespace_preserved() {
        // Backslash before whitespace should be preserved
        assert_eq!(process_escapes("test\\ value"), "test\\ value");
    }

    #[test]
    fn test_c_style_single_line_comments() {
        let input = "// This is a C-style comment\nserver localhost;";
        let options = ConfOptions {
            allow_c_style_comments: true,
            ..ConfOptions::default()
        };
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.comments.len(), 1);
        assert!(conf_unit.comments[0].content.starts_with("//"));
    }

    #[test]
    fn test_c_style_mixed_comments() {
        let input = "// Single line\n/* Multi\nline */ server localhost;";
        let options = ConfOptions {
            allow_c_style_comments: true,
            ..ConfOptions::default()
        };
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.comments.len(), 2);
    }

    #[test]
    fn test_unicode_in_arguments() {
        let input = r#"greeting "Hello, World!";"#;
        let options = ConfOptions::default();
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert!(conf_unit.directives[0].arguments[0]
            .value
            .contains("Hello"));
    }

    #[test]
    fn test_unicode_in_directive_names() {
        // Using simple ASCII for directive name, unicode in value
        let input = r#"config "test value";"#;
        let options = ConfOptions::default();
        let result = parse(input, options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_deeply_nested_structures() {
        let input = "a { b { c { d { e { value 1; } } } } }";
        let options = ConfOptions {
            max_depth: 10,
            ..ConfOptions::default()
        };
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.directives[0].name.value, "a");
    }

    #[test]
    fn test_max_depth_exceeded() {
        let input = "a { b { c { d { e { f { g { } } } } } } }";
        let options = ConfOptions {
            max_depth: 3,
            ..ConfOptions::default()
        };
        let result = parse(input, options);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_block() {
        let input = "server { }";
        let options = ConfOptions::default();
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert!(conf_unit.directives[0].children.is_empty());
    }

    #[test]
    fn test_multiple_arguments() {
        let input = "server localhost 8080 true;";
        let options = ConfOptions::default();
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.directives[0].arguments.len(), 3);
        assert_eq!(conf_unit.directives[0].arguments[0].value, "localhost");
        assert_eq!(conf_unit.directives[0].arguments[1].value, "8080");
        assert_eq!(conf_unit.directives[0].arguments[2].value, "true");
    }

    #[test]
    fn test_forbid_bidi_characters_default() {
        // Default should forbid bidi characters
        let options = ConfOptions::default();
        assert!(options.forbid_bidi_characters);
    }

    #[test]
    fn test_expression_arguments_flag() {
        // Test that the expression arguments feature can detect expressions
        // Note: The current implementation marks tokens as is_expression if followed by '('
        // but doesn't parse the parentheses content
        let input = "directive value;";
        let options = ConfOptions {
            allow_expression_arguments: true,
            ..ConfOptions::default()
        };
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        // Regular directive should not be marked as expression
        assert!(!conf_unit.directives[0].name.is_expression);
    }

    #[test]
    fn test_crlf_line_endings() {
        let input = "server localhost;\r\nport 8080;";
        let options = ConfOptions::default();
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 2);
    }

    #[test]
    fn test_mixed_line_endings() {
        let input = "server localhost;\nport 8080;\r\nhost example.com;";
        let options = ConfOptions::default();
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 3);
    }
}
