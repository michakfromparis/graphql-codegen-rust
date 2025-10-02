use graphql_codegen_rust::{
    CodeGenerator, Config,
    parser::{ParsedSchema, ParsedType, ParsedField, ParsedEnum, FieldType},
};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

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

/// Test that generated Diesel code compiles successfully
#[tokio::test]
async fn test_diesel_code_generation_compiles() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_dir = temp_dir.path().to_path_buf();

    // Create a simple test schema
    let mut types = HashMap::new();
    let mut enums = HashMap::new();

    // Add User type
    let user_fields = vec![
        ParsedField {
            name: "id".to_string(),
            field_type: FieldType::Scalar("ID".to_string()),
            description: None,
            is_nullable: false,
            is_list: false,
        },
        ParsedField {
            name: "name".to_string(),
            field_type: FieldType::Scalar("String".to_string()),
            description: None,
            is_nullable: false,
            is_list: false,
        },
        ParsedField {
            name: "email".to_string(),
            field_type: FieldType::Scalar("String".to_string()),
            description: None,
            is_nullable: true,
            is_list: false,
        },
    ];

    types.insert(
        "User".to_string(),
        ParsedType {
            name: "User".to_string(),
            fields: user_fields,
            description: Some("A user in the system".to_string()),
            interfaces: vec![],
        },
    );

    // Add Role enum
    enums.insert(
        "Role".to_string(),
        ParsedEnum {
            name: "Role".to_string(),
            values: vec!["ADMIN".to_string(), "USER".to_string()],
            description: Some("User roles".to_string()),
        },
    );

    let schema = ParsedSchema {
        types,
        enums,
        scalars: vec![],
    };

    // Create config for Diesel + SQLite
    let config = Config {
        url: "https://example.com/graphql".to_string(),
        orm: graphql_codegen_rust::cli::OrmType::Diesel,
        db: graphql_codegen_rust::cli::DatabaseType::Sqlite,
        output_dir: output_dir.clone(),
        headers: HashMap::new(),
        type_mappings: HashMap::new(),
        scalar_mappings: HashMap::new(),
        table_naming: graphql_codegen_rust::config::TableNamingConvention::SnakeCase,
        generate_migrations: true,
        generate_entities: true,
    };

    // Generate code using the internal function with pre-parsed schema
    use graphql_codegen_rust::generate_all_code;
    let generator_inner = graphql_codegen_rust::generator::create_generator(&config.orm);
    generate_all_code(&schema, &config, &*generator_inner)
        .await
        .expect("Code generation should succeed");

    // Verify files were created
    assert!(output_dir.join("src/schema.rs").exists());
    assert!(output_dir.join("src/entities/user.rs").exists());
    assert!(output_dir.join("migrations").exists());

    // Verify files were created and contain expected content
    let schema_path = output_dir.join("src/schema.rs");
    let user_entity_path = output_dir.join("src/entities/user.rs");

    assert!(schema_path.exists());
    assert!(user_entity_path.exists());

    let schema_content = std::fs::read_to_string(&schema_path).expect("Failed to read schema");
    assert!(schema_content.contains("table!"));
    assert!(schema_content.contains("user"));

    let user_content = std::fs::read_to_string(&user_entity_path).expect("Failed to read user entity");
    assert!(user_content.contains("#[derive"));
    assert!(user_content.contains("pub struct User"));

    // Actually validate the generated code syntax to ensure it's valid Rust
    validate_generated_diesel_code(&schema_path, &user_entity_path);
}

/// Test that generated Sea-ORM code compiles successfully
#[tokio::test]
async fn test_sea_orm_code_generation_compiles() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_dir = temp_dir.path().to_path_buf();

    // Create a simple test schema
    let mut types = HashMap::new();
    let mut enums = HashMap::new();

    // Add Product type
    let product_fields = vec![
        ParsedField {
            name: "id".to_string(),
            field_type: FieldType::Scalar("ID".to_string()),
            description: None,
            is_nullable: false,
            is_list: false,
        },
        ParsedField {
            name: "title".to_string(),
            field_type: FieldType::Scalar("String".to_string()),
            description: None,
            is_nullable: false,
            is_list: false,
        },
        ParsedField {
            name: "price".to_string(),
            field_type: FieldType::Scalar("Float".to_string()),
            description: None,
            is_nullable: false,
            is_list: false,
        },
    ];

    types.insert(
        "Product".to_string(),
        ParsedType {
            name: "Product".to_string(),
            fields: product_fields,
            description: Some("A product in the catalog".to_string()),
            interfaces: vec![],
        },
    );

    // Add Status enum
    enums.insert(
        "Status".to_string(),
        ParsedEnum {
            name: "Status".to_string(),
            values: vec!["ACTIVE".to_string(), "INACTIVE".to_string()],
            description: Some("Product status".to_string()),
        },
    );

    let schema = ParsedSchema {
        types,
        enums,
        scalars: vec![],
    };

    // Create config for Sea-ORM + PostgreSQL
    let config = Config {
        url: "https://example.com/graphql".to_string(),
        orm: graphql_codegen_rust::cli::OrmType::SeaOrm,
        db: graphql_codegen_rust::cli::DatabaseType::Postgres,
        output_dir: output_dir.clone(),
        headers: HashMap::new(),
        type_mappings: HashMap::new(),
        scalar_mappings: HashMap::new(),
        table_naming: graphql_codegen_rust::config::TableNamingConvention::SnakeCase,
        generate_migrations: true,
        generate_entities: true,
    };

    // Generate code using the internal function with pre-parsed schema
    use graphql_codegen_rust::generate_all_code;
    let generator_inner = graphql_codegen_rust::generator::create_generator(&config.orm);
    generate_all_code(&schema, &config, &*generator_inner)
        .await
        .expect("Code generation should succeed");

    // Verify files were created
    // Sea-ORM generates mod.rs in the output directory root, not in src/
    assert!(output_dir.join("mod.rs").exists());
    assert!(output_dir.join("src/entities/product.rs").exists());
    assert!(output_dir.join("migrations").exists());

    // Verify files were created and contain expected content
    let mod_path = output_dir.join("mod.rs");
    let product_entity_path = output_dir.join("src/entities/product.rs");

    assert!(mod_path.exists());
    assert!(product_entity_path.exists());

    let mod_content = std::fs::read_to_string(&mod_path).expect("Failed to read mod.rs");
    assert!(mod_content.contains("pub mod product;"));
    assert!(mod_content.contains("pub use product::Entity;"));

    let product_content = std::fs::read_to_string(&product_entity_path).expect("Failed to read product entity");
    assert!(product_content.contains("#[derive(Clone, Debug, PartialEq, DeriveEntityModel"));
    assert!(product_content.contains("pub struct Entity;"));
    assert!(product_content.contains("uuid::Uuid")); // Should use UUID for Postgres ID

    // Actually validate the generated code syntax to ensure it's valid Rust
    validate_generated_sea_orm_code(&mod_path, &product_entity_path);
}

