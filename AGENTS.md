# AGENTS.md

This file contains guidelines and commands for agentic coding agents working on the Friday API project - a Rust-based synchronization service for Мой Склад ERP with WordPress WooCommerce, with stock information collection from email and web sources.

## Project Overview

- **Language**: Rust (Edition 2021)
- **Framework**: Axum for HTTP services, Tokio for async runtime
- **Database**: PostgreSQL with SQLx for migrations and queries
- **Purpose**: Syncs Мой Склад ERP data with WooCommerce and collects stock info from email/web

## Development Commands

### Building and Running
```bash
# Build the project
cargo build

# Run in development mode
cargo run
# Or using just
just run

# Build for production
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run tests in specific module
cargo test module_name
```

### Code Quality
```bash
# Check code without building
cargo check

# Run linter
cargo clippy

# Format code
cargo fmt

# Run clippy with all targets
cargo clippy --all-targets --all-features
```

### Database
```bash
# Run migrations (handled automatically in main.rs)
# Migrations are in the `migrations/` directory
```

## Code Style Guidelines

### Import Organization
```rust
// Standard library imports first
use std::sync::Arc;
use std::ops::Deref;

// External crate imports
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tracing::{error, info};

// Local module imports
use crate::storage::StockStorage;
use crate::{models::Stock, utils::pause};
```

### Module Structure
- Use `mod.rs` files for module organization
- Export public items via `pub use` at module level
- Keep related functionality in dedicated modules
- Follow the established directory structure:
  ```
  src/
  ├── main.rs
  ├── lib.rs
  ├── error.rs
  ├── models/
  ├── storage/
  ├── stock_service/
  │   ├── mod.rs
  │   ├── mail_client.rs
  │   ├── web_spider.rs
  │   └── parser/
  └── synchronizer.rs
  ```

### Error Handling
- Use the custom `AppError` enum defined in `src/error.rs`
- Implement `From` traits for common error types
- Return `Result<T>` from functions that can fail
- Use `anyhow::Result<()>` for main functions and top-level operations
- Log errors appropriately with `tracing::error!`

### Naming Conventions
- **Structs**: PascalCase (e.g., `StockStorage`, `MailClient`)
- **Functions**: snake_case (e.g., `run`, `update_stock`)
- **Constants**: SCREAMING_SNAKE_CASE for env vars and secrets
- **Modules**: snake_case with descriptive names
- **File names**: snake_case matching module names

### Async Patterns
- Use `tokio::spawn` for concurrent operations
- Prefer `Arc` for shared state across async tasks
- Use `UnboundedSender/Receiver` for inter-task communication
- Implement cancellation points where appropriate
- Handle async errors with `?` operator

### Database Operations
- Use SQLx with PostgreSQL features
- Implement connection pooling with `PgPoolOptions`
- Use `sqlx::migrate!()` for database migrations
- Follow async/await patterns for all database calls
- Use `FromRow` derive for struct mapping

### Configuration
- Environment variables are required (not optional)
- Use `std::env::var()` with `expect()` for required vars
- Load environment via `.env` file in development
- Use tracing for debug output of configuration

### Testing Guidelines
- Write unit tests for core business logic
- Test error paths and edge cases
- Use `#[tokio::test]` for async test functions
- Mock external dependencies when testing integration
- Keep tests focused and maintainable

### Code Comments
- Add comments for complex business logic
- Use Russian comments where the codebase already uses them (consistent with existing code)
- Document public API functions with `///` doc comments
- Comment non-obvious async patterns or error handling

### Performance Considerations
- Use `Arc` for shared data to avoid unnecessary cloning
- Implement proper connection pooling for database access
- Use bounded channels when backpressure is needed
- Consider memory usage when processing large datasets
- Use `cargo clippy` to identify performance anti-patterns

## Environment Variables

Required environment variables (see `src/main.rs` and `src/lib.rs`):
- `DATABASE_URL` - PostgreSQL connection string
- `MS_TOKEN` - Мой Склад API token
- `SAFIRA_CK` - WooCommerce consumer key
- `SAFIRA_CS` - WooCommerce consumer secret
- `SAFIRA_HOST` - WooCommerce host URL
- `ORTGRAPH_USERNAME` - Web scraping username
- `ORTGRAPH_PASSWORD` - Web scraping password
- `MAIL_HOST` - IMAP server host
- `MAIL_USER` - IMAP username
- `MAIL_PASS` - IMAP password

## Key Dependencies

- `axum` - Web framework
- `sqlx` - Database toolkit
- `tokio` - Async runtime
- `serde` - Serialization/deserialization
- `anyhow` - Error handling
- `tracing` - Logging and instrumentation
- `reqwest` - HTTP client
- `rust-moysklad` - Мой Сklad API client
- `rust-woocommerce` - WooCommerce API client

## Git Workflow

- Use conventional commit messages
- Run `cargo clippy` and `cargo test` before committing
- Ensure code formatting with `cargo fmt`
- Check that all environment variables are documented when added