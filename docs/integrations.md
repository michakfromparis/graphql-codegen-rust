# 🔄 Integrations

Comprehensive guides for integrating GraphQL Rust Codegen with other tools and frameworks.

## GraphQL Code Generator Integration

**🚀 KEY FEATURE: Unified TypeScript + Rust Workflow**

For Tauri developers, GraphQL Rust Codegen integrates seamlessly with the popular [GraphQL Code Generator](https://the-guild.dev/graphql/codegen) tool, enabling a unified workflow for generating both frontend TypeScript types and backend Rust database code from a single GraphQL schema.

### Why This Integration Matters

- **Single Source of Truth**: One GraphQL schema powers your entire stack
- **No Duplicate Configuration**: Shared config files eliminate sync issues
- **Type Safety End-to-End**: Compile-time guarantees from API to database
- **Seamless Tauri Development**: Perfect for offline-first desktop apps

### Quick Start

#### 1. Install Both Tools

```bash
# GraphQL Code Generator (frontend types)
npm install --save-dev @graphql-codegen/cli

# GraphQL Rust Codegen (backend database)
cargo install graphql-codegen-rust
```

#### 2. Create Unified Configuration

```yaml
# codegen.yml - Single source of truth for your entire stack
schema: https://api.example.com/graphql
documents: './src/**/*.graphql'

# TypeScript codegen (frontend)
generates:
  ./src/gql/:
    preset: client
    plugins:
      - typescript
      - typescript-operations
      - typescript-react-apollo

# Rust codegen (backend) - NEW!
rust_codegen:
  orm: diesel
  db: sqlite
  output_dir: ./src-tauri/src/db
  generate_migrations: true
  generate_entities: true
```

#### 3. Add to Package Scripts

```json
// package.json
{
  "scripts": {
    "codegen": "graphql-codegen --config codegen.yml && graphql-codegen-rust",
    "codegen:frontend": "graphql-codegen --config codegen.yml",
    "codegen:backend": "graphql-codegen-rust",
    "tauri": "tauri"
  }
}
```

#### 4. Use in Development

```bash
# Generate both frontend and backend code
npm run codegen

# Or run separately for faster iteration
npm run codegen:frontend  # Fast TypeScript updates
npm run codegen:backend   # Database schema updates

# Build your Tauri app
npm run tauri build
```

### Configuration Reference

#### Rust Codegen Configuration

```yaml
rust_codegen:
  # ORM selection
  orm: diesel | sea_orm

  # Database type
  db: sqlite | postgres | mysql

  # Output directory (relative to project root)
  output_dir: ./src-tauri/src/db

  # Code generation options
  generate_migrations: true
  generate_entities: true
  generate_schema: true

  # Custom type mappings
  type_mappings:
    DateTime: "chrono::DateTime<chrono::Utc>"
    UUID: "uuid::Uuid"
    Decimal: "rust_decimal::Decimal"

  # Database-specific options
  sqlite:
    foreign_keys: true
    journal_mode: WAL

  postgres:
    schema: public

  # Advanced options
  table_naming: snake_case | camelCase
  column_naming: snake_case | camelCase
```

### Advanced Integration Patterns

#### Build-Time Code Generation

Add automatic code generation to your Tauri build process:

```rust
// src-tauri/build.rs
fn main() {
    // Regenerate database code on every build
    std::process::Command::new("graphql-codegen-rust")
        .status()
        .expect("Failed to regenerate database code");

    // Optional: Run migrations
    std::process::Command::new("diesel")
        .args(&["migration", "run"])
        .status()
        .ok(); // Don't fail build if migrations fail

    tauri_build::build()
}
```

#### Hot Reload Setup

For development with hot reloading:

```javascript
// vite.config.js or webpack.config.js
import { exec } from 'child_process';

export default {
  plugins: [
    {
      name: 'graphql-codegen-watch',
      configureServer(server) {
        // Watch for GraphQL schema changes
        exec('graphql-codegen --watch --config codegen.yml', (error, stdout, stderr) => {
          if (error) {
            console.error('GraphQL Codegen error:', error);
            return;
          }
          console.log('Frontend types updated');
        });

        // Watch for Rust codegen changes
        // Note: You'll need a file watcher for the Rust side
      }
    }
  ]
}
```

#### Multi-Environment Configuration

Handle different environments with shared configurations:

```yaml
# codegen.base.yml
schema: ${GRAPHQL_SCHEMA_URL}
documents: './src/**/*.graphql'

generates:
  ./src/gql/:
    preset: client
    plugins:
      - typescript
      - typescript-operations

rust_codegen: &rust_codegen_base
  orm: diesel
  db: sqlite
  output_dir: ./src-tauri/src/db
  generate_migrations: true

# codegen.development.yml
<<: *file(codegen.base.yml)
rust_codegen:
  <<: *rust_codegen_base
  db: sqlite

# codegen.production.yml
<<: *file(codegen.base.yml)
rust_codegen:
  <<: *rust_codegen_base
  db: postgres
```

### Working with Generated Code

#### TypeScript Frontend

```typescript
// src/gql/types.ts (auto-generated)
export type Product = {
  __typename?: 'Product';
  id: Scalars['ID'];
  name: string;
  price: Scalars['Float'];
  category: Category;
};

// src/components/ProductCard.tsx
import { useProductQuery } from '../gql';

export function ProductCard({ productId }: { productId: string }) {
  const { data, loading } = useProductQuery({
    variables: { id: productId }
  });

  if (loading) return <div>Loading...</div>;

  return (
    <div>
      <h3>{data?.product?.name}</h3>
      <p>${data?.product?.price}</p>
    </div>
  );
}
```

#### Rust Backend

```rust
// src-tauri/src/db/entities/product.rs (auto-generated)
#[derive(Queryable, Identifiable)]
#[table_name = "products"]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: f64,
    pub category_id: i32,
}

// src-tauri/src/main.rs
use tauri::State;
use diesel::prelude::*;
use crate::db::schema::*;
use crate::db::entities::*;

struct AppState {
    conn: SqliteConnection,
}

#[tauri::command]
async fn get_products(state: State<'_, AppState>) -> Result<Vec<Product>, String> {
    products::table
        .load::<Product>(&state.conn)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn sync_product(
    state: State<'_, AppState>,
    id: i32,
    updates: ProductUpdate
) -> Result<Product, String> {
    // Update local database
    diesel::update(products::table.find(id))
        .set(&updates)
        .get_result::<Product>(&state.conn)
        .map_err(|e| e.to_string())
}
```

### Troubleshooting

#### Common Issues

**"Command not found: graphql-codegen-rust"**
```bash
# Install the CLI tool
cargo install graphql-codegen-rust

# Or run from source
cargo run --bin graphql-codegen-rust -- [args]
```

**Schema mismatch between frontend and backend**
```yaml
# Ensure both use the same schema
schema: https://api.example.com/graphql
# OR
schema: ./schema.graphql
```

**Migration conflicts**
```bash
# Reset migrations if needed
rm -rf src-tauri/src/db/migrations/
graphql-codegen-rust init --reset
```

**Type mapping issues**
```yaml
# Explicit type mappings
rust_codegen:
  type_mappings:
    DateTime: "chrono::DateTime<chrono::Utc>"
    UUID: "uuid::Uuid"
    JSON: "serde_json::Value"
```

#### Performance Considerations

- **Incremental Generation**: Only regenerate changed parts
- **Build Caching**: Use build.rs for automatic regeneration
- **Database Connection Pooling**: Implement for production workloads
- **Migration Strategy**: Plan for schema evolution

### Migration from Separate Workflows

If you're currently using GraphQL Code Generator and want to add Rust codegen:

1. **Backup your existing setup**
2. **Add rust_codegen section** to your existing codegen.yml
3. **Install GraphQL Rust Codegen**
4. **Test the integration** on a feature branch
5. **Update your build scripts**

### Real-World Examples

#### E-Commerce Tauri App

```yaml
# codegen.yml
schema: https://commerce-api.example.com/graphql
documents: './src/**/*.{graphql,ts,tsx}'

generates:
  ./src/gql/:
    preset: client
    plugins:
      - typescript
      - typescript-operations
      - typescript-react-apollo

rust_codegen:
  orm: diesel
  db: sqlite
  output_dir: ./src-tauri/src/db
  type_mappings:
    Money: "rust_decimal::Decimal"
    DateTime: "chrono::DateTime<chrono::Utc>"
```

#### Social Media Client

```yaml
# codegen.yml
schema: https://social-api.example.com/graphql

generates:
  ./src/gql/:
    plugins:
      - typescript
      - typescript-operations

rust_codegen:
  orm: sea_orm
  db: sqlite
  output_dir: ./src-tauri/src/db
  generate_migrations: true
```

---

🚀 **Need help with your integration?** Check the [GitHub issues](https://github.com/michakfromparis/graphql-codegen-rust/issues) or join the discussion!