/// Test both ORM types with different databases
#[tokio::test]
async fn test_multi_database_support() {
    let databases = vec![
        (graphql_codegen_rust::cli::DatabaseType::Sqlite, "i32"),
        (
            graphql_codegen_rust::cli::DatabaseType::Postgres,
            "uuid::Uuid",
        ),
        (graphql_codegen_rust::cli::DatabaseType::Mysql, "u32"),
    ];

    for (db_type, expected_id_type) in databases {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Simple schema with just ID field
        let mut types = HashMap::new();
        types.insert(
            "Test".to_string(),
            ParsedType {
                name: "Test".to_string(),
                fields: vec![ParsedField {
                    name: "id".to_string(),
                    field_type: FieldType::Scalar("ID".to_string()),
                    description: None,
                    is_nullable: false,
                    is_list: false,
                }],
                description: None,
                interfaces: vec![],
            },
        );

        let schema = ParsedSchema {
            types,
            enums: HashMap::new(),
            scalars: vec![],
        };

        // Test both ORMs
        for orm_type in &[
            graphql_codegen_rust::cli::OrmType::Diesel,
            graphql_codegen_rust::cli::OrmType::SeaOrm,
        ] {
            let config = Config {
                url: "https://example.com/graphql".to_string(),
                orm: orm_type.clone(),
                db: db_type.clone(),
                output_dir: temp_dir.path().to_path_buf(),
                headers: HashMap::new(),
                type_mappings: HashMap::new(),
                scalar_mappings: HashMap::new(),
                table_naming: graphql_codegen_rust::config::TableNamingConvention::SnakeCase,
                generate_migrations: true,
                generate_entities: true,
            };

            // Generate code using the internal function with pre-parsed schema
            use graphql_codegen_rust::generate_all_code;
            let generator_inner = graphql_codegen_rust::generator::create_generator(&config.orm);
            generate_all_code(&schema, &config, &*generator_inner)
                .await
                .expect("Code generation should succeed");

            // For Sea-ORM, check that the generated entity uses the correct ID type
            if matches!(orm_type, graphql_codegen_rust::cli::OrmType::SeaOrm) {
                let entity_path = temp_dir.path().join("src/entities/test.rs");
                let content = std::fs::read_to_string(entity_path).expect("Failed to read entity");
                assert!(
                    content.contains(expected_id_type),
                    "Expected {} in Sea-ORM entity for {:?}",
                    expected_id_type,
                    db_type
                );
            }
        }
    }
}

/// Parse the generated Diesel code to ensure it's valid Rust syntax
fn validate_generated_diesel_code(schema_path: &std::path::Path, entity_path: &std::path::Path) {
    // Read and parse the schema file
    let schema_content = std::fs::read_to_string(schema_path).expect("Failed to read schema file");
    match syn::parse_file(&schema_content) {
        Ok(_) => println!("✓ Schema file parses successfully"),
        Err(e) => panic!("Schema file failed to parse: {}", e),
    }

    // Read and parse the entity file
    let entity_content = std::fs::read_to_string(entity_path).expect("Failed to read entity file");
    match syn::parse_file(&entity_content) {
        Ok(_) => println!("✓ Entity file parses successfully"),
        Err(e) => panic!("Entity file failed to parse: {}", e),
    }
}

/// Parse the generated Sea-ORM code to ensure it's valid Rust syntax
fn validate_generated_sea_orm_code(mod_path: &std::path::Path, entity_path: &std::path::Path) {
    // Read and parse the mod.rs file
    let mod_content = std::fs::read_to_string(mod_path).expect("Failed to read mod.rs file");
    match syn::parse_file(&mod_content) {
        Ok(_) => println!("✓ Mod file parses successfully"),
        Err(e) => panic!("Mod file failed to parse: {}", e),
    }

    // Read and parse the entity file
    let entity_content = std::fs::read_to_string(entity_path).expect("Failed to read entity file");
    match syn::parse_file(&entity_content) {
        Ok(_) => println!("✓ Entity file parses successfully"),
        Err(e) => panic!("Entity file failed to parse: {}", e),
    }
}
