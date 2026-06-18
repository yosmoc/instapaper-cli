# Instapaper CLI

A command-line interface for the [Instapaper API](https://www.instapaper.com/developers/v1/full-api), built in Rust.

## Features

- OAuth 1.0a authentication via xAuth
- Manage bookmarks: add, delete, star, archive, move
- Organize folders: create, delete, reorder
- Highlights: create, list, delete
- Track reading progress
- Fetch article text content

## Installation

### From source

```bash
cargo install --path .
```

### From GitHub Releases

Download pre-built binaries from [Releases](https://github.com/yosmoc/instapaper-cli/releases).

## Usage

### 1. Set environment variables

```bash
export INSTAPAPER_OAUTH_CONSUMER_KEY="your_consumer_key"
export INSTAPAPER_OAUTH_CONSUMER_SECRET="your_consumer_secret"
export INSTAPAPER_USERNAME="your_email@example.com"
export INSTAPAPER_PASSWORD="your_password"
```

### 2. Authenticate

Use the environment variable (recommended):

```bash
instapaper auth --username $INSTAPAPER_USERNAME
```

Or omit `--password` to enter it securely at the prompt.

For scripts only, you can still pass it on the command line:

```bash
instapaper auth --username $INSTAPAPER_USERNAME --password $INSTAPAPER_PASSWORD
```

### 3. Use the CLI

```bash
# List unread bookmarks
instapaper list-bookmarks --limit 10

# Save a URL
instapaper add-bookmark --url https://example.com --title "Example"

# Star a bookmark
instapaper star-bookmark 12345

# Archive a bookmark
instapaper archive-bookmark 12345

# List folders
instapaper list-folders

# Create a folder
instapaper add-folder "Work"

# List highlights
instapaper list-highlights 12345

# Create a highlight
instapaper create-highlight 12345 "Important text"
```

## Development

```bash
cargo build
cargo test
cargo fmt --all
```

## API Documentation

See the [full Instapaper API documentation](https://www.instapaper.com/developers/v1/full-api).

## License

MIT
