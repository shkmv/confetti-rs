use std::error::Error;
use std::fmt;
use std::ops::Range;

pub mod lexer;
pub mod parser;

// Private module for derive macro implementation details
#[doc(hidden)]
pub mod __private {
    pub fn is_option_type(type_name: &str) -> bool {
        type_name.starts_with("core::option::Option<") || 
        type_name.starts_with("std::option::Option<")
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
            result = result[1..result.len()-1].to_string();
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
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 0);
        assert_eq!(conf_unit.comments.len(), 0);
    }

    #[test]
    fn test_parse_simple_directive() {
        let input = "server localhost";
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
        assert_eq!(conf_unit.directives[0].arguments.len(), 0);
        assert_eq!(conf_unit.directives[0].children.len(), 1);
        assert_eq!(conf_unit.directives[0].children[0].name.value, "listen");
        assert_eq!(conf_unit.directives[0].children[0].arguments.len(), 1);
        assert_eq!(conf_unit.directives[0].children[0].arguments[0].value, "80");
    }

    #[test]
    fn test_parse_with_comments() {
        let input = "# Comment\nserver localhost";
        let options = ConfOptions {
            allow_c_style_comments: true,
            ..Default::default()
        };
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.comments.len(), 1);
    }

    #[test]
    fn test_parse_quoted_arguments() {
        let input = r#"server "localhost with spaces""#;
        let options = ConfOptions::default();
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.directives[0].name.value, "server");
        assert_eq!(conf_unit.directives[0].arguments.len(), 1);
        assert_eq!(conf_unit.directives[0].arguments[0].value, "\"localhost with spaces\"");
        assert!(conf_unit.directives[0].arguments[0].is_quoted);
    }

    #[test]
    fn test_parse_triple_quoted_arguments() {
        let input = r#"server """
            localhost
            with multiple
            lines
        """"#;
        let options = ConfOptions::default();
        let result = parse(input, options);
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.directives[0].name.value, "server");
        assert_eq!(conf_unit.directives[0].arguments.len(), 1);
        assert!(conf_unit.directives[0].arguments[0].is_quoted);
        assert!(conf_unit.directives[0].arguments[0].is_triple_quoted);
    }

    #[test]
    fn test_parse_line_continuation() {
        let input = "server \\\nlocalhost";
        let options = ConfOptions::default();
        let result = parse(input, options);
        if let Err(ref e) = result {
            println!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        let conf_unit = result.unwrap();
        assert_eq!(conf_unit.directives.len(), 1);
        assert_eq!(conf_unit.directives[0].name.value, "server");
        assert_eq!(conf_unit.directives[0].arguments.len(), 1);
        assert_eq!(conf_unit.directives[0].arguments[0].value, "localhost");
    }
}

