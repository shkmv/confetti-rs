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

```rust
use confetti_rs::{ConfMap, from_str, to_string};
use std::error::Error;

// Define a configuration structure
#[derive(ConfMap, Debug)]
struct ServerConfig {
    host: String,
    port: i32,
    #[conf_map(name = "ssl-enabled")]
    ssl_enabled: bool,
    max_connections: Option<i32>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Configuration string in Confetti syntax
    let config_str = r#"
    ServerConfig {
        host "localhost";
        port 8080;
        ssl-enabled false;
        max_connections 100;
    }
    "#;

    // Parse the configuration
    let server_config = from_str::<ServerConfig>(config_str)?;
    println!("Loaded config: {:?}", server_config);

    // Modify the configuration
    let new_config = ServerConfig {
        host: "0.0.0.0".to_string(),
        port: 443,
        ssl_enabled: true,
        max_connections: Some(200),
    };

    // Serialize to a string
    let serialized = to_string(&new_config)?;
    println!("Serialized config:\n{}", serialized);

    Ok(())
}
```

## Configuration Syntax

Confetti-rs uses a simple, readable syntax:

```ignore
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
        let mut result = value.to_string();
        if result.starts_with('"') && result.ends_with('"') {
            result = result[1..result.len() - 1].to_string();
        }
        result
    }
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
    /// Whether to allow C-style comments.
    pub allow_c_style_comments: bool,
    /// Whether to allow expression arguments.
    pub allow_expression_arguments: bool,
    /// The maximum depth of nested directives.
    pub max_depth: usize,
    /// Whether to allow bidirectional formatting characters.
    pub allow_bidi: bool,
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
            allow_bidi: false,
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
/// Load configuration from a file into a struct
///
/// # Example
///
/// ```ignore
/// use confetti_rs::{from_file, ConfMap};
///
/// #[derive(ConfMap, Debug)]
/// struct ServerConfig {
///     port: i32,
///     host: String,
///     #[conf_map(name = "max-connections")]
///     max_connections: Option<i32>,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let _server_config = ServerConfig { port: 8080, host: "localhost".into(), max_connections: Some(100) };
/// // Using from_file with explicit type parameters (both T and P):
/// // let server_config = from_file::<ServerConfig, _>("config.conf")?;
/// //
/// // Alternatively, use the FromConf trait method directly:
/// // let server_config = ServerConfig::from_file("config.conf")?;
/// # Ok(())
/// # }
/// ```
///
/// You can also directly use the `from_file` method from the trait implementation:
///
/// ```ignore
/// use confetti_rs::ConfMap;
///
/// #[derive(ConfMap, Debug)]
/// struct ServerConfig {
///     port: i32,
///     host: String,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let server_config = ServerConfig::from_file("config.conf")?;
/// println!("Server running at {}:{}", server_config.host, server_config.port);
/// # Ok(())
/// # }
/// ```
pub fn from_file<T: FromConf, P: AsRef<std::path::Path>>(
    path: P,
) -> Result<T, mapper::MapperError> {
    T::from_file(path)
}

/// Load configuration from a string into a struct
///
/// # Example
///
/// ```ignore
/// use confetti_rs::{from_str, ConfMap};
///
/// #[derive(ConfMap, Debug)]
/// struct ServerConfig {
///     port: i32,
///     host: String,
/// }
///
/// let config_str = r#"
/// ServerConfig {
///   port 8080;
///   host "localhost";
/// }
/// "#;
///
/// let server_config = from_str::<ServerConfig>(config_str).unwrap();
/// assert_eq!(server_config.port, 8080);
/// assert_eq!(server_config.host, "localhost");
/// ```
pub fn from_str<T: FromConf>(s: &str) -> Result<T, mapper::MapperError> {
    T::from_str(s)
}

/// Convert a struct to a configuration string
///
/// # Example
///
/// ```ignore
/// use confetti_rs::{to_string, ConfMap};
///
/// #[derive(ConfMap, Debug)]
/// struct ServerConfig {
///     port: i32,
///     host: String,
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

/// Save a struct to a configuration file
///
/// # Example
///
/// ```ignore
/// use confetti_rs::{to_file, ConfMap};
///
/// #[derive(ConfMap, Debug)]
/// struct ServerConfig {
///     port: i32,
///     host: String,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let server_config = ServerConfig {
///     port: 8080,
///     host: "localhost".into(),
/// };
///
/// // to_file(&server_config, "config.conf")?;
/// # Ok(())
/// # }
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
}
