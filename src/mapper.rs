use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use crate::{parse, ConfDirective, ConfOptions};

/// Error type for mapping operations
#[derive(Debug)]
pub enum MapperError {
    /// Error during parsing
    ParseError(String),
    /// Error during serialization
    SerializeError(String),
    /// Error during file I/O
    IoError(io::Error),
    /// Error during value conversion
    ConversionError(String),
    /// Error when a required field is missing
    MissingField(String),
}

impl Error for MapperError {}

impl fmt::Display for MapperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MapperError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            MapperError::SerializeError(msg) => write!(f, "Serialization error: {}", msg),
            MapperError::IoError(err) => write!(f, "I/O error: {}", err),
            MapperError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            MapperError::MissingField(name) => write!(f, "Missing required field: {}", name),
        }
    }
}

impl From<io::Error> for MapperError {
    fn from(error: io::Error) -> Self {
        MapperError::IoError(error)
    }
}

impl From<crate::ConfError> for MapperError {
    fn from(error: crate::ConfError) -> Self {
        MapperError::ParseError(error.to_string())
    }
}

/// Trait for types that can be mapped from configuration
pub trait FromConf: Sized {
    /// Convert from a configuration directive to the implementing type
    fn from_directive(directive: &ConfDirective) -> Result<Self, MapperError>;

    /// Create an instance from a configuration string
    fn from_str(s: &str) -> Result<Self, MapperError> {
        let options = MapperOptions::default().parser_options;
        let conf_unit = parse(s, options)?;

        if conf_unit.directives.is_empty() {
            return Err(MapperError::ParseError("No directives found".into()));
        }

        Self::from_directive(&conf_unit.directives[0])
    }

    /// Create an instance from a file
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, MapperError> {
        let content = fs::read_to_string(path)?;
        Self::from_str(&content)
    }
}

/// Trait for types that can be mapped to configuration
pub trait ToConf {
    /// Convert the implementing type to a configuration directive
    fn to_directive(&self) -> Result<ConfDirective, MapperError>;

    /// Convert the implementing type to a configuration string
    fn to_string(&self) -> Result<String, MapperError> {
        let directive = self.to_directive()?;

        // Simple serialization for now - can be enhanced later
        let mut result = String::new();
        serialize_directive(&directive, &mut result, 0)?;

        Ok(result)
    }

    /// Write the implementing type to a file
    fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), MapperError> {
        let content = self.to_string()?;
        fs::write(path, content)?;
        Ok(())
    }
}

/// Options for mapper configuration
#[derive(Debug, Clone)]
pub struct MapperOptions {
    /// Options for the parser
    pub parser_options: ConfOptions,
    /// Whether field names should be converted to kebab-case in the config
    pub use_kebab_case: bool,
    /// Indentation string to use when writing configs (defaults to 2 spaces)
    pub indent: String,
}

impl Default for MapperOptions {
    fn default() -> Self {
        Self {
            parser_options: ConfOptions::default(),
            use_kebab_case: false,
            indent: "  ".to_string(),
        }
    }
}

// Helper function to convert to kebab case
#[allow(dead_code)]
fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_lowercase = false;

    for c in s.chars() {
        if c.is_uppercase() {
            if prev_is_lowercase {
                result.push('-');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_is_lowercase = false;
        } else {
            result.push(c);
            prev_is_lowercase = true;
        }
    }

    result
}

// Helper function to convert from kebab case
#[allow(dead_code)]
fn from_kebab_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

// Private helper function to serialize a directive
fn serialize_directive(
    directive: &ConfDirective,
    output: &mut String,
    depth: usize,
) -> Result<(), MapperError> {
    // Get indent string based on depth
    let indent = "  ".repeat(depth);

    // Write directive name
    output.push_str(&indent);
    output.push_str(&directive.name.value);

    // Write arguments
    for arg in &directive.arguments {
        output.push(' ');
        if arg.is_quoted {
            output.push('"');
            // Remove quotes if they already exist in the value
            let mut value = if arg.value.starts_with('"') && arg.value.ends_with('"') {
                arg.value[1..arg.value.len() - 1].to_string()
            } else {
                arg.value.clone()
            };

            // Remove trailing commas from string values
            value = value.trim_end_matches(',').to_string();

            output.push_str(&value);
            output.push('"');
        } else {
            output.push_str(&arg.value);
        }
    }

    if directive.children.is_empty() {
        output.push_str(";\n");
    } else {
        output.push_str(" {\n");

        // Write children
        for child in &directive.children {
            serialize_directive(child, output, depth + 1)?;
        }

        output.push_str(&indent);
        output.push_str("}\n");
    }

    Ok(())
}

