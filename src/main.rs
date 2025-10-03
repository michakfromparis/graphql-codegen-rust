use clap::Parser;

mod cli;
mod config;
mod generator;
mod integration;
mod introspection;
mod logger;
mod parser;

#[cfg(feature = "yaml-codegen-config")]
use serde_yaml;

use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::generator::create_generator;
use crate::integration::{Integration, IntegrationConfig};
use crate::logger::Logger;
use crate::parser::GraphQLParser;

use fs_err as fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let logger = Logger::new(cli.verbose);

    match cli.command {
        Some(Commands::Init {
            url,
            orm,
            db,
            output,
            headers,
        }) => {
            logger.info("Initializing GraphQL codegen...");
            logger.debug(&format!("URL: {}", url));
            logger.debug(&format!("ORM: {:?}", orm));
            logger.debug(&format!("Database: {:?}", db));
            logger.debug(&format!("Output directory: {:?}", output));

            // Create output directory
            logger.trace("Creating output directory...");
            fs::create_dir_all(&output)?;

            // Create config
            let config = Config::from(&Commands::Init {
                url,
                orm,
                db,
                output,
                headers,
            });

            // Fetch and parse schema
            logger.info("Fetching GraphQL schema via introspection...");
            let parser = GraphQLParser::new();
            let schema = parser
                .parse_from_introspection(&config.url, &config.headers)
                .await?;

            // Save config
            let config_path = Config::config_path(&config.output_dir);
            logger.trace(&format!("Saving config to: {:?}", config_path));
            config.save_to_file(&config_path)?;

            // Generate code
            logger.info("Generating Rust code...");
            let generator = create_generator(&config.orm);
            crate::generate_all_code(&schema, &config, &*generator, &logger).await?;

            logger.success("Initialization complete!");
            logger.info(&format!("Config saved to: {:?}", config_path));
        }
        Some(Commands::Generate { config, output }) => {
            logger.info("Generating code...");

            // Find config file
            logger.trace("Locating config file...");
            let config_path = if let Some(path) = config {
                logger.debug(&format!("Using specified config: {:?}", path));
                path
            } else {
                logger.trace("Auto-detecting config file...");
                Config::auto_detect_config()?
            };

            logger.debug(&format!("Loading config from: {:?}", config_path));
            let mut config = Config::from_file(&config_path)?;

            // Override output if specified
            if let Some(output_dir) = output {
                logger.debug(&format!("Overriding output directory: {:?}", output_dir));
                config.output_dir = output_dir;
            }

            // Fetch and parse schema
            logger.info("Fetching GraphQL schema via introspection...");
            let parser = GraphQLParser::new();
            let schema = parser
                .parse_from_introspection(&config.url, &config.headers)
                .await?;

            // Generate code
            logger.info("Generating Rust code...");
            let generator = create_generator(&config.orm);
            crate::generate_all_code(&schema, &config, &*generator, &logger).await?;

            logger.success("Code generation complete!");
        }
        Some(Commands::Integrate {
            output,
            no_scripts,
            force,
        }) => {
            let config = IntegrationConfig {
                output_dir: output,
                add_scripts: !no_scripts,
                force,
            };

            Integration::integrate_with_existing_project(config, &logger).await?;
        }
        None => {
            // Default behavior: generate from auto-detected config
            logger.info("Generating code from auto-detected config...");

            logger.trace("Auto-detecting config file...");
            let config_path = Config::auto_detect_config()?;
            logger.debug(&format!("Loading config from: {:?}", config_path));
            let config = Config::from_file(&config_path)?;

            // Fetch and parse schema
            logger.info("Fetching GraphQL schema via introspection...");
            let parser = GraphQLParser::new();
            let schema = parser
                .parse_from_introspection(&config.url, &config.headers)
                .await?;

            // Generate code
            logger.info("Generating Rust code...");
            let generator = create_generator(&config.orm);
            crate::generate_all_code(&schema, &config, &*generator, &logger).await?;

            logger.success("Code generation complete!");
        }
    }

    Ok(())
}

async fn generate_all_code(
    schema: &parser::ParsedSchema,
    config: &Config,
    generator: &dyn generator::CodeGenerator,
    logger: &Logger,
) -> anyhow::Result<()> {
    // Create output directory structure
    logger.trace("Creating output directory structure...");
    fs::create_dir_all(&config.output_dir)?;
    let src_dir = config.output_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    // Generate schema file
    logger.trace("Generating schema file...");
    let schema_code = generator.generate_schema(schema, config)?;
    if config.orm == cli::OrmType::Diesel {
        let schema_path = src_dir.join("schema.rs");
        fs::write(schema_path, schema_code)?;
        logger.info("Generated schema.rs");
    } else if config.orm == cli::OrmType::SeaOrm {
        // Sea-ORM generates a mod.rs file at the root
        let mod_path = config.output_dir.join("mod.rs");
        fs::write(mod_path, schema_code)?;
        logger.info("Generated mod.rs");
    }

    // Generate entity files
    logger.trace("Generating entity files...");
    let entities = generator.generate_entities(schema, config)?;
    let entities_dir = src_dir.join("entities");
    fs::create_dir_all(&entities_dir)?;

    let entity_count = entities.len();
    for (filename, code) in entities {
        let entity_path = entities_dir.join(filename);
        fs::write(entity_path, code)?;
    }
    logger.info(&format!("Generated {} entity files", entity_count));

    // Generate migrations
    logger.trace("Generating migration files...");
    let migrations = generator.generate_migrations(schema, config)?;
    let migrations_dir = config.output_dir.join("migrations");
    fs::create_dir_all(&migrations_dir)?;

    let migration_count = migrations.len();
    for migration in migrations {
        let migration_dir = migrations_dir.join(&migration.name);
        fs::create_dir_all(&migration_dir)?;

        let up_path = migration_dir.join("up.sql");
        let down_path = migration_dir.join("down.sql");

        fs::write(up_path, migration.up_sql)?;
        fs::write(down_path, migration.down_sql)?;
    }
    logger.info(&format!("Generated {} migrations", migration_count));

    Ok(())
}


