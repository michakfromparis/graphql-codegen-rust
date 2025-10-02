# 💡 Examples

Real-world integration examples showing how GraphQL Rust Codegen fits into different application architectures.

## 🛒 E-Commerce with Vendure

**Scenario**: Building a desktop POS system that syncs with an e-commerce GraphQL API.

### Setup

```bash
# Initialize with Vendure's demo API
graphql-codegen-rust init \
  --url https://demo.vendure.io/shop-api \
  --orm diesel \
  --db sqlite \
  --output ./src/db
```

### Generated Structure

```
src/db/
├── entities/
│   ├── product.rs      # Product entity with variants
│   ├── customer.rs     # Customer data
│   ├── order.rs        # Order management
│   └── inventory.rs    # Stock levels
└── migrations/
    ├── 001_create_products/
    ├── 002_create_customers/
    └── 003_create_orders/
```

### Usage in Tauri

```rust
// src-tauri/src/main.rs
use tauri::State;
use diesel::prelude::*;
use crate::db::schema::*;
use crate::db::entities::*;

struct AppState {
    conn: SqliteConnection,
}

#[tauri::command]
async fn sync_products(state: State<'_, AppState>) -> Result<Vec<Product>, String> {
    // Sync latest products from GraphQL API
    let latest_products = fetch_from_graphql_api().await?;

    // Store locally for offline access
    diesel::insert_or_replace_into(products::table)
        .values(&latest_products)
        .execute(&state.conn)
        .map_err(|e| e.to_string())?;

    // Return cached products
    products::table
        .load::<Product>(&state.conn)
        .map_err(|e| e.to_string())
}
```

## 📱 Social Media Client

**Scenario**: Offline-first social media app with local caching.

### Configuration

```yaml
schema: https://social-api.example.com/graphql
rust_codegen:
  orm: sea_orm
  db: sqlite
  output_dir: ./src/database
  type_mappings:
    DateTime: "chrono::DateTime<chrono::Utc>"
```

### Generated Entities

```rust
// Auto-generated entities
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub content: String,
    pub author_id: i32,
    pub created_at: DateTime<Utc>,
    pub likes_count: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::AuthorId",
        to = "super::users::Column::Id"
    )]
    Author,
}
```

### Offline-First Architecture

```rust
// Repository pattern for data management
pub struct PostRepository {
    conn: DatabaseConnection,
}

impl PostRepository {
    pub async fn sync_latest_posts(&self) -> Result<(), DbErr> {
        // Fetch from API
        let api_posts = graphql_client::fetch_posts().await?;

        // Store locally
        for post in api_posts {
            post::ActiveModel::from(post)
                .insert(&self.conn)
                .await?;
        }

        Ok(())
    }

    pub async fn get_cached_posts(&self, limit: u64) -> Result<Vec<post::Model>, DbErr> {
        Post::find()
            .order_by_desc(post::Column::CreatedAt)
            .limit(limit)
            .all(&self.conn)
            .await
    }
}
```

## 🏢 Enterprise API Gateway

**Scenario**: Corporate application with multiple data sources.

### Multi-Schema Setup

```yaml
# Primary API
schema: https://internal-api.company.com/graphql
headers:
  Authorization: "${SERVICE_TOKEN}"

rust_codegen:
  orm: diesel
  db: postgres
  output_dir: ./src/enterprise

  # Enterprise naming conventions
  table_naming: snake_case
  type_mappings:
    CompanyID: "enterprise_types::CompanyId"
    DepartmentID: "enterprise_types::DepartmentId"
    AuditLog: "enterprise_types::AuditEntry"
```

### Advanced Relationships

```rust
// Generated with foreign key relationships
#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(Department)]
#[belongs_to(Company)]
pub struct Employee {
    pub id: i32,
    pub company_id: CompanyID,
    pub department_id: DepartmentID,
    pub name: String,
    pub email: String,
}

#[derive(Queryable, Identifiable)]
#[has_many(Employee)]
pub struct Department {
    pub id: i32,
    pub company_id: CompanyID,
    pub name: String,
    pub budget: Decimal,
}

#[derive(Queryable, Identifiable)]
#[has_many(Department)]
#[has_many(Employee)]
pub struct Company {
    pub id: i32,
    pub name: String,
    pub founded_date: NaiveDate,
}
```

