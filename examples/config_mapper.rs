#[cfg(feature = "derive")]
use confetti_rs::{ConfMap, from_str, to_string};
#[cfg(feature = "derive")]
use std::error::Error;

#[cfg(feature = "derive")]
fn main() -> Result<(), Box<dyn Error>> {
    // Example with the ConfMap derive macro
    println!("Confetti-rs Config Mapper Example");
    println!("=================================");

    // Define a config structure with the ConfMap derive macro
    #[derive(ConfMap, Debug)]
    struct AppConfig {
        name: String,
        version: String,
        max_connections: i32,
        #[conf_map(name = "debug-mode")]
        debug_mode: bool,
        #[conf_map(name = "log-level")]
        log_level: Option<String>,
    }

    // Create a sample configuration
    let config_str = r#"
    AppConfig {
        name "MyApp";
        version "1.0.0";
        max_connections 100;
        debug-mode true;
        log-level "INFO";
    }
    "#;

    // Parse the configuration
    let app_config = from_str::<AppConfig>(config_str)?;

    // Print the loaded configuration
    println!("Loaded configuration:");
    println!("  Name: {}", app_config.name);
    println!("  Version: {}", app_config.version);
    println!("  Max Connections: {}", app_config.max_connections);
    println!("  Debug Mode: {}", app_config.debug_mode);
    println!("  Log Level: {:?}", app_config.log_level);

    // Modify the configuration
    let modified_config = AppConfig {
        name: "MyApp".to_string(),
        version: "1.0.1".to_string(),
        max_connections: 200,
        debug_mode: false,
        log_level: Some("DEBUG".to_string()),
    };

    // Serialize back to a string
    let serialized = to_string(&modified_config)?;
    println!("\nSerialized configuration:");
    println!("{}", serialized);

    // For nested configurations, see the nested_config.rs example
    println!("\nNote: For nested configurations, see examples/nested_config.rs");

    Ok(())
}

#[cfg(not(feature = "derive"))]
fn main() {
    println!("This example requires the 'derive' feature.");
    println!("Run with: cargo run --example config_mapper --features derive");
}
