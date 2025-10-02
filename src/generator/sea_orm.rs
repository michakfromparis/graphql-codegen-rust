use std::collections::HashMap;

use crate::cli::DatabaseType;
use crate::config::Config;
use crate::generator::{
    CodeGenerator, MigrationFile, rust_type_for_field, sql_type_for_field, to_snake_case,
};
use crate::parser::{ParsedEnum, ParsedSchema, ParsedType};

pub struct SeaOrmGenerator;

impl SeaOrmGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SeaOrmGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for SeaOrmGenerator {
    fn generate_schema(&self, schema: &ParsedSchema, _config: &Config) -> anyhow::Result<String> {
        let mut output = String::new();

        // Add header comment
        output.push_str("//! Sea-ORM entities generated from GraphQL schema\n\n");

        // Generate module declarations for all entities
        for type_name in schema.types.keys() {
            let module_name = to_snake_case(type_name);
            output.push_str(&format!("pub mod {};\n", module_name));
        }

        // Generate module declarations for enums
        for enum_name in schema.enums.keys() {
            let module_name = to_snake_case(enum_name);
            output.push_str(&format!("pub mod {};\n", module_name));
        }

        output.push('\n');

        // Generate re-exports for convenience
        output.push_str("// Re-exports for convenience\n");
        for type_name in schema.types.keys() {
            let module_name = to_snake_case(type_name);
            output.push_str(&format!("pub use {}::Entity;\n", module_name));
            output.push_str(&format!("pub use {}::Model;\n", module_name));
            output.push_str(&format!("pub use {}::ActiveModel;\n", module_name));
            output.push_str(&format!("pub use {}::Column;\n", module_name));
        }

        // Re-export enums
        for enum_name in schema.enums.keys() {
            let module_name = to_snake_case(enum_name);
            output.push_str(&format!("pub use {}::{};\n", module_name, enum_name));
        }

        Ok(output)
    }

    fn generate_entities(
        &self,
        schema: &ParsedSchema,
        config: &Config,
    ) -> anyhow::Result<HashMap<String, String>> {
        let mut entities = HashMap::new();

        // Only generate entities for Object types (not interfaces or unions)
        for (type_name, parsed_type) in &schema.types {
            if matches!(parsed_type.kind, crate::parser::TypeKind::Object) {
                let entity_code = self.generate_entity_struct(type_name, parsed_type, config)?;
                entities.insert(format!("{}.rs", to_snake_case(type_name)), entity_code);
            }
        }

        // Generate enums
        for (enum_name, parsed_enum) in &schema.enums {
            let enum_code = self.generate_enum_type(enum_name, parsed_enum)?;
            entities.insert(format!("{}.rs", to_snake_case(enum_name)), enum_code);
        }

        Ok(entities)
    }

    fn generate_migrations(
        &self,
        schema: &ParsedSchema,
        config: &Config,
    ) -> anyhow::Result<Vec<MigrationFile>> {
        let mut migrations = Vec::new();

        // Only generate migrations for Object types (not interfaces or unions)
        for (type_name, parsed_type) in &schema.types {
            if matches!(parsed_type.kind, crate::parser::TypeKind::Object) {
                let migration = self.generate_table_migration(type_name, parsed_type, config)?;
                migrations.push(migration);
            }
        }

        Ok(migrations)
    }
}