## 🔄 Data Synchronization Patterns

### Background Sync Service

```rust
use tokio::time::{sleep, Duration};

pub struct SyncService {
    repositories: Vec<Box<dyn SyncableRepository>>,
}

#[async_trait]
pub trait SyncableRepository {
    async fn sync(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn name(&self) -> &str;
}

impl SyncService {
    pub async fn start_background_sync(&self) {
        loop {
            for repo in &self.repositories {
                if let Err(e) = repo.sync().await {
                    log::error!("Failed to sync {}: {}", repo.name(), e);
                }
            }
            sleep(Duration::from_secs(300)).await; // Sync every 5 minutes
        }
    }
}
```

### Conflict Resolution

```rust
pub enum SyncConflict {
    LocalNewer,
    RemoteNewer,
    BothModified,
}

pub async fn resolve_conflict<T: Syncable>(
    local: &T,
    remote: &T,
) -> Result<T, SyncError> {
    match detect_conflict(local, remote) {
        SyncConflict::LocalNewer => Ok(local.clone()),
        SyncConflict::RemoteNewer => Ok(remote.clone()),
        SyncConflict::BothModified => {
            // Custom merge logic
            merge_versions(local, remote)
        }
    }
}
```

## 🎨 Custom Integration Patterns

### Plugin Architecture

```rust
// Custom codegen plugins
pub trait CodegenPlugin {
    fn generate_custom_code(&self, schema: &ParsedSchema) -> String;
    fn modify_entity(&self, entity: &mut EntityModel);
}

pub struct AuditPlugin;

impl CodegenPlugin for AuditPlugin {
    fn generate_custom_code(&self, _schema: &ParsedSchema) -> String {
        r#"
pub trait Auditable {
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
    fn created_by(&self) -> Option<UserId>;
}
"#.to_string()
    }

    fn modify_entity(&self, entity: &mut EntityModel) {
        // Add audit fields to all entities
        entity.fields.push(Field {
            name: "created_at".to_string(),
            field_type: FieldType::Scalar("DateTime".to_string()),
            ..Default::default()
        });
    }
}
```

### Multi-Database Setup

```rust
// Support for read/write databases
pub struct DatabaseManager {
    read_conn: DatabaseConnection,
    write_conn: DatabaseConnection,
}

impl DatabaseManager {
    pub async fn execute_read<F, T, Fut>(&self, operation: F) -> Result<T, DbErr>
    where
        F: FnOnce(&DatabaseConnection) -> Fut,
        Fut: Future<Output = Result<T, DbErr>>,
    {
        operation(&self.read_conn).await
    }

    pub async fn execute_write<F, T, Fut>(&self, operation: F) -> Result<T, DbErr>
    where
        F: FnOnce(&DatabaseConnection) -> Fut,
        Fut: Future<Output = Result<T, DbErr>>,
    {
        operation(&self.write_conn).await
    }
}
```

## 📊 Performance Optimization Examples

### Connection Pooling

```rust
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::SqliteConnection;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

pub fn create_pool(database_url: &str) -> Result<DbPool, r2d2::Error> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder()
        .max_size(10)
        .build(manager)
}
```

### Query Optimization

```rust
// Efficient batch operations
pub async fn bulk_insert_products(
    conn: &DatabaseConnection,
    products: Vec<NewProduct>,
) -> Result<(), DbErr> {
    // Use transactions for atomicity
    let txn = conn.begin().await?;

    for chunk in products.chunks(1000) {
        Product::insert_many(chunk)
            .exec(&txn)
            .await?;
    }

    txn.commit().await?;
    Ok(())
}
```

---

🚀 **Want to share your integration pattern?** Open a PR with your example!
