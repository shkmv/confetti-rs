use confetti_rs::{parse, ConfOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example of document markup configuration
    let config = r#"
chapter "The Raven"
author "Edgar Allan Poe"
section "First Act" {
  paragraph {
    "Once upon a midnight dreary, while I pondered, weak and weary,"
    "Over many a quaint and curious volume of forgotten lore-"
  }
  paragraph {
    "While I nodded, nearly napping, suddenly there came a tapping,"
    "As of some one gently rapping-rapping at my chamber door."
  }
}
"#;

    // Create parser options with default settings
    let options = ConfOptions::default();
    
    // Parse the configuration
    let conf_unit = parse(config, options)?;
    
    println!("Parsed document markup:");
    
    // Find chapter title and author
    let mut chapter = "Unknown";
    let mut author = "Unknown";
    
    for directive in &conf_unit.directives {
        match directive.name.value.as_str() {
            "chapter" => {
                if !directive.arguments.is_empty() {
                    chapter = &directive.arguments[0].value;
                    // Remove quotes for display
                    chapter = &chapter[1..chapter.len()-1];
                }
            },
            "author" => {
                if !directive.arguments.is_empty() {
                    author = &directive.arguments[0].value;
                    // Remove quotes for display
                    author = &author[1..author.len()-1];
                }
            },
            _ => {}
        }
    }
    
    println!("Chapter: {}", chapter);
    println!("Author: {}", author);
    
    // Find sections and paragraphs
    for directive in &conf_unit.directives {
        if directive.name.value == "section" {
            if !directive.arguments.is_empty() {
                let section_name = &directive.arguments[0].value;
                // Remove quotes for display
                let section_name = &section_name[1..section_name.len()-1];
                println!("\nSection: {}", section_name);
                
                // Process paragraphs
                for paragraph in &directive.children {
                    if paragraph.name.value == "paragraph" {
                        println!("  Paragraph:");
                        for line in &paragraph.arguments {
                            // Remove quotes for display
                            let text = &line.value[1..line.value.len()-1];
                            println!("    {}", text);
                        }
                    }
                }
            }
        }
    }
    
    // Example of conversion to HTML
    println!("\nHTML Output:");
    println!("<h1>{}</h1>", chapter);
    println!("<h2>by {}</h2>", author);
    
    for directive in &conf_unit.directives {
        if directive.name.value == "section" {
            if !directive.arguments.is_empty() {
                let section_name = &directive.arguments[0].value;
                // Remove quotes for display
                let section_name = &section_name[1..section_name.len()-1];
                println!("<h3>{}</h3>", section_name);
                
                // Process paragraphs
                for paragraph in &directive.children {
                    if paragraph.name.value == "paragraph" {
                        println!("<p>");
                        for line in &paragraph.arguments {
                            // Remove quotes for display
                            let text = &line.value[1..line.value.len()-1];
                            println!("  {}", text);
                        }
                        println!("</p>");
                    }
                }
            }
        }
    }
    
    Ok(())
}