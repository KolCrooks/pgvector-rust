# pgvector-rust

[pgvector](https://github.com/ankane/pgvector) support for Rust

Supports [Rust-Postgres](https://github.com/sfackler/rust-postgres), [SQLx](https://github.com/launchbadge/sqlx), and [Diesel](https://github.com/diesel-rs/diesel)

[![Build Status](https://github.com/ankane/pgvector-rust/workflows/build/badge.svg?branch=master)](https://github.com/ankane/pgvector-rust/actions)

## Getting Started

Follow the instructions for your database library:

- [Rust-Postgres](#rust-postgres)
- [SQLx](#sqlx)
- [Diesel](#diesel)

## Rust-Postgres

Add this line to your application’s `Cargo.toml` under `[dependencies]`:

```toml
pgvector = { version = "0.1", features = ["postgres"] }
```

Create a vector from a `Vec<f32>`

```rust
let vec = pgvector::Vector::from(vec![1.0, 2.0, 3.0]);
```

Insert a vector

```rust
client.execute("INSERT INTO table (column) VALUES ($1)", &[&vec])?;
```

Get the nearest neighbor

```rust
let row = client.query_one("SELECT * FROM table ORDER BY column <-> $1 LIMIT 1", &[&vec])?;
```

Retrieve a vector

```rust
let row = client.query_one("SELECT column FROM table LIMIT 1", &[])?;
let vec: pgvector::Vector = row.get(0);
```

Use `Option` if the value could be `NULL`

```rust
let res: Option<pgvector::Vector> = row.get(0);
```

Convert a vector to a `Vec<f32>`

```rust
let f32_vec = vec.to_vec();
```

## SQLx

Add this line to your application’s `Cargo.toml` under `[dependencies]`:

```toml
pgvector = { version = "0.1", features = ["sqlx"] }
```

Create a vector from a `Vec<f32>`

```rust
let vec = pgvector::Vector::from(vec![1.0, 2.0, 3.0]);
```

Insert a vector

```rust
sqlx::query("INSERT INTO table (column) VALUES ($1)").bind(vec).execute(&pool).await?;
```

Get the nearest neighbors

```rust
let rows = sqlx::query("SELECT * FROM table ORDER BY column <-> $1 LIMIT 1").bind(vec).fetch_all(&pool).await?;
```

Retrieve a vector

```rust
let row = sqlx::query("SELECT column FROM table LIMIT 1").fetch_one(&pool).await?;
let vec: pgvector::Vector = row.try_get("column").unwrap();
```

## Diesel

Add this line to your application’s `Cargo.toml` under `[dependencies]`:

```toml
pgvector = { version = "0.1", features = ["diesel"] }
```

And add this line to your application’s `diesel.toml` under `[print_schema]`:

```toml
import_types = ["diesel::sql_types::*", "pgvector::sql_types::*"]
```

Create a migration

```sh
diesel migration generate create_vector_extension
```

with `up.sql`:

```sql
CREATE EXTENSION vector
```

and `down.sql`:

```sql
DROP EXTENSION vector
```

Run the migration

```sql
diesel migration run
```

You can now use the `vector` type in future migrations

```sql
CREATE TABLE items (
  factors VECTOR(3)
)
```

For models, use:

```rust
pub struct Item {
    pub factors: Option<pgvector::Vector>
}
```

Create a vector from a `Vec<f32>`

```rust
let factors = pgvector::Vector::from(vec![1.0, 2.0, 3.0]);
```

Insert a vector

```rust
let new_item = Item {
    factors: Some(factors)
};

diesel::insert_into(items::table)
        .values(&new_item)
        .get_result(conn)
        .expect("Error saving new item")
```

Convert a vector to a `Vec<f32>`

```rust
let f32_factors = factors.to_vec();
```

## History

View the [changelog](https://github.com/ankane/pgvector-rust/blob/master/CHANGELOG.md)

## Contributing

Everyone is encouraged to help improve this project. Here are a few ways you can help:

- [Report bugs](https://github.com/ankane/pgvector-rust/issues)
- Fix bugs and [submit pull requests](https://github.com/ankane/pgvector-rust/pulls)
- Write, clarify, or fix documentation
- Suggest or add new features

To get started with development:

```sh
git clone https://github.com/ankane/pgvector-rust.git
cd pgvector-rust
cargo test --features postgres
cargo test --features sqlx
cargo test --features diesel
```