/// Value converter trait for converting between config strings and Rust types
pub trait ValueConverter: Sized {
    /// Convert from a string to this type
    fn from_conf_value(value: &str) -> Result<Self, MapperError>;

    /// Convert this type to a string representation
    fn to_conf_value(&self) -> Result<String, MapperError>;

    /// Determine if this type requires quotes when serialized
    fn requires_quotes(&self) -> bool {
        true // By default all types require quotes, except for those that override this method
    }
}

// Implementation for primitive types

impl ValueConverter for String {
    fn from_conf_value(value: &str) -> Result<Self, MapperError> {
        Ok(value.to_string())
    }

    fn to_conf_value(&self) -> Result<String, MapperError> {
        // Remove leading and trailing quotes if they exist
        let value = if self.starts_with('"') && self.ends_with('"') {
            &self[1..self.len() - 1]
        } else {
            &self[..]
        };

        // Remove trailing commas
        let value = value.trim_end_matches(',');

        Ok(value.to_string())
    }

    fn requires_quotes(&self) -> bool {
        true
    }
}

impl ValueConverter for bool {
    fn from_conf_value(value: &str) -> Result<Self, MapperError> {
        match value.to_lowercase().as_str() {
            "true" | "yes" | "on" | "1" => Ok(true),
            "false" | "no" | "off" | "0" => Ok(false),
            _ => Err(MapperError::ConversionError(format!(
                "Cannot convert '{}' to bool",
                value
            ))),
        }
    }

    fn to_conf_value(&self) -> Result<String, MapperError> {
        Ok(self.to_string())
    }

    fn requires_quotes(&self) -> bool {
        false
    }
}

impl ValueConverter for i32 {
    fn from_conf_value(value: &str) -> Result<Self, MapperError> {
        value.parse::<i32>().map_err(|e| {
            MapperError::ConversionError(format!("Cannot convert '{}' to i32: {}", value, e))
        })
    }

    fn to_conf_value(&self) -> Result<String, MapperError> {
        Ok(self.to_string())
    }

    fn requires_quotes(&self) -> bool {
        false
    }
}

impl ValueConverter for f64 {
    fn from_conf_value(value: &str) -> Result<Self, MapperError> {
        value.parse::<f64>().map_err(|e| {
            MapperError::ConversionError(format!("Cannot convert '{}' to f64: {}", value, e))
        })
    }

    fn to_conf_value(&self) -> Result<String, MapperError> {
        Ok(self.to_string())
    }

    fn requires_quotes(&self) -> bool {
        false
    }
}

impl<T: ValueConverter> ValueConverter for Option<T> {
    fn from_conf_value(value: &str) -> Result<Self, MapperError> {
        if value.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(T::from_conf_value(value)?))
        }
    }

    fn to_conf_value(&self) -> Result<String, MapperError> {
        match self {
            Some(val) => val.to_conf_value(),
            None => Ok("".to_string()),
        }
    }

    fn requires_quotes(&self) -> bool {
        match self {
            Some(val) => val.requires_quotes(),
            None => false,
        }
    }
}

impl<T: ValueConverter> ValueConverter for Vec<T> {
    fn from_conf_value(value: &str) -> Result<Self, MapperError> {
        let values = value
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| T::from_conf_value(s))
            .collect::<Result<Vec<T>, _>>()?;

