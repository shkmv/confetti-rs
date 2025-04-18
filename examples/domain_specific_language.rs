use confetti_rs::{ConfOptions, parse};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example of stack-based language configuration
    let stack_language = r#"
push 1
push 2
add     # Pop the top two numbers and push their sum.
pop $x  # Pop the sum and store it in $x.
print "1 + 2 ="
print $x

func sum x y {
    add       # Pop the arguments 'x' and 'y' and push their sum.
    return 1  # One return value is left on the stack.
}
"#;

    // Example of control flow configuration
    let control_flow = r#"
# Try sending the user a message about their access level.
set $retry-count to 3
for $i in $retry-count {
    if $is_admin {
        print "Access granted"
        send_email "admin@example.com"
        exit 0 # Success!
    } else {
        sleep 1s # Let's try again after a moment.
    }
}
exit 1 # Failed to confirm admin role.
"#;

    // Example of state machine configuration
    let state_machine = r#"
states {
    greet_player {
        look_at $player
        wait 1s # Pause one second before walking towards the player.
        walk_to $player
        say "Good evening traveler."
    }

    last_words {
        say "Tis a cruel world!"
    }
}

events {
    player_spotted {
        goto_state greet_player
    }

    died {
        goto_state last_words
    }
}
"#;

    // Create parser options with C-style comments support
    let options = ConfOptions {
        allow_c_style_comments: true,
        ..ConfOptions::default()
    };

    // Parse configurations
    let stack_conf = parse(stack_language, options.clone())?;
    let control_conf = parse(control_flow, options.clone())?;
    let state_conf = parse(state_machine, options.clone())?;

    println!("Parsed Stack-Based Language:");

    // Interpreter for stack-based language
    struct StackInterpreter {
        stack: Vec<i32>,
        variables: HashMap<String, i32>,
    }

    impl StackInterpreter {
        fn new() -> Self {
            Self {
                stack: Vec::new(),
                variables: HashMap::new(),
            }
        }

        fn interpret(&mut self, directives: &[confetti_rs::ConfDirective]) {
            for directive in directives {
                match directive.name.value.as_str() {
                    "push" => {
                        if !directive.arguments.is_empty() {
                            if let Ok(value) = directive.arguments[0].value.parse::<i32>() {
                                self.stack.push(value);
                                println!("  Push {} -> Stack: {:?}", value, self.stack);
                            }
                        }
                    }
                    "add" => {
                        if self.stack.len() >= 2 {
                            let b = self.stack.pop().unwrap();
                            let a = self.stack.pop().unwrap();
                            let result = a + b;
                            self.stack.push(result);
                            println!(
                                "  Add {} + {} = {} -> Stack: {:?}",
                                a, b, result, self.stack
                            );
                        }
                    }
                    "pop" => {
                        if !self.stack.is_empty() && !directive.arguments.is_empty() {
                            let value = self.stack.pop().unwrap();
                            let var_name = directive.arguments[0]
                                .value
                                .trim_start_matches('$')
                                .to_string();
                            self.variables.insert(var_name.clone(), value);
                            println!("  Pop {} -> Variable ${} = {}", value, var_name, value);
                        }
                    }
                    "print" => {
                        if !directive.arguments.is_empty() {
                            let arg = &directive.arguments[0].value;
                            if arg.starts_with('$') {
                                // This is a variable
                                let var_name = arg.trim_start_matches('$');
                                if let Some(value) = self.variables.get(var_name) {
                                    println!("  Print: {}", value);
                                }
                            } else if arg.starts_with('"') && arg.ends_with('"') {
                                // This is a string
                                let text = &arg[1..arg.len() - 1];
                                println!("  Print: {}", text);
                            }
                        }
                    }
                    "func" => {
                        if !directive.arguments.is_empty() {
                            let func_name = &directive.arguments[0].value;
                            let mut params = Vec::new();

                            // Collect function parameters
                            for i in 1..directive.arguments.len() {
                                params.push(directive.arguments[i].value.clone());
                            }

                            println!(
                                "  Define function: {} with parameters: {:?}",
                                func_name, params
                            );
                            println!("  Function body:");

                            // Display function body
                            for child in &directive.children {
                                println!("    {}", child.name.value);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Create and run interpreter for stack-based language
    let mut stack_interpreter = StackInterpreter::new();
    stack_interpreter.interpret(&stack_conf.directives);

    println!("\nParsed Control Flow Language:");

    // Function to display control flow structure
    fn print_control_flow(directives: &[confetti_rs::ConfDirective], indent: usize) {
        let indent_str = " ".repeat(indent * 2);

        for directive in directives {
            match directive.name.value.as_str() {
                "set" => {
                    if directive.arguments.len() >= 3 {
                        println!(
                            "{}Set variable {} {} {}",
                            indent_str,
                            directive.arguments[0].value,
                            directive.arguments[1].value,
                            directive.arguments[2].value
                        );
                    }
                }
                "for" => {
                    if directive.arguments.len() >= 3 {
                        println!(
                            "{}For loop: {} {} {}",
                            indent_str,
                            directive.arguments[0].value,
                            directive.arguments[1].value,
                            directive.arguments[2].value
                        );

                        print_control_flow(&directive.children, indent + 1);
                    }
                }
                "if" => {
                    if !directive.arguments.is_empty() {
                        println!(
                            "{}If condition: {}",
                            indent_str, directive.arguments[0].value
                        );
                        print_control_flow(&directive.children, indent + 1);
                    }
                }
                "else" => {
                    println!("{}Else branch:", indent_str);
                    print_control_flow(&directive.children, indent + 1);
                }
                "print" => {
                    if !directive.arguments.is_empty() {
                        let arg = &directive.arguments[0].value;
                        if arg.starts_with('"') && arg.ends_with('"') {
                            println!("{}Print: {}", indent_str, &arg[1..arg.len() - 1]);
                        } else {
                            println!("{}Print: {}", indent_str, arg);
                        }
                    }
                }
                "send_email" => {
                    if !directive.arguments.is_empty() {
                        let arg = &directive.arguments[0].value;
                        if arg.starts_with('"') && arg.ends_with('"') {
                            println!("{}Send email to: {}", indent_str, &arg[1..arg.len() - 1]);
                        } else {
                            println!("{}Send email to: {}", indent_str, arg);
                        }
                    }
                }
                "sleep" => {
                    if !directive.arguments.is_empty() {
                        println!("{}Sleep for: {}", indent_str, directive.arguments[0].value);
                    }
                }
                "exit" => {
                    if !directive.arguments.is_empty() {
                        println!(
                            "{}Exit with code: {}",
                            indent_str, directive.arguments[0].value
                        );
                    }
                }
                _ => {
                    println!("{}Unknown command: {}", indent_str, directive.name.value);
                }
            }
        }
    }

    // Display control flow structure
    print_control_flow(&control_conf.directives, 0);

    println!("\nParsed State Machine:");

    // Function to display state machine structure
    fn print_state_machine(directives: &[confetti_rs::ConfDirective]) {
        for directive in directives {
            match directive.name.value.as_str() {
                "states" => {
                    println!("States:");

                    for state in &directive.children {
                        println!("  State: {}", state.name.value);

                        for action in &state.children {
                            let args = action
                                .arguments
                                .iter()
                                .map(|a| a.value.clone())
                                .collect::<Vec<String>>()
                                .join(" ");

                            println!("    Action: {} {}", action.name.value, args);
                        }
                    }
                }
                "events" => {
                    println!("\nEvents:");

                    for event in &directive.children {
                        println!("  Event: {}", event.name.value);

                        for action in &event.children {
                            let args = action
                                .arguments
                                .iter()
                                .map(|a| a.value.clone())
                                .collect::<Vec<String>>()
                                .join(" ");

                            println!("    Action: {} {}", action.name.value, args);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Display state machine structure
    print_state_machine(&state_conf.directives);

    // Example of generating code for state machine
    println!("\nGenerated State Machine Code:");
    println!("```python");
    println!("class StateMachine:");
    println!("    def __init__(self):");
    println!("        self.current_state = None");
    println!("        self.states = {{}}");
    println!("        self.events = {{}}");
    println!("        self._setup()");
    println!("");
    println!("    def _setup(self):");

    // Generate code for states
    if let Some(states) = state_conf
        .directives
        .iter()
        .find(|d| d.name.value == "states")
    {
        for state in &states.children {
            println!("        # Define state: {}", state.name.value);
            println!("        def state_{}(self, entity):", state.name.value);

            for action in &state.children {
                let args = action
                    .arguments
                    .iter()
                    .map(|a| {
                        if a.value.starts_with('$') {
                            format!("entity.{}", a.value.trim_start_matches('$'))
                        } else if a.value.starts_with('"') && a.value.ends_with('"') {
                            a.value[1..a.value.len() - 1].to_string()
                        } else {
                            a.value.clone()
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(", ");

                println!("            entity.{}({})", action.name.value, args);
            }

            println!("");
            println!(
                "        self.states['{}'] = state_{}",
                state.name.value, state.name.value
            );
            println!("");
        }
    }

    // Generate code for events
    if let Some(events) = state_conf
        .directives
        .iter()
        .find(|d| d.name.value == "events")
    {
        for event in &events.children {
            println!("        # Define event: {}", event.name.value);
            println!("        def event_{}(self, entity):", event.name.value);

            for action in &event.children {
                if action.name.value == "goto_state" && !action.arguments.is_empty() {
                    println!(
                        "            self.current_state = '{}'",
                        action.arguments[0].value
                    );
                    println!("            self.states[self.current_state](entity)");
                }
            }

            println!("");
            println!(
                "        self.events['{}'] = event_{}",
                event.name.value, event.name.value
            );
            println!("");
        }
    }

    println!("    def trigger_event(self, event_name, entity):");
    println!("        if event_name in self.events:");
    println!("            self.events[event_name](entity)");
    println!("```");

    Ok(())
}
