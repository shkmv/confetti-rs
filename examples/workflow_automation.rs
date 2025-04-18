use confetti_rs::{parse, ConfOptions};
use std::collections::HashMap;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example of workflow automation configuration
    let config = r#"
build {
    description "Compile the source code"
    command "echo 'Building project...'"
}

clean {
    description "Clean the build directory"
    command "echo 'Cleaning build directory...'"
}

test {
    description "Run unit tests"
    command "echo 'Running tests...'"
    depends_on { build }
}
"#;

    // Create parser options with default settings
    let options = ConfOptions::default();
    
    // Parse the configuration
    let conf_unit = parse(config, options)?;
    
    println!("Parsed workflow tasks:");
    
    // Create a structure to store tasks
    struct Task {
        description: String,
        command: String,
        dependencies: Vec<String>,
    }
    
    let mut tasks: HashMap<String, Task> = HashMap::new();
    
    // Process top-level directives (tasks)
    for directive in &conf_unit.directives {
        let task_name = directive.name.value.clone();
        let mut description = String::new();
        let mut command = String::new();
        let mut dependencies = Vec::new();
        
        // Process task properties
        for child in &directive.children {
            match child.name.value.as_str() {
                "description" => {
                    if !child.arguments.is_empty() {
                        description = child.arguments[0].value.clone();
                        // Remove quotes for display
                        description = description[1..description.len()-1].to_string();
                    }
                },
                "command" => {
                    if !child.arguments.is_empty() {
                        command = child.arguments[0].value.clone();
                        // Remove quotes for display
                        command = command[1..command.len()-1].to_string();
                    }
                },
                "depends_on" => {
                    for dep in &child.children {
                        dependencies.push(dep.name.value.clone());
                    }
                },
                _ => {}
            }
        }
        
        tasks.insert(task_name.clone(), Task {
            description,
            command,
            dependencies,
        });
        
        println!("Task: {}", task_name);
    }
    
    // Function to execute a task considering dependencies
    fn execute_task(task_name: &str, tasks: &HashMap<String, Task>, executed: &mut Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        // Check if the task has already been executed
        if executed.contains(&task_name.to_string()) {
            return Ok(());
        }
        
        // Get the task
        let task = match tasks.get(task_name) {
            Some(t) => t,
            None => return Err(format!("Task '{}' not found", task_name).into()),
        };
        
        // Execute dependencies
        for dep in &task.dependencies {
            execute_task(dep, tasks, executed)?;
        }
        
        // Execute the command
        println!("\nExecuting task: {}", task_name);
        println!("Description: {}", task.description);
        println!("Command: {}", task.command);
        
        // Execute the command in the system
        #[cfg(target_os = "windows")]
        let output = Command::new("cmd").args(&["/C", &task.command]).output()?;
        
        #[cfg(not(target_os = "windows"))]
        let output = Command::new("sh").args(&["-c", &task.command]).output()?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Output: {}", stdout);
        
        // Add the task to the list of executed tasks
        executed.push(task_name.to_string());
        
        Ok(())
    }
    
    // Execute the "test" task (which depends on "build")
    println!("\nExecuting task 'test' with dependencies:");
    let mut executed = Vec::new();
    execute_task("test", &tasks, &mut executed)?;
    
    Ok(())
}