use confetti_rs::{parse, ConfOptions};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example of user settings configuration
    let config = r#"
username JohnDoe
language en-US
theme dark
notifications on
"#;

    // Create parser options with default settings
    let options = ConfOptions::default();

    // Parse the configuration
    let conf_unit = parse(config, options)?;

    println!("Parsed user settings:");

    // Process directives
    for directive in &conf_unit.directives {
        let name = &directive.name.value;
        let value = if !directive.arguments.is_empty() {
            &directive.arguments[0].value
        } else {
            "No value"
        };

        println!("{}: {}", name, value);
    }

    // Example of saving configuration to a file
    let config_path = "user_settings.conf";
    fs::write(config_path, config)?;
    println!("\nConfiguration saved to {}", config_path);

    // Example of reading configuration from a file
    let read_config = fs::read_to_string(config_path)?;
    let read_conf_unit = parse(&read_config, ConfOptions::default())?;

    println!("\nRead configuration from file:");
    for directive in &read_conf_unit.directives {
        let name = &directive.name.value;
        let value = if !directive.arguments.is_empty() {
            &directive.arguments[0].value
        } else {
            "No value"
        };

        println!("{}: {}", name, value);
    }

    Ok(())
}
