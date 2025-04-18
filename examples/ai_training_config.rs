use confetti_rs::{parse, ConfOptions};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example of AI model training configuration
    let config = r#"
model {
  type "neural_network"
  architecture {
    layers {
      layer { type input; size 784 }
      layer { type dense; units 128; activation "relu" }
      layer { type output; units 10; activation "softmax" }
    }
  }

  training {
    data "/path/to/training/data"
    epochs 20
    early_stopping on
  }
}
"#;

    // Create parser options with default settings
    let options = ConfOptions::default();
    
    // Parse the configuration
    let conf_unit = parse(config, options)?;
    
    println!("Parsed AI training configuration:");
    
    // Find the root "model" directive
    if let Some(model) = conf_unit.directives.iter().find(|d| d.name.value == "model") {
        // Get the model type
        if let Some(type_dir) = model.children.iter().find(|d| d.name.value == "type") {
            if !type_dir.arguments.is_empty() {
                let model_type = &type_dir.arguments[0].value;
                // Remove quotes for display
                let model_type = &model_type[1..model_type.len()-1];
                println!("Model type: {}", model_type);
            }
        }
        
        // Get the model architecture
        if let Some(arch) = model.children.iter().find(|d| d.name.value == "architecture") {
            if let Some(layers) = arch.children.iter().find(|d| d.name.value == "layers") {
                println!("\nModel architecture:");
                
                // Process layers
                for (i, layer) in layers.children.iter().enumerate() {
                    if layer.name.value == "layer" {
                        println!("Layer {}:", i + 1);
                        
                        // Create a dictionary of layer properties
                        let mut properties = HashMap::new();
                        
                        for child in &layer.children {
                            let value = if !child.arguments.is_empty() {
                                let arg = &child.arguments[0].value;
                                if child.arguments[0].is_quoted {
                                    // Remove quotes for display
                                    arg[1..arg.len()-1].to_string()
                                } else {
                                    arg.clone()
                                }
                            } else {
                                "No value".to_string()
                            };
                            
                            properties.insert(child.name.value.clone(), value);
                        }
                        
                        // Display layer properties
                        for (key, value) in &properties {
                            println!("  {}: {}", key, value);
                        }
                    }
                }
            }
        }
        
        // Get training parameters
        if let Some(training) = model.children.iter().find(|d| d.name.value == "training") {
            println!("\nTraining parameters:");
            
            for child in &training.children {
                let value = if !child.arguments.is_empty() {
                    let arg = &child.arguments[0].value;
                    if child.arguments[0].is_quoted {
                        // Remove quotes for display
                        arg[1..arg.len()-1].to_string()
                    } else {
                        arg.clone()
                    }
                } else {
                    "No value".to_string()
                };
                
                println!("  {}: {}", child.name.value, value);
            }
        }
    }
    
    // Example of converting configuration to model training code
    println!("\nGenerated model training pseudocode:");
    println!("```python");
    println!("import tensorflow as tf");
    println!("from tensorflow.keras import layers, models");
    println!("");
    println!("# Create model");
    println!("model = models.Sequential()");
    
    // Find model layers
    if let Some(model) = conf_unit.directives.iter().find(|d| d.name.value == "model") {
        if let Some(arch) = model.children.iter().find(|d| d.name.value == "architecture") {
            if let Some(layers) = arch.children.iter().find(|d| d.name.value == "layers") {
                for layer in &layers.children {
                    if layer.name.value == "layer" {
                        // Get layer type
                        let layer_type = layer.children.iter()
                            .find(|d| d.name.value == "type")
                            .and_then(|d| d.arguments.first())
                            .map(|a| a.value.clone())
                            .unwrap_or_else(|| "unknown".to_string());
                        
                        match layer_type.as_str() {
                            "input" => {
                                let size = layer.children.iter()
                                    .find(|d| d.name.value == "size")
                                    .and_then(|d| d.arguments.first())
                                    .map(|a| a.value.clone())
                                    .unwrap_or_else(|| "0".to_string());
                                
                                println!("model.add(layers.Flatten(input_shape=({},)))", size);
                            },
                            "dense" => {
                                let units = layer.children.iter()
                                    .find(|d| d.name.value == "units")
                                    .and_then(|d| d.arguments.first())
                                    .map(|a| a.value.clone())
                                    .unwrap_or_else(|| "0".to_string());
                                
                                let activation = layer.children.iter()
                                    .find(|d| d.name.value == "activation")
                                    .and_then(|d| d.arguments.first())
                                    .map(|a| a.value.clone())
                                    .unwrap_or_else(|| "\"linear\"".to_string());
                                
                                // Remove quotes for display
                                let activation = if activation.starts_with('"') && activation.ends_with('"') {
                                    &activation[1..activation.len()-1]
                                } else {
                                    &activation
                                };
                                
                                println!("model.add(layers.Dense({}, activation='{}'))", units, activation);
                            },
                            "output" => {
                                let units = layer.children.iter()
                                    .find(|d| d.name.value == "units")
                                    .and_then(|d| d.arguments.first())
                                    .map(|a| a.value.clone())
                                    .unwrap_or_else(|| "0".to_string());
                                
                                let activation = layer.children.iter()
                                    .find(|d| d.name.value == "activation")
                                    .and_then(|d| d.arguments.first())
                                    .map(|a| a.value.clone())
                                    .unwrap_or_else(|| "\"linear\"".to_string());
                                
                                // Remove quotes for display
                                let activation = if activation.starts_with('"') && activation.ends_with('"') {
                                    &activation[1..activation.len()-1]
                                } else {
                                    &activation
                                };
                                
                                println!("model.add(layers.Dense({}, activation='{}'))", units, activation);
                            },
                            _ => println!("# Unknown layer type: {}", layer_type),
                        }
                    }
                }
            }
        }
        
        // Get training parameters
        if let Some(training) = model.children.iter().find(|d| d.name.value == "training") {
            let data_path = training.children.iter()
                .find(|d| d.name.value == "data")
                .and_then(|d| d.arguments.first())
                .map(|a| a.value.clone())
                .unwrap_or_else(|| "\"/path/to/data\"".to_string());
            
            let epochs = training.children.iter()
                .find(|d| d.name.value == "epochs")
                .and_then(|d| d.arguments.first())
                .map(|a| a.value.clone())
                .unwrap_or_else(|| "10".to_string());
            
            let early_stopping = training.children.iter()
                .find(|d| d.name.value == "early_stopping")
                .and_then(|d| d.arguments.first())
                .map(|a| a.value.clone())
                .unwrap_or_else(|| "off".to_string());
            
            // Remove quotes for display
            let data_path = if data_path.starts_with('"') && data_path.ends_with('"') {
                &data_path[1..data_path.len()-1]
            } else {
                &data_path
            };
            
            println!("");
            println!("# Compile model");
            println!("model.compile(optimizer='adam',");
            println!("              loss='sparse_categorical_crossentropy',");
            println!("              metrics=['accuracy'])");
            println!("");
            println!("# Load data");
            println!("train_data = load_data('{}')", data_path);
            println!("");
            println!("# Training callbacks");
            println!("callbacks = []");
            
            if early_stopping == "on" {
                println!("callbacks.append(tf.keras.callbacks.EarlyStopping(patience=3))");
            }
            
            println!("");
            println!("# Train model");
            println!("model.fit(train_data,");
            println!("          epochs={},", epochs);
            println!("          callbacks=callbacks)");
        }
    }
    
    println!("```");
    
    Ok(())
}