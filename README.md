# Komora

WIP toy relational database management system for personal learning
purpose.

Goals:

- learn how databases work under the hood
- practice Rust

## Build

```bash
cargo build
```

## Format

```bash
cargo fmt
```

## Lint

```bash
cargo clippy
```

## Run

```bash
cargo run -- --help
cargo run -- --db /var/tmp/komora/test_db init
cargo run -- --db /var/tmp/komora/test_db info
```

## Logging

Log level is controlled via env variable `LOG_LEVEL`

```bash
LOG_LEVEL=debug cargo run -- --db /var/tmp/komora/test_db init
```

## Tests

```bash
cargo test
```

## Design notes

### Storage

Row-based storage was chosen here as it is considered a classic for OLTP
workloads
and my goal is to understand how database management systems optimized for these
kind
of workloads work under the hood.

Example of database folder contents:

```
catalog.toml
tables/
  1.schema.toml
  1.data
  2.schema.toml
  2.data
```

### Catalog

Catalog is stored in a file named `catalog.toml` and holds meta-information like
storage type, version and next table id. It also holds a mapping from table
names to table ids.

Example:

```
engine = "Komora"
storage_type = "RowOriented"
version = 1
next_table_id = 2

[tables]
table1 = 1
```

### Table id

Table id is a monotonically increasing integer id.
It is useful to have some stable table identifier which contrary to
table name does not change. Without a stable identifier we would have to rename
table schema and data files every time the table name changes.

### Table schema

For now only integer and text fields are supported. Fields can be nullable.
Table schema is stored in a separate file. File name format:
`<table_id>.schema.toml`

Example of a schema with two columns named `column1` and `column2`:

```
[[columns]]
name = "column1"
typ = "Int"
nullable = false

[[columns]]
name = "column2"
typ = "Text"
nullable = true
```

### Table data

Table data is stored in separate file. File name format: `<table_id>.data`. The
format of the file itself is to be defined.

### Record id

Internal record id is represented with `(page_id, slot_id)` pair where `page_id`
and `slot_id` are unsigned 32-bit integers.
