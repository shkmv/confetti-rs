# Confetti-rs

A configuration language and parser library for Rust, with a flexible mapper for converting between configuration files and Rust structs.

## Features

- Simple, intuitive configuration syntax
- A powerful parser with customizable options
- Automatic mapping between configuration and Rust structs
- Support for custom data types
- Comprehensive error handling

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
confetti-rs = { version = "0.1.0", features = ["derive"] }
```

The `derive` feature enables the derive macros for automatic configuration mapping.

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

## Official Specification

This library implements the [Confetti configuration language specification](https://confetti.hgs3.me/specification/), providing a robust and fully compliant parser for the Confetti language.

## Configuration Syntax

Confetti-rs uses a simple, readable syntax:

```
DirectiveName {
  nested_directive "value";
  another_directive 123;
  
  block_directive {
    setting true;
    array 1, 2, 3, 4;
  }
}
```

## Mapping Features

### Automatic Mapping with Derive Macro

The `ConfMap` derive macro automatically implements the required traits for mapping between your structs and the configuration format:

```rust
#[derive(ConfMap, Debug)]
struct AppConfig {
    name: String,
    version: String,
    #[conf_map(name = "max-connections")]
    max_connections: i32,
}
```

### Common Usage Examples

#### Loading from a string

```rust
use confetti_rs::{ConfMap, from_str};

#[derive(ConfMap, Debug)]
struct ServerConfig {
    host: String,
    port: i32,
}

// Parse from a string
let config_str = r#"
ServerConfig {
    host "localhost";
    port 8080;
}
"#;

let config = from_str::<ServerConfig>(config_str).unwrap();
println!("Server at {}:{}", config.host, config.port);
```

#### Loading from a file

```rust
use confetti_rs::{ConfMap, from_file};
use std::path::Path;

#[derive(ConfMap, Debug)]
struct AppConfig {
    name: String,
    version: String,
}

// Load from a file
let config = from_file::<AppConfig>(Path::new("config/app.conf")).unwrap();
println!("App: {} v{}", config.name, config.version);
```

#### Serializing to a file

```rust
use confetti_rs::{ConfMap, to_file};
use std::path::Path;

#[derive(ConfMap, Debug)]
struct LogConfig {
    level: String,
    path: String,
}

let log_config = LogConfig {
    level: "info".to_string(),
    path: "/var/log/app.log".to_string(),
};

// Save to a file
to_file(&log_config, Path::new("config/logging.conf")).unwrap();
```

For more comprehensive examples, including nested structures, collections, and custom field mappings, see the [examples directory](https://github.com/shkmv/confetti-rs/tree/main/examples) in the repository.

### Custom Field Names

Use the `conf_map` attribute to customize field names in the configuration:

```rust
#[derive(ConfMap, Debug)]
struct Config {
    #[conf_map(name = "api-key")]
    api_key: String,
}
```

### Optional Fields

Fields with `Option<T>` type are treated as optional:

```rust
#[derive(ConfMap, Debug)]
struct Config {
    required_field: String,
    optional_field: Option<i32>,
}
```

### Supported Types

Out of the box, Confetti-rs supports these types:
- `String`
- `i32`, `f64`
- `bool`
- `Option<T>` where T is a supported type
- `Vec<T>` where T is a supported type

### Implementing Custom Type Conversion

For custom types, implement the `ValueConverter` trait:

```rust
use confetti_rs::{ValueConverter, MapperError};
use std::net::IpAddr;
use std::str::FromStr;

impl ValueConverter for IpAddr {
    fn from_conf_value(value: &str) -> Result<Self, MapperError> {
        IpAddr::from_str(value).map_err(|e| 
            MapperError::ConversionError(format!("Invalid IP address: {}", e))
        )
    }
    
    fn to_conf_value(&self) -> Result<String, MapperError> {
        Ok(self.to_string())
    }
}
```

### Nested Configurations

For nested configurations, you'll need to implement `FromConf` and `ToConf`:

```rust
#[derive(ConfMap, Debug)]
struct DatabaseConfig {
    host: String,
    port: i32,
}

#[derive(Debug)]
struct AppConfig {
    name: String,
    database: DatabaseConfig,
}

impl FromConf for AppConfig {
    fn from_directive(directive: &ConfDirective) -> Result<Self, MapperError> {
        // Extract simple fields
        let name = directive.children.iter()
            .find(|d| d.name.value == "name")
            .and_then(|d| d.arguments.get(0))
            .map(|arg| arg.value.clone())
            .ok_or_else(|| MapperError::MissingField("name".into()))?;
        
        // Extract nested configuration
        let db_directive = directive.children.iter()
            .find(|d| d.name.value == "database")
            .ok_or_else(|| MapperError::MissingField("database".into()))?;
        
        let database = DatabaseConfig::from_directive(db_directive)?;
        
        Ok(AppConfig { name, database })
    }
}

impl ToConf for AppConfig {
    // Implementation omitted for brevity
}
```

## Advanced Parser Options

Confetti-rs allows you to customize the parser behavior:

```rust
use confetti_rs::{MapperOptions, parse};

let options = MapperOptions {
    use_kebab_case: true,
    indent: "    ".to_string(),
    parser_options: confetti_rs::ConfOptions {
        allow_c_style_comments: true,
        allow_triple_quotes: true,
        ..Default::default()
    },
};
```

## Error Handling

Confetti-rs provides detailed error information:

```rust
match config_result {
    Ok(config) => {
        // Use the config
    },
    Err(e) => match e {
        MapperError::ParseError(msg) => println!("Parse error: {}", msg),
        MapperError::MissingField(field) => println!("Missing required field: {}", field),
        MapperError::ConversionError(msg) => println!("Type conversion error: {}", msg),
        MapperError::IoError(io_error) => println!("I/O error: {}", io_error),
        MapperError::SerializeError(msg) => println!("Serialization error: {}", msg),
    },
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details. 