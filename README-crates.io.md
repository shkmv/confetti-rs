# Confetti-rs

[![Rust](https://github.com/shkmv/confetti-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/shkmv/confetti-rs/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/confetti-rs.svg)](https://crates.io/crates/confetti-rs)
[![Documentation](https://docs.rs/confetti-rs/badge.svg)](https://docs.rs/confetti-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

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
confetti-rs = "0.1.0"

# If you need derive macros
confetti-rs = { version = "0.1.0", features = ["derive"] }
```

## Basic Usage

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

    // Serialize to a string
    let serialized = to_string(&server_config)?;
    
    Ok(())
}
```

## More Information

For more examples and detailed documentation, please visit:
- [GitHub repository](https://github.com/shkmv/confetti-rs)
- [Comprehensive documentation on docs.rs](https://docs.rs/confetti-rs)
- [Release notes and changelog](https://github.com/shkmv/confetti-rs/blob/main/CHANGELOG.md) 