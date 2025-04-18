# Examples of using the confetti-rs library

This directory contains examples of using the confetti-rs library for various scenarios.

## Running examples

To run an example, use the following command:

```bash
cargo run --example <example_name>
```

For example:

```bash
cargo run --example user_settings
```

## Available examples

### Basic examples

1. **user_settings.rs** - Example of using the library for storing user settings.
   ```bash
   cargo run --example user_settings
   ```

2. **application_settings.rs** - Example of using the library for storing application settings with hierarchical structure.
   ```bash
   cargo run --example application_settings
   ```

3. **document_markup.rs** - Example of using the library for document markup.
   ```bash
   cargo run --example document_markup
   ```

### Advanced examples

4. **workflow_automation.rs** - Example of using the library for workflow automation.
   ```bash
   cargo run --example workflow_automation
   ```

5. **ui_configuration.rs** - Example of using the library for UI configuration.
   ```bash
   cargo run --example ui_configuration
   ```

6. **ai_training_config.rs** - Example of using the library for AI model training configuration.
   ```bash
   cargo run --example ai_training_config
   ```

### Domain-specific language examples

7. **domain_specific_language.rs** - Example of using the library to create domain-specific languages.
   ```bash
   cargo run --example domain_specific_language
   ```

### Language extension examples

8. **language_extensions.rs** - Example of using language extensions (C-style comments, expression arguments, punctuators).
   ```bash
   cargo run --example language_extensions
   ```

## Example structure

Each example demonstrates:

1. Parsing a configuration file
2. Processing the resulting data structure
3. Applying various parser options
4. Practical use of parsing results

## Additional information

More detailed information about the capabilities of the confetti-rs library can be found in the documentation and on the official website: [https://confetti.hgs3.me/](https://confetti.hgs3.me/)