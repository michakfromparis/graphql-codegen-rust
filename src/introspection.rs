use reqwest::header::{HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize)]
struct IntrospectionQuery {
    query: String,
}

#[derive(Debug, Deserialize)]
struct IntrospectionResponse {
    data: Option<IntrospectionData>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct IntrospectionData {
    #[serde(rename = "__schema")]
    schema: Schema,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Schema {
    pub query_type: Option<TypeRef>,
    pub mutation_type: Option<TypeRef>,
    pub subscription_type: Option<TypeRef>,
    pub types: Vec<Type>,
    pub directives: Vec<Directive>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Type {
    pub name: Option<String>,
    pub kind: TypeKind,
    pub description: Option<String>,
    pub fields: Option<Vec<Field>>,
    pub interfaces: Option<Vec<TypeRef>>,
    pub possible_types: Option<Vec<TypeRef>>,
    pub enum_values: Option<Vec<EnumValue>>,
    pub input_fields: Option<Vec<InputValue>>,
    pub of_type: Option<Box<TypeRef>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeKind {
    Scalar,
    Object,
    Interface,
    Union,
    Enum,
    InputObject,
    List,
    NonNull,
}

#[derive(Debug, Deserialize)]
pub struct TypeRef {
    pub name: Option<String>,
    pub kind: Option<TypeKind>,
    pub of_type: Option<Box<TypeRef>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Field {
    pub name: String,
    pub description: Option<String>,
    pub args: Vec<InputValue>,
    pub type_: TypeRef,
    #[serde(rename = "isDeprecated")]
    pub is_deprecated: bool,
    pub deprecation_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct InputValue {
    pub name: String,
    pub description: Option<String>,
    pub type_: TypeRef,
    pub default_value: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct EnumValue {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "isDeprecated")]
    pub is_deprecated: bool,
    pub deprecation_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Directive {
    pub name: String,
    pub description: Option<String>,
    pub locations: Vec<DirectiveLocation>,
    pub args: Vec<InputValue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectiveLocation {
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    VariableDefinition,
    Schema,
    Scalar,
    Object,
    FieldDefinition,
    ArgumentDefinition,
    Interface,
    Union,
    Enum,
    EnumValue,
    InputObject,
    InputFieldDefinition,
}

pub struct Introspector {
    client: reqwest::Client,
}

#[allow(dead_code)]
impl Default for Introspector {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl Introspector {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn introspect_schema(
        &self,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> anyhow::Result<Schema> {
        let introspection_query = r#"
            query IntrospectionQuery {
                __schema {
                    queryType { name }
                    mutationType { name }
                    subscriptionType { name }
                    types {
                        ...FullType
                    }
                    directives {
                        name
                        description
                        locations
                        args {
                            ...InputValue
                        }
                    }
                }
            }

            fragment FullType on __Type {
                kind
                name
                description
                fields(includeDeprecated: true) {
                    name
                    description
                    args {
                        ...InputValue
                    }
                    type {
                        ...TypeRef
                    }
                    isDeprecated
                    deprecationReason
                }
                inputFields {
                    ...InputValue
                }
                interfaces {
                    ...TypeRef
                }
                enumValues(includeDeprecated: true) {
                    name
                    description
                    isDeprecated
                    deprecationReason
                }
                possibleTypes {
                    ...TypeRef
                }
            }

            fragment InputValue on __InputValue {
                name
                description
                type {
                    ...TypeRef
                }
                defaultValue
            }

            fragment TypeRef on __Type {
                kind
                name
                ofType {
                    kind
                    name
                    ofType {
                        kind
                        name
                        ofType {
                            kind
                            name
                            ofType {
                                kind
                                name
                                ofType {
                                    kind
                                    name
                                    ofType {
                                        kind
                                        name
                                        ofType {
                                            kind
                                            name
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        "#;

        let query = IntrospectionQuery {
            query: introspection_query.to_string(),
        };

        let mut request = self.client.post(url).json(&query);

        // Add custom headers
        for (key, value) in headers {
            let header_name = HeaderName::from_bytes(key.as_bytes())?;
            let header_value = HeaderValue::from_str(value)?;
            request = request.header(header_name, header_value);
        }

        let response = request.send().await?;
        let status = response.status();

        if !status.is_success() {
            let status_code = status.as_u16();
            let error_msg = match status_code {
                400 => "Bad Request - The GraphQL query may be malformed",
                401 => "Unauthorized - Authentication required. Check your headers",
                403 => "Forbidden - Access denied. Verify your credentials and permissions",
                404 => "Not Found - GraphQL endpoint not found at the specified URL",
                500 => "Internal Server Error - The GraphQL server encountered an error",
                _ => "HTTP request failed",
            };

            return Err(anyhow::anyhow!(
                "GraphQL introspection failed with HTTP {}: {}\nURL: {}\n\nTroubleshooting:\n- Verify the URL is correct and accessible\n- Check authentication headers if required\n- Ensure the server supports GraphQL introspection",
                status_code, error_msg, url
            ));
        }

        let introspection_response: IntrospectionResponse = response.json().await?;

        if let Some(errors) = introspection_response.errors {
            let error_messages: Vec<String> = errors.into_iter().map(|e| e.message).collect();
            let error_count = error_messages.len();

            let mut error_text = format!(
                "GraphQL introspection failed with {} error{}:\n",
                error_count,
                if error_count == 1 { "" } else { "s" }
            );

            for (i, message) in error_messages.iter().enumerate() {
                error_text.push_str(&format!("{}. {}\n", i + 1, message));
            }

            error_text.push_str("\nCommon causes:\n");
            error_text.push_str("- Introspection is disabled on the GraphQL server\n");
            error_text.push_str("- Authentication or authorization issues\n");
            error_text.push_str("- Server-side GraphQL schema errors\n");
            error_text.push_str("- Network connectivity problems\n");

            return Err(anyhow::anyhow!(error_text));
        }

        let schema = introspection_response
            .data
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No data returned from GraphQL introspection\n\nThis typically indicates:\n- The GraphQL endpoint returned an empty response\n- The server may not support the introspection query\n- Network issues prevented a complete response\n\nTry:\n- Checking if the endpoint supports GraphQL introspection\n- Verifying network connectivity\n- Testing with a simple GraphQL query first"
                )
            })?
            .schema;

        Ok(schema)
    }

    fn object_type_to_sdl(&self, type_def: &Type) -> String {
        let mut sdl = String::new();

        if let Some(description) = &type_def.description {
            sdl.push_str(&format!("\"\"\"\n{}\n\"\"\"\n", description));
        }

        let name = type_def.name.as_ref().unwrap();
        sdl.push_str(&format!("type {} ", name));

        // Add interfaces
        if let Some(interfaces) = &type_def.interfaces {
            if !interfaces.is_empty() {
                let interface_names: Vec<String> =
                    interfaces.iter().filter_map(|i| i.name.clone()).collect();
                sdl.push_str(&format!("implements {} ", interface_names.join(" & ")));
            }
        }

        sdl.push_str("{\n");

        if let Some(fields) = &type_def.fields {
            for field in fields {
                if let Some(description) = &field.description {
                    sdl.push_str(&format!("  \"\"\"\n  {}\n  \"\"\"\n", description));
                }
                sdl.push_str(&format!(
                    "  {}: {}\n",
                    field.name,
                    self.type_ref_to_sdl(&field.type_)
                ));
            }
        }

        sdl.push_str("}\n\n");
        sdl
    }

    fn interface_type_to_sdl(&self, type_def: &Type) -> String {
        let mut sdl = String::new();

        if let Some(description) = &type_def.description {
            sdl.push_str(&format!("\"\"\"\n{}\n\"\"\"\n", description));
        }

        let name = type_def.name.as_ref().unwrap();
        sdl.push_str(&format!("interface {} {{\n", name));

        if let Some(fields) = &type_def.fields {
            for field in fields {
                if let Some(description) = &field.description {
                    sdl.push_str(&format!("  \"\"\"\n  {}\n  \"\"\"\n", description));
                }
                sdl.push_str(&format!(
                    "  {}: {}\n",
                    field.name,
                    self.type_ref_to_sdl(&field.type_)
                ));
            }
        }

        sdl.push_str("}\n\n");
        sdl
    }

    fn enum_type_to_sdl(&self, type_def: &Type) -> String {
        let mut sdl = String::new();

        if let Some(description) = &type_def.description {
            sdl.push_str(&format!("\"\"\"\n{}\n\"\"\"\n", description));
        }

        let name = type_def.name.as_ref().unwrap();
        sdl.push_str(&format!("enum {} {{\n", name));

        if let Some(values) = &type_def.enum_values {
            for value in values {
                if let Some(description) = &value.description {
                    sdl.push_str(&format!("  \"\"\"\n  {}\n  \"\"\"\n", description));
                }
                sdl.push_str(&format!("  {}\n", value.name));
            }
        }

        sdl.push_str("}\n\n");
        sdl
    }

    fn input_object_type_to_sdl(&self, type_def: &Type) -> String {
        let mut sdl = String::new();

        if let Some(description) = &type_def.description {
            sdl.push_str(&format!("\"\"\"\n{}\n\"\"\"\n", description));
        }

        let name = type_def.name.as_ref().unwrap();
        sdl.push_str(&format!("input {} {{\n", name));

        if let Some(fields) = &type_def.input_fields {
            for field in fields {
                if let Some(description) = &field.description {
                    sdl.push_str(&format!("  \"\"\"\n  {}\n  \"\"\"\n", description));
                }
                let type_str = self.type_ref_to_sdl(&field.type_);
                let default_value = field
                    .default_value
                    .as_ref()
                    .map(|v| format!(" = {}", v))
                    .unwrap_or_default();
                sdl.push_str(&format!(
                    "  {}: {}{}\n",
                    field.name, type_str, default_value
                ));
            }
        }

        sdl.push_str("}\n\n");
        sdl
    }

    fn scalar_type_to_sdl(&self, type_def: &Type) -> String {
        let mut sdl = String::new();

        if let Some(description) = &type_def.description {
            sdl.push_str(&format!("\"\"\"\n{}\n\"\"\"\n", description));
        }

        let name = type_def.name.as_ref().unwrap();
        sdl.push_str(&format!("scalar {}\n\n", name));
        sdl
    }

    fn union_type_to_sdl(&self, type_def: &Type) -> String {
        let mut sdl = String::new();

        if let Some(description) = &type_def.description {
            sdl.push_str(&format!("\"\"\"\n{}\n\"\"\"\n", description));
        }

        let name = type_def.name.as_ref().unwrap();
        sdl.push_str(&format!("union {} = ", name));

        if let Some(possible_types) = &type_def.possible_types {
            let type_names: Vec<String> = possible_types
                .iter()
                .filter_map(|t| t.name.clone())
                .collect();
            sdl.push_str(&type_names.join(" | "));
        }

        sdl.push_str("\n\n");
        sdl
    }

    #[allow(clippy::only_used_in_recursion)]
    fn type_ref_to_sdl(&self, type_ref: &TypeRef) -> String {
        let mut result = String::new();

        // Handle NonNull and List wrappers
        match type_ref.kind {
            Some(TypeKind::NonNull) => {
                if let Some(of_type) = &type_ref.of_type {
                    result.push_str(&self.type_ref_to_sdl(of_type));
                    result.push('!');
                }
            }
            Some(TypeKind::List) => {
                if let Some(of_type) = &type_ref.of_type {
                    result.push('[');
                    result.push_str(&self.type_ref_to_sdl(of_type));
                    result.push(']');
                }
            }
            _ => {
                if let Some(name) = &type_ref.name {
                    result.push_str(name);
                }
            }
        }

        result
    }

    /// Convert introspection schema to SDL string
    pub fn schema_to_sdl(&self, schema: &Schema) -> String {
        let mut sdl = String::new();

        // Add schema definition
        sdl.push_str("schema {\n");
        if let Some(query) = &schema.query_type {
            if let Some(name) = &query.name {
                sdl.push_str(&format!("  query: {}\n", name));
            }
        }
        if let Some(mutation) = &schema.mutation_type {
            if let Some(name) = &mutation.name {
                sdl.push_str(&format!("  mutation: {}\n", name));
            }
        }
        if let Some(subscription) = &schema.subscription_type {
            if let Some(name) = &subscription.name {
                sdl.push_str(&format!("  subscription: {}\n", name));
            }
        }
        sdl.push_str("}\n\n");

        // Add types
        for type_def in &schema.types {
            if let Some(name) = &type_def.name {
                // Skip introspection types
                if name.starts_with("__") {
                    continue;
                }

                match type_def.kind {
                    TypeKind::Object => {
                        sdl.push_str(&self.object_type_to_sdl(type_def));
                    }
                    TypeKind::Interface => {
                        sdl.push_str(&self.interface_type_to_sdl(type_def));
                    }
                    TypeKind::Enum => {
                        sdl.push_str(&self.enum_type_to_sdl(type_def));
                    }
                    TypeKind::InputObject => {
                        sdl.push_str(&self.input_object_type_to_sdl(type_def));
                    }
                    TypeKind::Scalar => {
                        sdl.push_str(&self.scalar_type_to_sdl(type_def));
                    }
                    TypeKind::Union => {
                        sdl.push_str(&self.union_type_to_sdl(type_def));
                    }
                    _ => {} // Skip List, NonNull as they're handled in type refs
                }
            }
        }

        sdl
    }
}
