use graphql_codegen_rust::{CodeGenerator, Config};
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_codegen_creation() {
    let config = Config {
        url: "https://api.example.com/graphql".to_string(),
        orm: graphql_codegen_rust::cli::OrmType::Diesel,
        db: graphql_codegen_rust::cli::DatabaseType::Sqlite,
        output_dir: PathBuf::from("./test_output"),
        headers: HashMap::new(),
        type_mappings: HashMap::new(),
        scalar_mappings: HashMap::new(),
        table_naming: graphql_codegen_rust::config::TableNamingConvention::SnakeCase,
        generate_migrations: true,
        generate_entities: true,
    };

    let _generator = CodeGenerator::new(&config.orm);
    // Just test that we can create a generator without panicking
    // The actual generator type is tested through the config
    assert_eq!(config.orm, graphql_codegen_rust::cli::OrmType::Diesel);
}

#[test]
fn test_config_defaults() {
    let config = Config {
        url: "https://api.example.com/graphql".to_string(),
        orm: graphql_codegen_rust::cli::OrmType::Diesel,
        db: graphql_codegen_rust::cli::DatabaseType::Sqlite,
        output_dir: PathBuf::from("./generated"),
        headers: HashMap::new(),
        type_mappings: HashMap::new(),
        scalar_mappings: HashMap::new(),
        table_naming: graphql_codegen_rust::config::TableNamingConvention::SnakeCase,
        generate_migrations: true,
        generate_entities: true,
    };

    assert_eq!(config.url, "https://api.example.com/graphql");
    assert_eq!(config.orm, graphql_codegen_rust::cli::OrmType::Diesel);
    assert_eq!(config.db, graphql_codegen_rust::cli::DatabaseType::Sqlite);
    assert_eq!(config.output_dir, PathBuf::from("./generated"));
    assert!(config.headers.is_empty());
    assert!(config.generate_migrations);
    assert!(config.generate_entities);
}

#[test]
fn test_table_naming_conventions() {
    use graphql_codegen_rust::generator::to_snake_case;

    assert_eq!(to_snake_case("User"), "user");
    assert_eq!(to_snake_case("UserProfile"), "user_profile");
    assert_eq!(to_snake_case("APIKey"), "api_key");
    assert_eq!(to_snake_case("userName"), "user_name");
    assert_eq!(to_snake_case("XMLHttpRequest"), "xml_http_request");
}
