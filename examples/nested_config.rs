#[cfg(feature = "derive")]
use confetti_rs::{
    ConfArgument, ConfDirective, ConfMap, FromConf, MapperError, ToConf, from_str, to_string,
};
#[cfg(feature = "derive")]
use std::error::Error;

#[cfg(feature = "derive")]
fn main() -> Result<(), Box<dyn Error>> {
    println!("Confetti-rs Nested Config Example");
    println!("=================================");

    // Define configuration structures with the ConfMap derive macro
    #[derive(ConfMap, Debug)]
    struct DatabaseConfig {
        host: String,
        port: i32,
        username: String,
        password: String,
        #[conf_map(name = "max-pool-size")]
        max_pool_size: Option<i32>,
    }

    #[derive(ConfMap, Debug)]
    struct ServerConfig {
        host: String,
        port: i32,
        #[conf_map(name = "ssl-enabled")]
        ssl_enabled: bool,
    }

    // Define a nested config structure
    #[derive(Debug)]
    struct ServiceConfig {
        name: String,
        version: String,
        database: DatabaseConfig,
        server: ServerConfig,
    }

    // Implement FromConf for ServiceConfig manually
    impl FromConf for ServiceConfig {
        fn from_directive(directive: &ConfDirective) -> Result<Self, MapperError> {
            // Check if directive name matches
            if directive.name.value != "ServiceConfig" {
                return Err(MapperError::ParseError(format!(
                    "Expected directive name ServiceConfig, found {}",
                    directive.name.value
                )));
            }

            // Extract name and version from direct child directives
            let name = directive
                .children
                .iter()
                .find(|d| d.name.value == "name")
                .and_then(|d| d.arguments.get(0))
                .map(|arg| arg.value.clone())
                .ok_or_else(|| MapperError::MissingField("name".into()))?;

            let version = directive
                .children
                .iter()
                .find(|d| d.name.value == "version")
                .and_then(|d| d.arguments.get(0))
                .map(|arg| arg.value.clone())
                .ok_or_else(|| MapperError::MissingField("version".into()))?;

            // Find and parse database configuration - creating a custom directive for it
            let database_child = directive
                .children
                .iter()
                .find(|d| d.name.value == "database")
                .ok_or_else(|| MapperError::MissingField("database".into()))?;

            // Create a proper DatabaseConfig directive
            let database_directive = ConfDirective {
                name: ConfArgument {
                    value: "DatabaseConfig".to_string(),
                    span: database_child.name.span.clone(),
                    is_quoted: false,
                    is_triple_quoted: false,
                    is_expression: false,
                },
                arguments: Vec::new(),
                children: database_child.children.clone(),
            };

            let database = DatabaseConfig::from_directive(&database_directive)?;

            // Find and parse server configuration
            let server_child = directive
                .children
                .iter()
                .find(|d| d.name.value == "server")
                .ok_or_else(|| MapperError::MissingField("server".into()))?;

            // Create a proper ServerConfig directive
            let server_directive = ConfDirective {
                name: ConfArgument {
                    value: "ServerConfig".to_string(),
                    span: server_child.name.span.clone(),
                    is_quoted: false,
                    is_triple_quoted: false,
                    is_expression: false,
                },
                arguments: Vec::new(),
                children: server_child.children.clone(),
            };

            let server = ServerConfig::from_directive(&server_directive)?;

            Ok(ServiceConfig {
                name,
                version,
                database,
                server,
            })
        }
    }

    // Implement ToConf for ServiceConfig manually
    impl ToConf for ServiceConfig {
        fn to_directive(&self) -> Result<ConfDirective, MapperError> {
            let mut children = Vec::new();

            // Add name and version directives
            let name_arg = ConfArgument {
                value: self.name.clone(),
                span: 0..0,
                is_quoted: true,
                is_triple_quoted: false,
                is_expression: false,
            };

            let name_directive = ConfDirective {
                name: ConfArgument {
                    value: "name".to_string(),
                    span: 0..0,
                    is_quoted: false,
                    is_triple_quoted: false,
                    is_expression: false,
                },
                arguments: vec![name_arg],
                children: vec![],
            };

            children.push(name_directive);

            let version_arg = ConfArgument {
                value: self.version.clone(),
                span: 0..0,
                is_quoted: true,
                is_triple_quoted: false,
                is_expression: false,
            };

            let version_directive = ConfDirective {
                name: ConfArgument {
                    value: "version".to_string(),
                    span: 0..0,
                    is_quoted: false,
                    is_triple_quoted: false,
                    is_expression: false,
                },
                arguments: vec![version_arg],
                children: vec![],
            };

            children.push(version_directive);

            // Add database and server directives
            let database_directive = self.database.to_directive()?;
            children.push(database_directive);

            let server_directive = self.server.to_directive()?;
            children.push(server_directive);

            Ok(ConfDirective {
                name: ConfArgument {
                    value: "ServiceConfig".to_string(),
                    span: 0..0,
                    is_quoted: false,
                    is_triple_quoted: false,
                    is_expression: false,
                },
                arguments: vec![],
                children,
            })
        }
    }

    // Create a nested sample configuration
    let config_str = r#"
    ServiceConfig {
        name "MyService";
        version "1.0.0";
        
        database {
            host "localhost";
            port 5432;
            username "user";
            password "pass";
            max-pool-size 10;
        }
        
        server {
            host "0.0.0.0";
            port 8080;
            ssl-enabled false;
        }
    }
    "#;

    // Parse the configuration
    let service_config = from_str::<ServiceConfig>(config_str)?;

    // Print the loaded configuration
    println!("Loaded service configuration:");
    println!("  Name: {}", service_config.name);
    println!("  Version: {}", service_config.version);
    println!("  Database:");
    println!("    Host: {}", service_config.database.host);
    println!("    Port: {}", service_config.database.port);
    println!("    Username: {}", service_config.database.username);
    println!("    Password: {}", service_config.database.password);
    println!(
        "    Max Pool Size: {:?}",
        service_config.database.max_pool_size
    );
    println!("  Server:");
    println!("    Host: {}", service_config.server.host);
    println!("    Port: {}", service_config.server.port);
    println!("    SSL Enabled: {}", service_config.server.ssl_enabled);

    // Modify the configuration
    let modified_config = ServiceConfig {
        name: "MyService".to_string(),
        version: "1.1.0".to_string(),
        database: DatabaseConfig {
            host: "db.example.com".to_string(),
            port: 5432,
            username: "admin".to_string(),
            password: "secure_password".to_string(),
            max_pool_size: Some(20),
        },
        server: ServerConfig {
            host: "api.example.com".to_string(),
            port: 443,
            ssl_enabled: true,
        },
    };

    // Serialize back to a string
    let serialized = to_string(&modified_config)?;
    println!("\nSerialized configuration:");
    println!("{}", serialized);

    Ok(())
}

#[cfg(not(feature = "derive"))]
fn main() {
    println!("This example requires the 'derive' feature.");
    println!("Run with: cargo run --example nested_config --features derive");
}
