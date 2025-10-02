use clap::Parser;
use graphql_rust_codegen::cli::{Cli, Commands, DatabaseType, OrmType};

#[test]
fn test_cli_no_args() {
    let cli = Cli::try_parse_from(["graphql-rust-codegen"]).unwrap();
    assert!(cli.command.is_none());
}

#[test]
fn test_cli_init_command() {
    let cli = Cli::try_parse_from([
        "graphql-rust-codegen",
        "init",
        "--url", "https://api.example.com/graphql",
        "--orm", "diesel",
        "--db", "sqlite",
        "--output", "./generated"
    ]).unwrap();

    match cli.command {
        Some(Commands::Init { url, orm, db, output, headers }) => {
            assert_eq!(url, "https://api.example.com/graphql");
            assert_eq!(orm, OrmType::Diesel);
            assert_eq!(db, DatabaseType::Sqlite);
            assert_eq!(output, std::path::PathBuf::from("./generated"));
            assert!(headers.is_empty());
        }
        _ => panic!("Expected Init command"),
    }
}

#[test]
fn test_cli_init_with_headers() {
    let cli = Cli::try_parse_from([
        "graphql-rust-codegen",
        "init",
        "--url", "https://api.example.com/graphql",
        "--orm", "sea-orm",
        "--db", "postgres",
        "--output", "./db",
        "-H", "Authorization:Bearer token123",
        "-H", "X-API-Key:key456"
    ]).unwrap();

    match cli.command {
        Some(Commands::Init { url, orm, db, output, headers }) => {
            assert_eq!(url, "https://api.example.com/graphql");
            assert_eq!(orm, OrmType::SeaOrm);
            assert_eq!(db, DatabaseType::Postgres);
            assert_eq!(output, std::path::PathBuf::from("./db"));
            assert_eq!(headers.len(), 2);
            assert_eq!(headers[0], ("Authorization".to_string(), "Bearer token123".to_string()));
            assert_eq!(headers[1], ("X-API-Key".to_string(), "key456".to_string()));
        }
        _ => panic!("Expected Init command"),
    }
}

#[test]
fn test_cli_generate_command() {
    let cli = Cli::try_parse_from([
        "graphql-rust-codegen",
        "generate",
        "--config", "codegen.yml"
    ]).unwrap();

    match cli.command {
        Some(Commands::Generate { config, types: _, output }) => {
            assert_eq!(config, Some(std::path::PathBuf::from("codegen.yml")));
            assert!(output.is_none());
        }
        _ => panic!("Expected Generate command"),
    }
}

#[test]
fn test_cli_generate_with_output() {
    let cli = Cli::try_parse_from([
        "graphql-rust-codegen",
        "generate",
        "--output", "./custom_output"
    ]).unwrap();

    match cli.command {
        Some(Commands::Generate { config, types: _, output }) => {
            assert!(config.is_none());
            assert_eq!(output, Some(std::path::PathBuf::from("./custom_output")));
        }
        _ => panic!("Expected Generate command"),
    }
}

#[test]
fn test_cli_invalid_command() {
    let result = Cli::try_parse_from(["graphql-rust-codegen", "invalid"]);
    assert!(result.is_err());
}