        Ok(values)
    }

    fn to_conf_value(&self) -> Result<String, MapperError> {
        let values: Result<Vec<String>, _> = self.iter().map(|val| val.to_conf_value()).collect();

        Ok(values?.join(", "))
    }

    fn requires_quotes(&self) -> bool {
        // Vec always serializes as a string with commas
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConfArgument, ConfDirective};

    #[test]
    fn test_serialize_string_without_comma() {
        // Create a test directive with a string value that has a comma
        let directive = ConfDirective {
            name: ConfArgument {
                value: "TestConfig".to_string(),
                span: 0..0,
                is_quoted: false,
                is_triple_quoted: false,
                is_expression: false,
            },
            arguments: vec![],
            children: vec![ConfDirective {
                name: ConfArgument {
                    value: "host".to_string(),
                    span: 0..0,
                    is_quoted: false,
                    is_triple_quoted: false,
                    is_expression: false,
                },
                arguments: vec![ConfArgument {
                    value: "127.0.0.1,".to_string(),
                    span: 0..0,
                    is_quoted: true,
                    is_triple_quoted: false,
                    is_expression: false,
                }],
                children: vec![],
            }],
        };

        // Serialize the directive
        let mut output = String::new();
        serialize_directive(&directive, &mut output, 0).unwrap();

        // Verify the output has the comma removed
        assert!(output.contains("\"127.0.0.1\""));
        assert!(!output.contains("\"127.0.0.1,\""));
    }

    #[test]
    fn test_serialize_numeric_without_quotes() {
        // Create a test directive with a numeric value
        let directive = ConfDirective {
            name: ConfArgument {
                value: "TestConfig".to_string(),
                span: 0..0,
                is_quoted: false,
                is_triple_quoted: false,
                is_expression: false,
            },
            arguments: vec![],
            children: vec![ConfDirective {
                name: ConfArgument {
                    value: "port".to_string(),
                    span: 0..0,
                    is_quoted: false,
                    is_triple_quoted: false,
                    is_expression: false,
                },
                arguments: vec![ConfArgument {
                    value: "3000".to_string(),
                    span: 0..0,
                    is_quoted: false,
                    is_triple_quoted: false,
                    is_expression: false,
                }],
                children: vec![],
            }],
        };

        // Serialize the directive
        let mut output = String::new();
        serialize_directive(&directive, &mut output, 0).unwrap();

        // Verify the output has no quotes for the numeric value
        assert!(output.contains("port 3000;"));
        assert!(!output.contains("port \"3000\";"));
    }

    #[test]
    fn test_server_config_serialization() {
        // Test case similar to the reported issue
        let directive = ConfDirective {
            name: ConfArgument {
                value: "ServerConfig".to_string(),
                span: 0..0,
                is_quoted: false,
                is_triple_quoted: false,
                is_expression: false,
            },
            arguments: vec![],
            children: vec![
                ConfDirective {
                    name: ConfArgument {
                        value: "host".to_string(),
                        span: 0..0,
                        is_quoted: false,
                        is_triple_quoted: false,
                        is_expression: false,
                    },
                    arguments: vec![ConfArgument {
                        value: "127.0.0.1,".to_string(),
                        span: 0..0,
                        is_quoted: true,
                        is_triple_quoted: false,
                        is_expression: false,
                    }],
                    children: vec![],
                },
                ConfDirective {
                    name: ConfArgument {
                        value: "port".to_string(),
                        span: 0..0,
                        is_quoted: false,
                        is_triple_quoted: false,
                        is_expression: false,
                    },
                    arguments: vec![ConfArgument {
                        value: "3000".to_string(),
                        span: 0..0,
                        is_quoted: false,
                        is_triple_quoted: false,
                        is_expression: false,
                    }],
                    children: vec![],
                },
            ],
        };

        // Serialize the directive
        let mut output = String::new();
        serialize_directive(&directive, &mut output, 0).unwrap();

        // Expected output should be correct
        let expected = "ServerConfig {\n  host \"127.0.0.1\";\n  port 3000;\n}\n";

        assert_eq!(output, expected);
    }

    #[test]
    fn test_to_conf_value_string_with_quotes() {
        // Test that string values with existing quotes have them removed
        let value = "\"test value\"".to_string();
        let result = value.to_conf_value().unwrap();
        assert_eq!(result, "test value");
    }

    #[test]
    fn test_to_conf_value_string_with_comma() {
        // Test that string values with trailing commas have them removed
        let value = "test value,".to_string();
        let result = value.to_conf_value().unwrap();
        assert_eq!(result, "test value");
    }

    #[test]
    fn test_requires_quotes() {
        // Test that string values require quotes
        let string_value = String::from("test");
        assert!(string_value.requires_quotes());

        // Test that numeric values don't require quotes
        let int_value = 3000;
        assert!(!int_value.requires_quotes());

        let float_value = std::f64::consts::PI;
        assert!(!float_value.requires_quotes());

        // Test that boolean values don't require quotes
        let bool_value = true;
        assert!(!bool_value.requires_quotes());
    }
}
