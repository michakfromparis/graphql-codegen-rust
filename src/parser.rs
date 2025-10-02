use std::collections::HashMap;

use crate::introspection::{Introspector, Schema as IntrospectionSchema};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParsedSchema {
    pub types: HashMap<String, ParsedType>,
    pub enums: HashMap<String, ParsedEnum>,
    pub scalars: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Object,
    Interface,
    Union,
}

#[derive(Debug, Clone)]
pub struct ParsedType {
    #[allow(dead_code)]
    pub name: String,
    pub kind: TypeKind,
    pub fields: Vec<ParsedField>,
    #[allow(dead_code)]
    pub description: Option<String>,
    #[allow(dead_code)]
    pub interfaces: Vec<String>, // For objects and interfaces: implemented interfaces
    #[allow(dead_code)]
    pub union_members: Vec<String>, // For unions: member types
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParsedField {
    pub name: String,
    pub field_type: FieldType,
    pub description: Option<String>,
    pub is_nullable: bool,
    pub is_list: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FieldType {
    Scalar(String),
    Reference(String),
    Enum(String),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ParsedEnum {
    pub name: String,
    pub values: Vec<String>,
    pub description: Option<String>,
}

pub struct GraphQLParser {
    introspector: Introspector,
}

#[allow(dead_code)]
impl Default for GraphQLParser {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl GraphQLParser {
    pub fn new() -> Self {
        Self {
            introspector: Introspector::new(),
        }
    }

    /// Parse schema from introspection
    pub async fn parse_from_introspection(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> anyhow::Result<ParsedSchema> {
        let schema = self.introspector.introspect_schema(url, headers).await?;
        self.parse_schema(schema)
    }

    /// Parse schema from SDL string
    pub fn parse_from_sdl(&self, sdl: &str) -> anyhow::Result<ParsedSchema> {
        use graphql_parser::parse_schema;

        let document =
            parse_schema(sdl).map_err(|e| anyhow::anyhow!("Failed to parse SDL: {}", e))?;

        self.parse_sdl_document(document)
    }

    /// Parse schema from SDL string (simplified version)
    pub fn parse_from_sdl_simple(&self, sdl: &str) -> anyhow::Result<ParsedSchema> {
        // Alias for parse_from_sdl for backward compatibility
        self.parse_from_sdl(sdl)
    }

    fn parse_sdl_document<'a>(
        &self,
        document: graphql_parser::schema::Document<'a, &'a str>,
    ) -> anyhow::Result<ParsedSchema> {
        let mut types = HashMap::new();
        let mut enums = HashMap::new();
        let mut scalars = Vec::new();

        for definition in document.definitions {
            match definition {
                graphql_parser::schema::Definition::TypeDefinition(type_def) => {
                    match type_def {
                        graphql_parser::schema::TypeDefinition::Object(obj) => {
                            if let Some(parsed_type) = self.parse_sdl_object_type(&obj) {
                                types.insert(obj.name.to_string(), parsed_type);
                            }
                        }
                        graphql_parser::schema::TypeDefinition::Enum(enum_def) => {
                            if let Some(parsed_enum) = self.parse_sdl_enum_type(&enum_def) {
                                enums.insert(enum_def.name.to_string(), parsed_enum);
                            }
                        }
                        graphql_parser::schema::TypeDefinition::Scalar(scalar) => {
                            scalars.push(scalar.name.to_string());
                        }
                        graphql_parser::schema::TypeDefinition::Interface(interface) => {
                            if let Some(parsed_type) = self.parse_sdl_interface_type(&interface) {
                                types.insert(interface.name.to_string(), parsed_type);
                            }
                        }
                        graphql_parser::schema::TypeDefinition::Union(union_def) => {
                            if let Some(parsed_type) = self.parse_sdl_union_type(&union_def) {
                                types.insert(union_def.name.to_string(), parsed_type);
                            }
                        }
                        graphql_parser::schema::TypeDefinition::InputObject(_) => {
                            // Skip input objects for now - they don't affect ORM generation
                        }
                    }
                }
                graphql_parser::schema::Definition::SchemaDefinition(_)
                | graphql_parser::schema::Definition::DirectiveDefinition(_) => {
                    // Skip schema and directive definitions for ORM generation
                }
                graphql_parser::schema::Definition::TypeExtension(_) => {
                    // Skip type extensions for now
                }
            }
        }

        Ok(ParsedSchema {
            types,
            enums,
            scalars,
        })
    }

    fn parse_schema(&self, schema: IntrospectionSchema) -> anyhow::Result<ParsedSchema> {
        let mut types = HashMap::new();
        let mut enums = HashMap::new();
        let mut scalars = Vec::new();

        for type_def in schema.types {
            if let Some(name) = &type_def.name {
                // Skip introspection types and built-in scalars
                if name.starts_with("__")
                    || name == "String"
                    || name == "Int"
                    || name == "Float"
                    || name == "Boolean"
                    || name == "ID"
                {
                    if matches!(type_def.kind, crate::introspection::TypeKind::Scalar)
                        && !name.starts_with("__")
                    {
                        scalars.push(name.clone());
                    }
                    continue;
                }

                match type_def.kind {
                    crate::introspection::TypeKind::Object => {
                        if let Some(parsed_type) = self.parse_object_type(&type_def) {
                            types.insert(name.clone(), parsed_type);
                        }
                    }
                    crate::introspection::TypeKind::Interface => {
                        if let Some(parsed_type) = self.parse_interface_type(&type_def) {
                            types.insert(name.clone(), parsed_type);
                        }
                    }
                    crate::introspection::TypeKind::Union => {
                        if let Some(parsed_type) = self.parse_union_type(&type_def) {
                            types.insert(name.clone(), parsed_type);
                        }
                    }
                    crate::introspection::TypeKind::Enum => {
                        if let Some(parsed_enum) = self.parse_enum_type(&type_def) {
                            enums.insert(name.clone(), parsed_enum);
                        }
                    }
                    crate::introspection::TypeKind::Scalar => {
                        scalars.push(name.clone());
                    }
                    _ => {
                        // Skip input objects and other types for ORM generation
                    }
                }
            }
        }

        Ok(ParsedSchema {
            types,
            enums,
            scalars,
        })
    }

    // fn parse_document is removed for now - focusing on introspection
    // TODO: Re-implement SDL parsing when needed

    fn parse_object_type(&self, type_def: &crate::introspection::Type) -> Option<ParsedType> {
        let name = type_def.name.as_ref()?;
        let mut fields = Vec::new();

        if let Some(introspection_fields) = &type_def.fields {
            for field in introspection_fields {
                if let Some(parsed_field) = self.parse_field(field) {
                    fields.push(parsed_field);
                }
            }
        }

        let interfaces = type_def
            .interfaces
            .as_ref()
            .map(|interfaces| interfaces.iter().filter_map(|i| i.name.clone()).collect())
            .unwrap_or_default();

        Some(ParsedType {
            name: name.clone(),
            kind: TypeKind::Object,
            fields,
            description: type_def.description.clone(),
            interfaces,
            union_members: vec![],
        })
    }

    fn parse_interface_type(&self, type_def: &crate::introspection::Type) -> Option<ParsedType> {
        let name = type_def.name.as_ref()?;
        let mut fields = Vec::new();

        if let Some(type_fields) = &type_def.fields {
            for field in type_fields {
                if let Some(parsed_field) = self.parse_field(field) {
                    fields.push(parsed_field);
                }
            }
        }

        let interfaces = type_def
            .interfaces
            .as_ref()
            .map(|interfaces| interfaces.iter().filter_map(|i| i.name.clone()).collect())
            .unwrap_or_default();

        Some(ParsedType {
            name: name.clone(),
            kind: TypeKind::Interface,
            fields,
            description: type_def.description.clone(),
            interfaces,
            union_members: vec![],
        })
    }

    fn parse_union_type(&self, type_def: &crate::introspection::Type) -> Option<ParsedType> {
        let name = type_def.name.as_ref()?;

        // For unions, get the possible types (union members)
        let union_members = type_def
            .possible_types
            .as_ref()
            .map(|types| types.iter().filter_map(|t| t.name.clone()).collect())
            .unwrap_or_default();

        Some(ParsedType {
            name: name.clone(),
            kind: TypeKind::Union,
            fields: vec![], // Union types don't have fields
            description: type_def.description.clone(),
            interfaces: vec![],
            union_members,
        })
    }

    fn parse_field(&self, field: &crate::introspection::Field) -> Option<ParsedField> {
        let (field_type, is_nullable, is_list) = self.parse_type_ref(&field.type_)?;

        Some(ParsedField {
            name: field.name.clone(),
            field_type,
            description: field.description.clone(),
            is_nullable,
            is_list,
        })
    }

    #[allow(clippy::only_used_in_recursion)]
    fn parse_type_ref(
        &self,
        type_ref: &crate::introspection::TypeRef,
    ) -> Option<(FieldType, bool, bool)> {
        match type_ref.kind {
            Some(crate::introspection::TypeKind::NonNull) => {
                if let Some(of_type) = &type_ref.of_type {
                    let (field_type, _, is_list) = self.parse_type_ref(of_type)?;
                    Some((field_type, false, is_list))
                } else {
                    None
                }
            }
            Some(crate::introspection::TypeKind::List) => {
                if let Some(of_type) = &type_ref.of_type {
                    let (field_type, is_nullable, _) = self.parse_type_ref(of_type)?;
                    Some((field_type, is_nullable, true))
                } else {
                    None
                }
            }
            _ => {
                if let Some(name) = &type_ref.name {
                    let field_type = match name.as_str() {
                        "String" | "Int" | "Float" | "Boolean" => FieldType::Scalar(name.clone()),
                        "ID" => FieldType::Scalar("ID".to_string()),
                        _ => {
                            // Check if it's an enum (this is a simplification)
                            // In a real implementation, we'd need to check the schema
                            FieldType::Reference(name.clone())
                        }
                    };
                    Some((field_type, true, false))
                } else {
                    None
                }
            }
        }
    }

    fn parse_enum_type(&self, type_def: &crate::introspection::Type) -> Option<ParsedEnum> {
        let name = type_def.name.as_ref()?;
        let mut values = Vec::new();

        if let Some(enum_values) = &type_def.enum_values {
            for value in enum_values {
                values.push(value.name.clone());
            }
        }

        Some(ParsedEnum {
            name: name.clone(),
            values,
            description: type_def.description.clone(),
        })
    }

    // SDL parsing helper methods
    fn parse_sdl_object_type<'a>(
        &self,
        obj: &graphql_parser::schema::ObjectType<'a, &'a str>,
    ) -> Option<ParsedType> {
        let mut fields = Vec::new();

        for field in &obj.fields {
            if let Some(parsed_field) = self.parse_sdl_field(field) {
                fields.push(parsed_field);
            }
        }

        let interfaces = obj
            .implements_interfaces
            .iter()
            .map(|name| name.to_string())
            .collect();

        Some(ParsedType {
            name: obj.name.to_string(),
            kind: TypeKind::Object,
            fields,
            description: obj.description.as_ref().map(|s| s.to_string()),
            interfaces,
            union_members: vec![],
        })
    }

    fn parse_sdl_interface_type<'a>(
        &self,
        interface: &graphql_parser::schema::InterfaceType<'a, &'a str>,
    ) -> Option<ParsedType> {
        let mut fields = Vec::new();

        for field in &interface.fields {
            if let Some(parsed_field) = self.parse_sdl_field(field) {
                fields.push(parsed_field);
            }
        }

        let interfaces = interface
            .implements_interfaces
            .iter()
            .map(|name| name.to_string())
            .collect();

        Some(ParsedType {
            name: interface.name.to_string(),
            kind: TypeKind::Interface,
            fields,
            description: interface.description.as_ref().map(|s| s.to_string()),
            interfaces,
            union_members: vec![],
        })
    }

    fn parse_sdl_union_type<'a>(
        &self,
        union_def: &graphql_parser::schema::UnionType<'a, &'a str>,
    ) -> Option<ParsedType> {
        // For unions, we store union members separately
        let union_members = union_def
            .types
            .iter()
            .map(|name| name.to_string())
            .collect();

        Some(ParsedType {
            name: union_def.name.to_string(),
            kind: TypeKind::Union,
            fields: vec![], // Union types don't have fields in GraphQL
            description: union_def.description.as_ref().map(|s| s.to_string()),
            interfaces: vec![],
            union_members,
        })
    }

    fn parse_sdl_enum_type<'a>(
        &self,
        enum_def: &graphql_parser::schema::EnumType<'a, &'a str>,
    ) -> Option<ParsedEnum> {
        let values = enum_def
            .values
            .iter()
            .map(|value| value.name.to_string())
            .collect();

        Some(ParsedEnum {
            name: enum_def.name.to_string(),
            values,
            description: enum_def.description.as_ref().map(|s| s.to_string()),
        })
    }

