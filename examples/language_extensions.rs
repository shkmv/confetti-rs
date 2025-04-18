use confetti_rs::{parse, ConfOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example of configuration using language extensions

    // Example with C-style comments (Annex A)
    let comments_example = r#"
# Confetti style comment.

// C-style single-line comment.

/* C-style multi-
   line comment. */

server {
    listen 80;  # Standard comment
    /* Multi-line comment
       explaining server config */
    server_name example.com;
}
"#;

    // Example with expression arguments (Annex B)
    let expressions_example = r#"
if (x > y) {
    print "x is greater than y"
}

while (count < 10) {
    increment count
    print (count * 2)
}
"#;

    // Example with punctuators (Annex C)
    let punctuators_example = r#"
x = 123
y=456
config-file = "/etc/app.conf"

database {
    host = "localhost"
    port = 5432
    credentials = { username = "admin", password = "secret" }
}
"#;

    println!("Example 1: Comment Syntax Extension (Annex A)");

    // Create parser options with C-style comments support
    let comment_options = ConfOptions {
        allow_c_style_comments: true,
        ..ConfOptions::default()
    };

    // Parse configuration with comments
    let comment_conf = parse(comments_example, comment_options)?;

    println!("Parsed configuration with C-style comments:");
    println!(
        "Found {} directives and {} comments",
        comment_conf.directives.len(),
        comment_conf.comments.len()
    );

    // Display all comments
    println!("\nComments:");
    for (i, comment) in comment_conf.comments.iter().enumerate() {
        println!("Comment {}: {}", i + 1, comment.content);
        println!("  Is multi-line: {}", comment.is_multi_line);
    }

    println!("\nExample 2: Expression Arguments Extension (Annex B)");

    // Create parser options with expression arguments support
    let expr_options = ConfOptions {
        allow_expression_arguments: true,
        ..ConfOptions::default()
    };

    // Parse configuration with expressions
    let expr_conf = parse(expressions_example, expr_options)?;

    println!("Parsed configuration with expression arguments:");

    // Function to display directives with expressions
    fn print_expression_directives(directives: &[confetti_rs::ConfDirective], indent: usize) {
        let indent_str = " ".repeat(indent * 2);

        for directive in directives {
            print!("{}Directive: {}", indent_str, directive.name.value);

            // Display arguments
            for arg in &directive.arguments {
                if arg.is_expression {
                    print!(" Expression[{}]", arg.value);
                } else {
                    print!(" {}", arg.value);
                }
            }
            println!();

            // Recursively process child directives
            if !directive.children.is_empty() {
                print_expression_directives(&directive.children, indent + 1);
            }
        }
    }

    // Display directives with expressions
    print_expression_directives(&expr_conf.directives, 0);

    println!("\nExample 3: Punctuator Arguments Extension (Annex C)");

    // For Annex C (punctuators) there is no direct support in ConfOptions,
    // but we can show how to process such configurations

    // Parse configuration with punctuators
    let punct_conf = parse(punctuators_example, ConfOptions::default())?;

    println!("Parsed configuration with punctuators:");

    // Function to process directives with punctuators
    fn process_punctuator_directives(directives: &[confetti_rs::ConfDirective], indent: usize) {
        let indent_str = " ".repeat(indent * 2);

        for directive in directives {
            // Check if directive has arguments
            if directive.arguments.len() >= 2 && directive.arguments[0].value == "=" {
                // This is an assignment (key = value)
                println!(
                    "{}Assignment: {} = {}",
                    indent_str, directive.name.value, directive.arguments[1].value
                );
            } else {
                // This is a regular directive
                println!("{}Directive: {}", indent_str, directive.name.value);

                // Display arguments
                for arg in &directive.arguments {
                    println!("{}  Argument: {}", indent_str, arg.value);
                }

                // Recursively process child directives
                if !directive.children.is_empty() {
                    process_punctuator_directives(&directive.children, indent + 1);
                }
            }
        }
    }

    // Process directives with punctuators
    process_punctuator_directives(&punct_conf.directives, 0);

    // Example of using all extensions together
    println!("\nExample 4: Using All Extensions Together");

    let combined_example = r#"
// Configuration with all extensions

/* This is a multi-line comment
   explaining the configuration */

# Server configuration
server {
    listen = 80;  // Port to listen on
    
    if (debug_mode == true) {
        log-level = "debug"
    } else {
        log-level = "info"
    }
    
    locations = {
        "/api" = { proxy_pass = "http://api-server:8080" },
        "/static" = { root = "/var/www/static" }
    }
}
"#;

    // Create parser options with all extensions
    let all_options = ConfOptions {
        allow_c_style_comments: true,
        allow_expression_arguments: true,
        ..ConfOptions::default()
    };

    // Parse configuration with all extensions
    let combined_conf = parse(combined_example, all_options)?;

    println!("Parsed configuration with all extensions:");
    println!(
        "Found {} directives and {} comments",
        combined_conf.directives.len(),
        combined_conf.comments.len()
    );

    // Display configuration structure
    fn print_combined_config(directives: &[confetti_rs::ConfDirective], indent: usize) {
        let indent_str = " ".repeat(indent * 2);

        for directive in directives {
            print!("{}Directive: {}", indent_str, directive.name.value);

            // Display arguments
            for arg in &directive.arguments {
                if arg.is_expression {
                    print!(" Expression[{}]", arg.value);
                } else if arg.is_quoted {
                    print!(" Quoted[{}]", arg.value);
                } else {
                    print!(" {}", arg.value);
                }
            }
            println!();

            // Recursively process child directives
            if !directive.children.is_empty() {
                print_combined_config(&directive.children, indent + 1);
            }
        }
    }

    // Display configuration structure
    print_combined_config(&combined_conf.directives, 0);

    Ok(())
}
