# 📖 Reference Guide

Technical reference for GraphQL Rust Codegen - type mappings, generated structure, limitations, and advanced usage.

## 🗂️ Generated Project Structure

When you run code generation, the following structure is created:

```
output_dir/
├── graphql-codegen-rust.toml     # Configuration file (auto-generated)
├── src/
│   ├── schema.rs                 # Diesel table! macros
│   └── entities/                 # Individual entity files
│       ├── user.rs
│       ├── post.rs
│       ├── category.rs
│       └── ...
└── migrations/                   # SQL migration files
    ├── 001_create_users_table/
    │   ├── up.sql               # Migration up script
    │   └── down.sql             # Migration rollback
    ├── 002_create_posts_table/
    │   ├── up.sql
    │   └── down.sql
    └── ...
```

### File Purposes

| File/Directory | Purpose | ORM Support |
|----------------|---------|-------------|
| `schema.rs` | Table definitions and relationships | Diesel only |
| `entities/*.rs` | Type-safe entity structs | Both Diesel & Sea-ORM |
| `migrations/` | Database schema evolution | Both ORMs |
| `graphql-codegen-rust.toml` | Configuration reference | Both ORMs |

## 🗃️ Type Mapping Reference

### GraphQL to Rust Type Conversion

| GraphQL Type | SQLite (Rust) | PostgreSQL (Rust) | MySQL (Rust) | SQL Type |
|-------------|----------------|-------------------|--------------|----------|
| `ID` | `i32` | `uuid::Uuid` | `u32` | `INTEGER` / `UUID` |
| `String` | `String` | `String` | `String` | `TEXT` / `VARCHAR` |
| `Int` | `i32` | `i32` | `i32` | `INTEGER` |
| `Float` | `f64` | `f64` | `f64` | `REAL` / `DOUBLE` |
| `Boolean` | `bool` | `bool` | `bool` | `INTEGER` / `BOOLEAN` |
| `DateTime` | `chrono::NaiveDateTime` | `chrono::DateTime<Utc>` | `chrono::NaiveDateTime` | `TEXT` / `TIMESTAMP` |
| `JSON` | `serde_json::Value` | `serde_json::Value` | `serde_json::Value` | `TEXT` / `JSON` |
| `BigInt` | `i64` | `i64` | `i64` | `INTEGER` |

### Custom Scalar Mappings

Override default mappings in your configuration:

```yaml
rust_codegen:
  type_mappings:
    # Temporal types
    DateTime: "chrono::DateTime<chrono::Utc>"
    Date: "chrono::NaiveDate"
    Time: "chrono::NaiveTime"

    # Custom business types
    UserID: "my_app::UserId"
    ProductCode: "my_app::ProductCode"

    # Special types
    UUID: "uuid::Uuid"
    Decimal: "rust_decimal::Decimal"
    Bytes: "Vec<u8>"
```

## 🔗 Relationship Mapping

### Foreign Key Detection

The tool automatically detects relationships based on naming patterns:

```
User {
  id: ID!
  name: String!
}

Post {
  id: ID!
  title: String!
  authorId: ID!      # → BelongsTo relationship with User
  categoryId: ID     # → BelongsTo relationship with Category
}
```

### Generated Relationship Code

**Diesel:**
```rust
#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(User, foreign_key = "author_id")]
#[belongs_to(Category, foreign_key = "category_id")]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub author_id: i32,
    pub category_id: Option<i32>,
}

// Joinable relationships
joinable!(posts -> users (author_id));
joinable!(posts -> categories (category_id));
```

**Sea-ORM:**
```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub author_id: i32,
    pub category_id: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::AuthorId",
        to = "super::users::Column::Id"
    )]
    Author,

    #[sea_orm(
        belongs_to = "super::categories::Entity",
        from = "Column::CategoryId",
        to = "super::categories::Column::Id"
    )]
    Category,
}
```

## 🎯 Union & Interface Support

### Current Implementation

GraphQL unions and interfaces are parsed but generate limited code:

```graphql
interface Node {
  id: ID!
}

type User implements Node {
  id: ID!
  name: String!
}

type Post implements Node {
  id: ID!
  title: String!
}

union SearchResult = User | Post
```

**Generated:** Type definitions are created but no special union/interface handling yet.

### Future Enhancements

- Polymorphic queries
- Interface implementation validation
- Union type resolution helpers

## 📊 Database-Specific Features

### SQLite (Default)
- **Best for:** Development, embedded apps, simple deployments
- **ID Type:** `i32` (auto-increment)
- **Performance:** Fast reads, ACID transactions
- **Limitations:** No concurrent writes, single file database

