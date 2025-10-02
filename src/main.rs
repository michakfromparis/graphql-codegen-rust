use clap::Parser;

mod cli;
mod config;
mod generator;
mod introspection;
mod parser;

use cli::{Cli, Commands};
use config::Config;
use generator::create_generator;
use parser::GraphQLParser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init {
            url,
            orm,
            db,
            output,
            headers,
        }) => {
            println!("Initializing GraphQL codegen...");
            println!("URL: {}", url);
            println!("ORM: {:?}", orm);
            println!("Database: {:?}", db);
            println!("Output directory: {:?}", output);

            // Create output directory
            std::fs::create_dir_all(&output)?;

            // Create config
            let config = Config::from(&Commands::Init {
                url,
                orm,
                db,
                output,
                headers,
            });

            // Fetch and parse schema
            let parser = GraphQLParser::new();
            let schema = parser
                .parse_from_introspection(&config.url, &config.headers)
                .await?;

            // Save config
            let config_path = Config::config_path(&config.output_dir);
            config.save_to_file(&config_path)?;

            // Generate code
            let generator = create_generator(&config.orm);
            generate_all_code(&schema, &config, &*generator).await?;

            println!("✅ Initialization complete!");
            println!("Config saved to: {:?}", config_path);
        }
        Some(Commands::Generate {
            config,
            types: _,
            output,
        }) => {
            println!("Generating code...");

            // Find config file
            let config_path = if let Some(path) = config {
                path
            } else {
                Config::auto_detect_config()?
            };

            let mut config = Config::from_file(&config_path)?;

            // Override output if specified
            if let Some(output_dir) = output {
                config.output_dir = output_dir;
            }

            // Fetch and parse schema
            let parser = GraphQLParser::new();
            let schema = parser
                .parse_from_introspection(&config.url, &config.headers)
                .await?;

            // Generate code
            let generator = create_generator(&config.orm);
            generate_all_code(&schema, &config, &*generator).await?;

            println!("✅ Code generation complete!");
        }
        None => {
            // Default behavior: generate from auto-detected config
            println!("Generating code from auto-detected config...");

            let config_path = Config::auto_detect_config()?;
            let config = Config::from_file(&config_path)?;

            // Fetch and parse schema
            let parser = GraphQLParser::new();
            let schema = parser
                .parse_from_introspection(&config.url, &config.headers)
                .await?;

            // Generate code
            let generator = create_generator(&config.orm);
            generate_all_code(&schema, &config, &*generator).await?;

            println!("✅ Code generation complete!");
        }
    }

    Ok(())
}

async fn generate_all_code(
    schema: &parser::ParsedSchema,
    config: &Config,
    generator: &dyn generator::CodeGenerator,
) -> anyhow::Result<()> {
    // Create output directory structure
    std::fs::create_dir_all(&config.output_dir)?;
    let src_dir = config.output_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    // Generate schema file (for Diesel)
    if config.orm == cli::OrmType::Diesel {
        let schema_code = generator.generate_schema(schema, config)?;
        let schema_path = src_dir.join("schema.rs");
        std::fs::write(schema_path, schema_code)?;
        println!("Generated schema.rs");
    }

    // Generate entity files
    let entities = generator.generate_entities(schema, config)?;
    let entities_dir = src_dir.join("entities");
    std::fs::create_dir_all(&entities_dir)?;

    let entity_count = entities.len();
    for (filename, code) in entities {
        let entity_path = entities_dir.join(filename);
        std::fs::write(entity_path, code)?;
    }
    println!("Generated {} entity files", entity_count);

    // Generate migrations
    let migrations = generator.generate_migrations(schema, config)?;
    let migrations_dir = config.output_dir.join("migrations");
    std::fs::create_dir_all(&migrations_dir)?;

    let migration_count = migrations.len();
    for migration in migrations {
        let migration_dir = migrations_dir.join(&migration.name);
        std::fs::create_dir_all(&migration_dir)?;

        let up_path = migration_dir.join("up.sql");
        let down_path = migration_dir.join("down.sql");

        std::fs::write(up_path, migration.up_sql)?;
        std::fs::write(down_path, migration.down_sql)?;
    }
    println!("Generated {} migrations", migration_count);

    Ok(())
}
