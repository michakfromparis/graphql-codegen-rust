# GraphQL Diesel Sync

A Rust CLI tool that automates the synchronization of remote GraphQL schemas with local Diesel or Sea-ORM databases. Perfect for offline-first Tauri applications that need to sync GraphQL types to local SQLite/PostgreSQL databases.

## Features

- **GraphQL Schema Introspection**: Fetches and parses GraphQL schemas from remote endpoints
- **Multi-ORM Support**: Generates code for both Diesel and Sea-ORM
- **Database Support**: Works with SQLite (default), PostgreSQL, and MySQL
- **Migration Generation**: Automatically creates SQL migration files
- **Configuration Management**: TOML-based configuration for custom mappings
- **Type Safety**: Generates strongly-typed Rust structs from GraphQL schemas

## Installation

### From Source

```bash
git clone https://github.com/yourusername/graphql-diesel-sync.git
cd graphql-diesel-sync
cargo build --release
```

### From Crates.io (future)

```bash
cargo install graphql-diesel-sync
```

## Usage

### Initialize a New Project

```bash
graphql-diesel-sync init \
  --url https://api.example.com/graphql \
  --orm diesel \
  --db sqlite \
  --output ./db
```

This will:
1. Introspect the GraphQL schema from the endpoint
2. Create a configuration file (`graphql-diesel-sync.toml`)
3. Generate Diesel schema definitions, entity structs, and migration files

### Regenerate Code

```bash
graphql-diesel-sync generate
```

Regenerates code from the existing configuration, useful when the remote schema has changed.

### Generate Specific Types

```bash
graphql-diesel-sync generate --types User,Product,Order
```

Only generates code for the specified GraphQL types.

## Configuration

The tool creates a `graphql-diesel-sync.toml` configuration file:

```toml
url = "https://api.example.com/graphql"
orm = "Diesel"
db = "Sqlite"
output_dir = "./generated"
generate_migrations = true
generate_entities = true
table_naming = "snake_case"

[headers]
Authorization = "Bearer your-token-here"

[type_mappings]
# Custom scalar mappings
DateTime = "chrono::NaiveDateTime"
```

## Generated Structure

```
output_dir/
├── graphql-diesel-sync.toml
├── src/
│   ├── schema.rs          # Diesel table! macros
│   └── entities/          # Entity structs
│       ├── user.rs
│       └── product.rs
└── migrations/
    ├── 001_create_users_table/
    │   ├── up.sql
    │   └── down.sql
    └── 002_create_products_table/
        ├── up.sql
        └── down.sql
```

## GraphQL Type Mapping

| GraphQL Type | Rust Type (SQLite) | Rust Type (PostgreSQL) | SQL Type |
|-------------|-------------------|----------------------|----------|
| `ID` | `i32` | `uuid::Uuid` | `INTEGER` / `UUID` |
| `String` | `String` | `String` | `TEXT` |
| `Int` | `i32` | `i32` | `INTEGER` |
| `Float` | `f64` | `f64` | `REAL` |
| `Boolean` | `bool` | `bool` | `INTEGER` / `BOOLEAN` |

## Examples

### Vendure Integration

```bash
graphql-diesel-sync init \
  --url https://demo.vendure.io/shop-api \
  --orm diesel \
  --db sqlite \
  --output ./src/db
```

### Tauri App Integration

Add to your `build.rs`:

```rust
use std::process::Command;

fn main() {
    // Regenerate database code before build
    Command::new("graphql-diesel-sync")
        .arg("generate")
        .status()
        .expect("Failed to regenerate database code");

    tauri_build::build()
}
```

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Running

```bash
cargo run -- init --url http://localhost:4000/graphql
```

## Limitations

- Currently focuses on object types and basic relationships
- Unions, interfaces, and complex nested types are not yet fully supported
- Subscriptions are ignored
- Custom scalars require manual mapping in configuration

## Contributing

Contributions are welcome! Areas for improvement:

- SDL parsing support (currently introspection-only)
- Union/interface support
- Better relationship handling
- More ORM integrations
- Plugin system for custom generators

## License

MIT License

## Roadmap

- [ ] SDL file parsing support
- [ ] Union and interface type generation
- [ ] Advanced relationship mapping
- [ ] Sea-ORM migration generation
- [ ] Plugin system
- [ ] GraphQL subscription support
- [ ] CI/CD and releases
