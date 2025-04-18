use confetti_rs::{parse, ConfOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example of UI configuration
    let config = r#"
Application {
    VerticalLayout {
        Label {
            text "This application has a single button."
        }

        Button {
            text "Click Me"
            on_click """
function() {
    console.log(`You clicked a button named: ${this.text}`);
}
"""
        }
    }
}
"#;

    // Create parser options with C-style comments enabled
    let options = ConfOptions {
        allow_c_style_comments: true,
        ..ConfOptions::default()
    };

    // Parse the configuration
    let conf_unit = parse(config, options)?;

    println!("Parsed UI configuration:");

    // Function for recursive display of UI components
    fn print_ui_component(directive: &confetti_rs::ConfDirective, indent: usize) {
        let indent_str = " ".repeat(indent * 2);
        println!("{}Component: {}", indent_str, directive.name.value);

        // Display component properties
        for child in &directive.children {
            if child.children.is_empty() {
                let value = if !child.arguments.is_empty() {
                    let arg = &child.arguments[0].value;
                    if child.arguments[0].is_triple_quoted {
                        // For multi-line values
                        format!("<<< Multi-line code block >>>")
                    } else if child.arguments[0].is_quoted {
                        // Remove quotes for display
                        arg[1..arg.len() - 1].to_string()
                    } else {
                        arg.clone()
                    }
                } else {
                    "No value".to_string()
                };

                println!("{}  Property: {} = {}", indent_str, child.name.value, value);
            } else {
                // Recursively process nested components
                print_ui_component(child, indent + 1);
            }
        }
    }

    // Process the root component (Application)
    if let Some(app) = conf_unit.directives.first() {
        print_ui_component(app, 0);
    }

    // Example of generating pseudocode for UI
    println!("\nGenerated UI pseudocode:");

    fn generate_ui_code(directive: &confetti_rs::ConfDirective, indent: usize) -> String {
        let indent_str = " ".repeat(indent * 2);
        let mut code = format!("{}create{}(", indent_str, directive.name.value);

        // Collect properties
        let mut properties = Vec::new();
        for child in &directive.children {
            if child.children.is_empty() {
                let value = if !child.arguments.is_empty() {
                    let arg = &child.arguments[0].value;
                    if child.arguments[0].is_triple_quoted {
                        // For multi-line values (remove triple quotes)
                        let trimmed = &arg[3..arg.len() - 3];
                        format!("{}", trimmed)
                    } else if child.arguments[0].is_quoted {
                        // Remove quotes for display
                        arg[1..arg.len() - 1].to_string()
                    } else {
                        arg.clone()
                    }
                } else {
                    "null".to_string()
                };

                properties.push(format!("{}: {}", child.name.value, value));
            }
        }

        if !properties.is_empty() {
            code.push_str(&format!("{{\n{}\n{}}}", properties.join(",\n"), indent_str));
        }

        code.push_str(")");

        // Add child components
        for child in &directive.children {
            if !child.children.is_empty() {
                code.push_str(&format!(
                    "\n{}.addChild(\n{}\n{})",
                    indent_str,
                    generate_ui_code(child, indent + 1),
                    indent_str
                ));
            }
        }

        code
    }

    // Generate code for the root component (Application)
    if let Some(app) = conf_unit.directives.first() {
        println!("{}", generate_ui_code(app, 0));
    }

    Ok(())
}
