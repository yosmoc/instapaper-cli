# AGENTS.md

## Development Workflow

- **TDD first**: Write tests, then implement. Use `wiremock` for HTTP mocking.
- **After implementing a new function**: `cargo fmt --all` → `cargo clippy --all-targets --all-features` → `cargo test` →
  update `TODO.md`, `README.md`, and `skills/instapaper-cli/SKILL.md`.
- **Pre-commit hooks**: secretlint, cargo-fmt, cargo-clippy, cargo-test, pinact, oxfmt.
  All must pass.

## Key Commands

```bash
cargo test              # Run all tests
cargo test <pattern>    # Run matching tests
cargo fmt --all         # Format code
cargo fmt --all -- --check  # Check formatting (CI)
cargo clippy --all-targets --all-features         # Run clippy
cargo clippy --all-targets --all-features -- -D warnings  # Fail on warnings (CI)
```

## Project Structure

- Single crate: `instapaper-cli` (binary name `instapaper`)
- `src/main.rs` — CLI entry point with `clap` subcommands
- `src/commands/<feature>.rs` — one file per feature, tests inline in same file
- `src/client.rs` — `ApiClient` with OAuth 1.0a signing
- `skills/instapaper-cli/` — opencode skill for using this CLI

## API Notes

- **Auth**: OAuth 1.0a with HMAC-SHA1 only
- All requests via POST with parameters in request body
- OAuth params in Authorization header
- HTTPS required
- xAuth is the only way to get access tokens
- Output is JSON array with type field (user, bookmark, folder, error, highlight, meta)
- Some endpoints return non-standard formats:
  - `bookmarks/list` returns array of typed objects: `[{"type": "meta"}, {"type": "user", ...}, {"type": "bookmark", ...}]`
  - `bookmarks/get_text` returns raw HTML
- Tokens stored in `~/.config/instapaper-cli/token.json` with restricted permissions (0600)

## Testing

- Tests use `wiremock` for HTTP mocking.
- Shared test token via `crate::commands::test_token()`
- Each command file has its own `#[cfg(test)]` module.

## Environment Variables

- `INSTAPAPER_OAUTH_CONSUMER_KEY` — OAuth consumer key (required)
- `INSTAPAPER_OAUTH_CONSUMER_SECRET` — OAuth consumer secret (required)
- `INSTAPAPER_USERNAME` — Instapaper username/email (for auth command)
- `INSTAPAPER_PASSWORD` — Instapaper password (for auth command)
- `INSTAPAPER_BASE_URL` — API base URL (default: https://www.instapaper.com)

## Release

- Push `v*` tag to trigger cross-platform build (Linux, macOS, Windows).
- Uses `dtolnay/rust-toolchain` and `taiki-e/install-action` (cross).
- **Do not use** `actions-rs/*` — deprecated.

## Conventions

- `skills/`, `node_modules/`, `TODO.md` are excluded from oxfmt (see `.oxfmtrc.json`).
- `package.json` exists only for dev tooling (secretlint, oxfmt).
