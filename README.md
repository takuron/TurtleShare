# TurtleShare

[简体中文](./README.zh-CN.md)

TurtleShare is a Rust-based membership content distribution backend for single-operator creators. It is designed for Patreon- or Afdian-like workflows: publish articles, distribute attachments, manage users and subscription periods, and serve a lightweight frontend from the same service.

Built with Axum, SQLite, sqlx, JWT authentication, and local file storage, the project aims to stay simple to deploy and easy to operate.

## Features

- Single-admin backend with JWT-based authentication
- User accounts, password changes, and time-based subscriptions
- Article publishing with Markdown content, attachments, and `publish_at`
- Access control based on subscription tier and publication time
- Local file uploads with randomized UUID-based storage paths
- Public site metadata via `[siteinfo]` in `config.toml`
- SQLite auto-initialization on startup

## Quick Start

### 1. Prerequisites

- Rust toolchain installed

### 2. Configure the server

The repository already includes a sample [`config.toml`](./config.toml). For local development, it currently contains a sample admin password hash for `admin123`. Replace it before any real deployment.

Generate a new Argon2id password hash:

```bash
cargo run -- hash-pw your-password
```

Then update at least these fields in `config.toml`:

```toml
[admin]
username = "admin"
password_hash = "$argon2id$..."

[server]
base_url = "http://127.0.0.1:3000"

[jwt]
base_secret = "change-this-in-production"
```

### 3. Run the server

```bash
cargo run
```

Useful options:

- `cargo run -- --help`
- `cargo run -- --config path/to/config.toml`
- `cargo run -- --require-existing-db`

On first startup, TurtleShare will create the SQLite database file, initialize the schema, and ensure the upload directory exists.

### 4. Verify it is running

- Open `http://127.0.0.1:3000/api/health`, or
- Call the health endpoint with your HTTP client of choice

## Documentation

- [`docs/architecture.md`](./docs/architecture.md)
- [`docs/configuration.md`](./docs/configuration.md)
- [`docs/api.md`](./docs/api.md)
- [`docs/database.md`](./docs/database.md)
- [`docs/project-structure.md`](./docs/project-structure.md)
- [`docs/TODO.md`](./docs/TODO.md)

## License

This project is licensed under the GNU Affero General Public License v3.0. See [`LICENSE`](./LICENSE).
