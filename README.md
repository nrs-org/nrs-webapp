# nrs-webapp

Monolithic web application for NRS, written in Rust. It is intended to implement
[NRS](https://github.com/nrs-org/nrs) version 3.0.

> idk maybe i will introduce vibe-coding into this in the future, but well i
> gotta make a good project structure first

## Instructions

Specify **dev-only** environment variables in `.cargo/config.toml`:

```toml
[env]
RUST_LOG = "nrs_webapp=debug"
SERVICE_DB_URL = "postgres://nrs_webapp_user:dev_only_pwd@localhost:3622/nrs_webapp_dev"
STATIC_SERVE_DIR = "nrs-webapp/static"

# app secrets, should be stored securely in prod
SERVICE_PASSWORD_PEPPER = "jIYb9KCTzZpdqbKM_e9DwcJzCefzvnAUGZzD_WH029OXbyMnn5nyUWerd_NPWrVNKKwDM6rEzmipNuFNdJ8vKej1XEb5dD2NzKwSWTNqiQKgRph6rVABcIrybPAjC31wN-7rfpQYoB1oZyYb5wl1meNgDzwjpLfpBL90R89BGx8="
SERVICE_COOKIE_KEY = "ub3lweHTzpWiVZyVrskQjcsRqRlNTFGbHMaTYec97AIFJ2wg05FYyXVjI9as9MqE19Ur5ztCWfZDPMjwxycZZlD48SeMbzH0ppxJSLgxCYTotPO79Tf1lH7IkM27ujBiiHS_MUMgFA9zKdDC9bYhHEXug6XFt-2_ZkrmNEhU_Wo="
SERVICE_SESSION_EXPIRY_SECS = "1800"
```

> [!WARNING]
> In production, make sure to store secrets securely. DO NOT use the above
> dev-only values in a production environment.

To generate password pepper and secret values, use the `nrs-webapp-keygen`
binary crate.

```sh
# Generate a new secret, encoded in URL-safe base64
cargo run --bin nrs-webapp-keygen

# Pipe the output to clipboard (diagnostics output are printed to stderr)
cargo run --bin nrs-webapp-keygen | wl-copy
```

Development database are wiped and re-created on each test run, DO NOT USE YOUR
LOCAL DATABASE, if it contains important data. The easiest way to set up a
separated DB is via Docker:

```sh
docker run --rm --name pg -p 3622:5432 -e POSTGRES_PASSWORD=password -d postgres:17
```

This will start a PostgreSQL 17 instance, listening on port `3622`, with a
`postgres` superuser with password `password`. DO NOT CHANGE THE PASSWORD,
the `_dev_utils` crate expects this specific one.

Connect to this DB via

```sh
docker exec -it -u postgres pg psql
```

For live-reloading during development, install [bacon](https://dystroy.org/bacon/).

Here are four tasks to run during development:

- `bacon dev`: live-reloads the server on code changes
- `bacon quick-dev`: re-run the `quick_dev.rs` example for quick testing
- `bacon build-core-wasm`: build `nrs-webapp-core` crate as wasm for frontend
  code.
- `pnpm --prefix nrs-webapp-frontend tailwind`: run Tailwind CSS in watch mode
  for frontend styles.
