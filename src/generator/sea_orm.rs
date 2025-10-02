use std::collections::HashMap;

use crate::cli::DatabaseType;
use crate::config::Config;
use crate::generator::{sql_type_for_field, to_snake_case, CodeGenerator, MigrationFile, rust_type_for_field};
use crate::parser::{ParsedEnum, ParsedField, ParsedSchema, ParsedType};

pub struct SeaOrmGenerator;

impl SeaOrmGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl CodeGenerator for SeaOrmGenerator {
    fn generate_schema(&self, schema: &ParsedSchema, config: &Config) -> anyhow::Result<String> {
        // Sea-ORM doesn't have a schema.rs equivalent like Diesel
        // Return empty string or basic setup
        Ok("// Sea-ORM entities are defined in separate files\n".to_string())
    }

    fn generate_entities(&self, schema: &ParsedSchema, config: &Config) -> anyhow::Result<HashMap<String, String>> {
        let mut entities = HashMap::new();

        for (type_name, parsed_type) in &schema.types {
            let entity_code = self.generate_entity_struct(type_name, parsed_type, config)?;
            entities.insert(format!("{}.rs", to_snake_case(type_name)), entity_code);
        }

        // Generate enums
        for (enum_name, parsed_enum) in &schema.enums {
            let enum_code = self.generate_enum_type(enum_name, parsed_enum)?;
            entities.insert(format!("{}.rs", to_snake_case(enum_name)), enum_code);
        }

        Ok(entities)
    }

    fn generate_migrations(&self, schema: &ParsedSchema, config: &Config) -> anyhow::Result<Vec<MigrationFile>> {
        let mut migrations = Vec::new();

        for (type_name, parsed_type) in &schema.types {
            let migration = self.generate_table_migration(type_name, parsed_type, config)?;
            migrations.push(migration);
        }

        Ok(migrations)
    }
}

impl SeaOrmGenerator {
    fn generate_entity_struct(&self, type_name: &str, parsed_type: &ParsedType, config: &Config) -> anyhow::Result<String> {
        let struct_name = type_name.to_string();
        let table_name = to_snake_case(type_name);

        let mut output = String::new();

        // Add imports
        output.push_str("use sea_orm::entity::prelude::*;\n");
        output.push_str("use serde::{Deserialize, Serialize};\n\n");

        // Generate the entity struct
        output.push_str("#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]\n");
        output.push_str(&format!("#[sea_orm(table_name = \"{}\")]\n", table_name));
        output.push_str(&format!("pub struct Model {{\n"));

        for field in &parsed_type.fields {
            let field_name = to_snake_case(&field.name);
            let field_type = rust_type_for_field(field, &config.db, &config.type_mappings);
            let column_attr = format!("#[sea_orm(column_name = \"{}\")]", field_name);

            output.push_str(&format!("    {}\n", column_attr));
            output.push_str(&format!("    pub {}: {},\n", field_name, field_type));
        }

        output.push_str("}\n\n");

        // Generate relation enum (empty for now)
        output.push_str("#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]\n");
        output.push_str("pub enum Relation {}\n\n");

        // Generate ActiveModel
        output.push_str("#[derive(Copy, Clone, Debug, EnumIter, DeriveCustomColumn)]\n");
        output.push_str("pub enum Column {\n");
        for field in &parsed_type.fields {
            let field_name = to_snake_case(&field.name);
            output.push_str(&format!("    {},\n", field_name));
        }
        output.push_str("}\n\n");

        // Generate ActiveModelBehavior
        output.push_str("#[derive(Copy, Clone, Debug, EnumIter)]\n");
        output.push_str("pub enum PrimaryKey {\n");
        // Assume id is primary key
        output.push_str("    Id,\n");
        output.push_str("}\n\n");

        output.push_str("impl PrimaryKeyTrait for PrimaryKey {\n");
        output.push_str("    type ValueType = i32;\n");
        output.push_str("    fn auto_increment() -> bool {\n");
        output.push_str("        true\n");
        output.push_str("    }\n");
        output.push_str("}\n\n");

        output.push_str("impl ActiveModelBehavior for ActiveModel {}\n");

        Ok(output)
    }

    fn generate_enum_type(&self, enum_name: &str, parsed_enum: &ParsedEnum) -> anyhow::Result<String> {
        let mut output = String::new();

        if let Some(description) = &parsed_enum.description {
            output.push_str(&format!("/// {}\n", description));
        }

        output.push_str("#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]\n");
        output.push_str("#[sea_orm(rs_type = \"String\", db_type = \"String(Some(1))\")]\n");
        output.push_str(&format!("pub enum {} {{\n", enum_name));

        for value in &parsed_enum.values {
            output.push_str(&format!("    #[sea_orm(string_value = \"{}\")]\n", value));
            output.push_str(&format!("    {},\n", value));
        }

        output.push_str("}\n");

        Ok(output)
    }

    fn generate_table_migration(&self, type_name: &str, parsed_type: &ParsedType, config: &Config) -> anyhow::Result<MigrationFile> {
        let table_name = to_snake_case(type_name);
        let migration_name = format!("m{}_create_{}_table", chrono::Utc::now().timestamp(), table_name);

        let mut up_sql = format!("CREATE TABLE {} (\n", table_name);

        let mut columns = Vec::new();

        // Add id column if not present
        let has_id = parsed_type.fields.iter().any(|f| f.name == "id");
        if !has_id {
            let id_type = match config.db {
                DatabaseType::Sqlite => "INTEGER PRIMARY KEY AUTOINCREMENT",
                DatabaseType::Postgres => "UUID PRIMARY KEY DEFAULT gen_random_uuid()",
                DatabaseType::Mysql => "INT UNSIGNED PRIMARY KEY AUTO_INCREMENT",
            };
            columns.push(format!("    id {}", id_type));
        }

        for field in &parsed_type.fields {
            let column_name = to_snake_case(&field.name);
            let sql_type = sql_type_for_field(field, &config.db, &config.type_mappings);

            let nullable = if field.is_nullable { "" } else { " NOT NULL" };
            let primary_key = if field.name == "id" { " PRIMARY KEY" } else { "" };

            columns.push(format!("    {} {}{}{}", column_name, sql_type, nullable, primary_key));
        }

        up_sql.push_str(&columns.join(",\n"));
        up_sql.push_str("\n);");

        let down_sql = format!("DROP TABLE {};", table_name);

        Ok(MigrationFile {
            name: migration_name,
            up_sql,
            down_sql,
        })
    }
}
