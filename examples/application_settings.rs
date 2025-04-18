use confetti_rs::{ConfOptions, parse};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example of application settings configuration with hierarchical structure
    let config = r#"
application {
    version 1.2.3
    auto-update true
    log-level debug
}

display {
    resolution 1920x1080
    full-screen true
}
"#;

    // Create parser options with default settings
    let options = ConfOptions::default();

    // Parse the configuration
    let conf_unit = parse(config, options)?;

    println!("Parsed application settings:");

    // Process top-level directives
    for directive in &conf_unit.directives {
        let section_name = &directive.name.value;
        println!("Section: {}", section_name);

        // Process child directives
        for child in &directive.children {
            let name = &child.name.value;
            let value = if !child.arguments.is_empty() {
                &child.arguments[0].value
            } else {
                "No value"
            };

            println!("  {}: {}", name, value);
        }
        println!();
    }

    // Example of accessing specific settings
    println!("Accessing specific settings:");

    // Find application version
    if let Some(app_section) = conf_unit
        .directives
        .iter()
        .find(|d| d.name.value == "application")
    {
        if let Some(version) = app_section
            .children
            .iter()
            .find(|d| d.name.value == "version")
        {
            if !version.arguments.is_empty() {
                println!("Application version: {}", version.arguments[0].value);
            }
        }
    }

    // Find display resolution
    if let Some(display_section) = conf_unit
        .directives
        .iter()
        .find(|d| d.name.value == "display")
    {
        if let Some(resolution) = display_section
            .children
            .iter()
            .find(|d| d.name.value == "resolution")
        {
            if !resolution.arguments.is_empty() {
                println!("Display resolution: {}", resolution.arguments[0].value);
            }
        }
    }

    Ok(())
}
