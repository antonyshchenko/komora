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

## Run

```bash
cargo run -- --help
cargo run -- --db /var/tmp/komora/test_db init
cargo run -- --db /var/tmp/komora/test_db doctor
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