### PostgreSQL
- **Best for:** Production, complex queries, concurrent access
- **ID Type:** `uuid::Uuid` (recommended for distributed systems)
- **Features:** Advanced indexing, JSON support, custom types
- **Performance:** Excellent for complex queries

### MySQL
- **Best for:** Legacy systems, high write throughput
- **ID Type:** `u32` (auto-increment)
- **Features:** Foreign key constraints, partitioning
- **Performance:** Fast writes, good for OLTP

## 🏃‍♂️ Performance Characteristics

### Compilation Time
- **Small schemas (< 10 types):** < 1 second
- **Medium schemas (10-50 types):** 2-5 seconds
- **Large schemas (50+ types):** 5-15 seconds

### Generated Code Size
- **Per entity:** ~100-300 lines of code
- **Schema file:** Scales with number of relationships
- **Migrations:** 1-2 files per type

### Runtime Performance
- **Zero overhead:** Generated code compiles to native performance
- **Type safety:** Compile-time guarantees prevent runtime errors
- **Query optimization:** ORM-specific optimizations applied

## ⚠️ Limitations & Known Issues

### Current Limitations

- **Many-to-many relationships:** Not automatically detected
- **Polymorphic associations:** Limited support
- **Complex nested types:** May require manual adjustments
- **Subscriptions:** Ignored (focus on data persistence)
- **Mutations:** Only type definitions generated

### Schema Requirements

- **Object types only:** Currently focuses on entities with database representation
- **ID fields:** Assumes each type has an `id` field
- **Naming conventions:** Foreign keys should follow `fieldNameId` pattern

### ORM-Specific Notes

**Diesel:**
- Requires explicit schema definitions
- Macros may need adjustment for complex relationships
- Excellent for type safety and performance

**Sea-ORM:**
- More flexible with async operations
- Better for complex relationship modeling
- Runtime relationship resolution capabilities

## 🔧 Advanced Configuration

### Custom Type Overrides

```yaml
rust_codegen:
  # Override specific field types
  field_overrides:
    user.email: "EmailAddress"  # Custom email type
    product.price: "Decimal"    # Custom decimal type

  # Skip certain types
  exclude_types:
    - "InternalType"
    - "AdminOnlyType"

  # Include only specific types
  include_types:
    - "User"
    - "Post"
    - "Category"
```

### Migration Customization

```yaml
rust_codegen:
  # Migration naming
  migration_timestamp: true  # Include timestamps in names

  # Migration content
  include_indexes: true      # Generate index migrations
  include_constraints: true  # Generate constraint migrations
```

## 🐛 Troubleshooting

### Common Issues

**"Table already exists"**
```sql
-- Drop and recreate (development only)
DROP TABLE IF EXISTS your_table;
-- Then run migrations again
```

**"Foreign key constraint fails"**
- Ensure referenced tables exist
- Check data insertion order
- Verify foreign key values are valid

**"Type mismatch in generated code"**
- Check your `type_mappings` configuration
- Ensure custom types implement required traits
- Update dependencies if using custom types

### Debug Information

Enable detailed logging:

```bash
RUST_LOG=graphql_codegen_rust=debug graphql-codegen-rust generate
```

### Manual Overrides

For complex cases, you can manually modify generated code:

```rust
// In your entity file, add custom logic
impl User {
    pub fn custom_method(&self) -> String {
        format!("Custom: {}", self.name)
    }
}
```

## 📈 Metrics & Monitoring

### Generation Statistics

The tool reports generation metrics:

```
Generated 15 entity files
Generated 23 migration files
Total relationships detected: 8
Generation time: 2.3 seconds
```

### Performance Monitoring

For large schemas, monitor:

- **Memory usage:** Large schemas may require more RAM
- **Disk I/O:** Migration file generation
- **Compilation time:** Impact on build times

## 🔄 Version Compatibility

### Rust Version Support
- **MSRV:** Rust 1.85+
- **Recommended:** Latest stable Rust
- **Nightly:** May work but not officially supported

### ORM Version Compatibility

| ORM | Supported Versions | Notes |
|-----|-------------------|--------|
| Diesel | 2.0+ | Full feature support |
| Sea-ORM | 0.12+ | Async runtime required |

### Database Driver Compatibility

| Database | Driver | Version |
|----------|--------|---------|
| SQLite | rusqlite | 0.29+ |
| PostgreSQL | diesel::pg | 2.0+ |
| MySQL | diesel::mysql | 2.0+ |

## 🚀 Future Features

### Planned Enhancements

- **Advanced relationships:** Many-to-many, polymorphic
- **Custom generators:** Plugin system for specialized code
- **Schema evolution:** Smart migration generation
- **Performance optimizations:** Compile-time query optimization

---

📚 **Need help?** Check the [examples](examples.md) or open an issue on GitHub.
