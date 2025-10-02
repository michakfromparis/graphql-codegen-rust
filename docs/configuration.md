# ⚙️ Configuration

GraphQL Rust Codegen supports flexible configuration through TOML and YAML files. YAML configs are fully compatible with [GraphQL Code Generator](https://the-guild.dev/graphql/codegen), making it easy to share configurations across your stack.

## 📄 Configuration Files

The tool automatically detects these files (in order of preference):
1. `codegen.yml` or `codegen.yaml`
2. `graphql-codegen-rust.toml`
3. `.graphqlrc.yml`
4. Custom path via `--config` flag

## 🔧 YAML Configuration (Recommended)

Perfect for Tauri apps and teams using GraphQL Code Generator:

```yaml
# GraphQL schema source
schema: https://api.example.com/graphql
# Or use local SDL files
# schema: ./schema.graphql

# Optional: authentication headers
headers:
  Authorization: "Bearer your-token-here"
  X-API-Key: "your-api-key"

# Rust codegen configuration
rust_codegen:
  # ORM selection
  orm: diesel          # or "sea_orm"

  # Database type
  db: sqlite           # or "postgres", "mysql"

  # Output configuration
  output_dir: ./src/db
  generate_migrations: true
  generate_entities: true

  # Naming conventions
  table_naming: snake_case  # or "camelCase", "pascalCase"

  # Custom scalar type mappings
  type_mappings:
    DateTime: "chrono::NaiveDateTime"
    UUID: "uuid::Uuid"
    JSON: "serde_json::Value"
    BigInt: "i64"
```

## 📋 TOML Configuration

For Rust-native projects:

```toml
# GraphQL schema source
url = "https://api.example.com/graphql"
# Or local SDL file
# sdl_file = "./schema.graphql"

# ORM and database
orm = "Diesel"        # or "SeaOrm"
db = "Sqlite"         # or "Postgres", "Mysql"

# Output settings
output_dir = "./generated"
generate_migrations = true
generate_entities = true
table_naming = "snake_case"

# Authentication
[headers]
Authorization = "Bearer your-token-here"
X-API-Key = "your-api-key"

# Custom type mappings
[type_mappings]
DateTime = "chrono::NaiveDateTime"
UUID = "uuid::Uuid"
JSON = "serde_json::Value"
BigInt = "i64"
```

## 🎯 Configuration Options

### Schema Sources

| Option | Description | Example |
|--------|-------------|---------|
| `schema` (YAML) | GraphQL endpoint URL | `https://api.example.com/graphql` |
| `url` (TOML) | GraphQL endpoint URL | `"https://api.example.com/graphql"` |
| `sdl_file` | Local SDL schema file | `"./schema.graphql"` |

### ORM Selection

| ORM | Description | Best For |
|-----|-------------|----------|
| `diesel` | Mature, battle-tested ORM | Production applications |
| `sea_orm` | Async-first, modern ORM | New async projects |

### Database Types

| Database | Rust Type (ID) | Notes |
|----------|----------------|--------|
| `sqlite` | `i32` | Fast, embedded, no setup |
| `postgres` | `uuid::Uuid` | Advanced features, production |
| `mysql` | `u32` | High performance, legacy systems |

### Naming Conventions

```yaml
# Table naming
table_naming: snake_case  # users, user_posts
table_naming: camelCase   # users, userPosts
table_naming: pascalCase  # Users, UserPosts
```

### Custom Type Mappings

Map GraphQL scalars to your preferred Rust types:

```yaml
type_mappings:
  # Temporal types
  DateTime: "chrono::DateTime<chrono::Utc>"
  Date: "chrono::NaiveDate"
  Time: "chrono::NaiveTime"

  # Numeric types
  BigInt: "i64"
  Decimal: "rust_decimal::Decimal"

  # Special types
  UUID: "uuid::Uuid"
  JSON: "serde_json::Value"
  Bytes: "Vec<u8>"
```

## 🔐 Authentication & Headers

### Static Headers

```yaml
headers:
  Authorization: "Bearer your-jwt-token"
  X-API-Key: "your-api-key"
  User-Agent: "MyApp/1.0"
```

### Environment Variables

Use environment variables for sensitive data:

```yaml
headers:
  Authorization: "${AUTH_TOKEN}"
  X-API-Key: "${API_KEY}"
```

```bash
export AUTH_TOKEN="Bearer your-jwt-token"
export API_KEY="your-api-key"
graphql-codegen-rust
```

## 📁 Output Structure

Customize where generated code lives:

```yaml
rust_codegen:
  output_dir: ./src/database  # Base output directory

  # Generated structure:
  # ./src/database/
  # ├── graphql-codegen-rust.toml
  # ├── src/
  # │   ├── schema.rs           # Table definitions
  # │   └── entities/           # Entity structs
  # │       ├── user.rs
  # │       └── post.rs
  # └── migrations/             # SQL migrations
  #     └── 001_create_users/
```

## 🚀 Advanced Configuration

### Conditional Generation

```yaml
rust_codegen:
  generate_migrations: true   # Create SQL migration files
  generate_entities: true     # Create Rust entity structs
  # Future: selective type generation
  # generate_unions: false    # Skip union types
  # generate_interfaces: true # Include interface types
```

### Development vs Production

```yaml
# For development
schema: http://localhost:4000/graphql
rust_codegen:
  db: sqlite

# For production
# schema: https://api.production.com/graphql
# rust_codegen:
#   db: postgres
```

## 🔍 Validation

The tool validates your configuration on startup:

- **Schema accessibility** - Can reach GraphQL endpoint
- **ORM compatibility** - Selected ORM supports database type
- **Type mappings** - Custom scalars are valid Rust types
- **Output permissions** - Can write to specified directory

## 📚 Examples by Use Case

### Tauri Desktop App
```yaml
schema: https://api.example.com/graphql
rust_codegen:
  orm: diesel
  db: sqlite
  output_dir: ./src-tauri/src/db
```

### Web Service Backend
```yaml
schema: ./schema.graphql  # Local SDL file
rust_codegen:
  orm: sea_orm
  db: postgres
  output_dir: ./src/database
```

### Enterprise Monorepo
```yaml
schema: https://internal-api.company.com/graphql
headers:
  Authorization: "${SERVICE_TOKEN}"
rust_codegen:
  orm: diesel
  db: postgres
  table_naming: snake_case
  type_mappings:
    CompanyID: "custom_types::CompanyId"
```

## 🔄 Migration Workflow

When your GraphQL schema changes:

1. **Update schema** - Modify your GraphQL API
2. **Regenerate code** - Run `graphql-codegen-rust`
3. **Apply migrations** - Run generated SQL migrations
4. **Update application** - Use new entity types

The tool handles schema evolution automatically, generating appropriate database migrations for schema changes.

---

💡 **Pro Tip**: Start with minimal configuration and add customizations as needed. Most projects work great with just `schema`, `orm`, and `db` settings!
