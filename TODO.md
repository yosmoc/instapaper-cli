# Instapaper CLI - TODO

## Project Setup

- [x] Create TODO.md
- [x] Initialize Rust project with cargo
- [x] Add dependencies: clap, reqwest, serde, serde_json, tokio, thiserror, oauth1-request
- [x] Configure INSTAPAPER_OAUTH_CONSUMER_KEY, INSTAPAPER_OAUTH_CONSUMER_SECRET environment variables
- [x] Configure INSTAPAPER_USERNAME, INSTAPAPER_PASSWORD for xAuth
- [x] Set up prek pre-commit hooks (secretlint, cargo-fmt, cargo-test, pinact, oxfmt)
- [x] Set up GitHub Actions CI/CD (test, release, secretlint, oxfmt)
- [x] Create AGENTS.md, skills/instapaper-cli/SKILL.md, README.md
- [x] Add crates.io installation instructions to README.md and SKILL.md

## Commands Implemented (17 total)

### Authentication

- [x] `auth` — Get OAuth access token via xAuth (stores token/secret locally) — **Live verified**

### Account

- [x] `verify-credentials` — Returns the currently logged in user — **Live verified**

### Bookmarks

- [x] `list-bookmarks` — List unread bookmarks (options: `--limit`, `--folder-id`, `--tag`, `--have`, `--highlights`) — **Live verified**
- [x] `add-bookmark` — Add a new bookmark (options: `--url`, `--title`, `--description`, `--folder-id`, `--archived`, `--tags`) — **Wiremock tested**
- [x] `delete-bookmark` — Permanently delete a bookmark — **Wiremock tested**
- [x] `star-bookmark` — Star a bookmark — **Wiremock tested**
- [x] `unstar-bookmark` — Unstar a bookmark — **Wiremock tested**
- [x] `archive-bookmark` — Move bookmark to Archive — **Wiremock tested**
- [x] `unarchive-bookmark` — Move bookmark to Unread — **Wiremock tested**
- [x] `move-bookmark` — Move bookmark to a folder — **Wiremock tested**
- [x] `get-bookmark-text` — Get bookmark's processed text HTML — **Wiremock tested**
- [x] `update-read-progress` — Update reading progress on a bookmark — **Wiremock tested**

### Folders

- [x] `list-folders` — List user-created folders — **Live verified**
- [x] `add-folder` — Create a new folder — **Wiremock tested**
- [x] `delete-folder` — Delete a folder — **Wiremock tested**
- [x] `set-folder-order` — Re-order folders — **Wiremock tested**

### Highlights

- [x] `list-highlights` — List highlights for a bookmark — **Wiremock tested**
- [x] `create-highlight` — Create a new highlight — **Wiremock tested**
- [x] `delete-highlight` — Delete a highlight — **Wiremock tested**

## Implementation Approach

- TDD: Write tests first, then implement
- One command at a time
- Commit after each command passes tests
- Use reqwest for HTTP calls
- Use oauth1-request for OAuth 1.0a signing
- Use clap for CLI argument parsing
- Output JSON by default
- Store OAuth tokens in ~/.config/instapaper-cli/token.json

## Current Status

All 17 commands implemented. 19 tests passing. `auth`, `verify-credentials`, `list-bookmarks`, and `list-folders` verified against live API.

## API Notes

- OAuth 1.0a with HMAC-SHA1 only
- All requests via POST with parameters in request body
- OAuth params in Authorization header
- HTTPS required
- xAuth is the only way to get access tokens
- Output is JSON array with type field (user, bookmark, folder, error, highlight, meta)
- `bookmarks/list` returns array of typed objects: `[{"type": "meta"}, {"type": "user", ...}, {"type": "bookmark", ...}]`
- `bookmarks/get_text` returns raw HTML
- `starred` and `archive` fields are strings (`"0"`/`"1"`), not booleans
- Tokens stored in `~/.config/instapaper-cli/token.json` with restricted permissions (0600)