    fn parse_sdl_field<'a>(
        &self,
        field: &graphql_parser::schema::Field<'a, &'a str>,
    ) -> Option<ParsedField> {
        let (field_type, is_nullable, is_list) = self.parse_sdl_type(&field.field_type)?;

        Some(ParsedField {
            name: field.name.to_string(),
            field_type,
            description: field.description.as_ref().map(|s| s.to_string()),
            is_nullable,
            is_list,
        })
    }

    #[allow(clippy::only_used_in_recursion)]
    fn parse_sdl_type<'a>(
        &self,
        field_type: &graphql_parser::schema::Type<'a, &'a str>,
    ) -> Option<(FieldType, bool, bool)> {
        match field_type {
            graphql_parser::schema::Type::NamedType(name) => {
                let field_type = match *name {
                    "ID" | "String" | "Int" | "Float" | "Boolean" => {
                        FieldType::Scalar(name.to_string())
                    }
                    _ => FieldType::Reference(name.to_string()),
                };
                Some((field_type, true, false)) // Named types are nullable by default
            }
            graphql_parser::schema::Type::ListType(inner_type) => {
                if let Some((inner_field_type, _, _)) = self.parse_sdl_type(inner_type) {
                    Some((inner_field_type, true, true))
                } else {
                    None
                }
            }
            graphql_parser::schema::Type::NonNullType(inner_type) => {
                if let Some((inner_field_type, _, is_list)) = self.parse_sdl_type(inner_type) {
                    Some((inner_field_type, false, is_list))
                } else {
                    None
                }
            }
        }
    }
}