impl SeaOrmGenerator {
    fn generate_entity_struct(
        &self,
        type_name: &str,
        parsed_type: &ParsedType,
        config: &Config,
    ) -> anyhow::Result<String> {
        let _struct_name = type_name.to_string();
        let table_name = to_snake_case(type_name);

        let mut output = String::new();

        // Add imports
        output.push_str("use sea_orm::entity::prelude::*;\n");
        output.push_str("use serde::{Deserialize, Serialize};\n\n");

        // Generate the entity struct
        output.push_str(
            "#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]\n",
        );
        output.push_str(&format!("#[sea_orm(table_name = \"{}\")]\n", table_name));
        output.push_str("pub struct Model {\n");

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

        // Generate PrimaryKey
        output.push_str("#[derive(Copy, Clone, Debug, EnumIter)]\n");
        output.push_str("pub enum PrimaryKey {\n");
        // Assume id is primary key
        output.push_str("    Id,\n");
        output.push_str("}\n\n");

        // Determine the ID type based on database
        let id_type = match config.db {
            DatabaseType::Sqlite => "i32",
            DatabaseType::Postgres => "uuid::Uuid",
            DatabaseType::Mysql => "u32",
        };

        let auto_increment = match config.db {
            DatabaseType::Sqlite => "true",
            DatabaseType::Postgres => "false", // UUIDs don't auto-increment
            DatabaseType::Mysql => "true",
        };

        output.push_str("impl PrimaryKeyTrait for PrimaryKey {\n");
        output.push_str(&format!("    type ValueType = {};\n", id_type));
        output.push_str("    fn auto_increment() -> bool {\n");
        output.push_str(&format!("        {}\n", auto_increment));
        output.push_str("    }\n");
        output.push_str("}\n\n");

        output.push_str("impl ActiveModelBehavior for ActiveModel {}\n\n");

        // Generate Entity constant (Sea-ORM convention)
        output.push_str("pub struct Entity;\n\n");
        output.push_str("impl EntityName for Entity {\n");
        output.push_str("    fn table_name(&self) -> &str {\n");
        output.push_str(&format!("        \"{}\"\n", table_name));
        output.push_str("    }\n");
        output.push_str("}\n\n");

        // Generate relationships based on detected foreign keys
        // For Sea-ORM, we can use derive macros and relationship definitions
        let mut has_relationships = false;

        for field in &parsed_type.fields {
            if field.name.ends_with("Id") && field.name.len() > 2 {
                let related_type = &field.name[..field.name.len() - 2];
                if related_type.chars().next().map_or(false, |c| c.is_uppercase()) {
                    if !has_relationships {
                        output.push_str("// Relationships\n");
                        has_relationships = true;
                    }
                    let _relation_name = to_snake_case(&field.name[..field.name.len() - 2]);
                    output.push_str(&format!("#[derive(Clone, Debug, PartialEq, DeriveRelation)]\n"));
                    output.push_str(&format!("#[sea_orm(table_name = \"{}\")]\n", table_name));
                    output.push_str(&format!("pub enum Relation {{\n"));
                    output.push_str(&format!("    #[sea_orm(\n"));
                    output.push_str(&format!("        belongs_to = \"super::{}::Entity\",\n", related_type));
                    output.push_str(&format!("        from = \"Column::{}\",\n", field.name));
                    output.push_str(&format!("        to = \"super::{}::Column::Id\",\n", related_type));
                    output.push_str(&format!("        on_update = \"Cascade\",\n"));
                    output.push_str(&format!("        on_delete = \"Cascade\"\n"));
                    output.push_str(&format!("    )]\n"));
                    output.push_str(&format!("    {},\n", related_type));
                    output.push_str(&format!("}}\n\n"));
                }
            }
        }

        Ok(output)
    }

    fn generate_enum_type(
        &self,
        enum_name: &str,
        parsed_enum: &ParsedEnum,
    ) -> anyhow::Result<String> {
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

    fn generate_table_migration(
        &self,
        type_name: &str,
        parsed_type: &ParsedType,
        config: &Config,
    ) -> anyhow::Result<MigrationFile> {
        let table_name = to_snake_case(type_name);
        let migration_name = format!(
            "m{}_create_{}_table",
            chrono::Utc::now().timestamp(),
            table_name
        );

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
            let primary_key = if field.name == "id" {
                " PRIMARY KEY"
            } else {
                ""
            };

            columns.push(format!(
                "    {} {}{}{}",
                column_name, sql_type, nullable, primary_key
            ));
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
