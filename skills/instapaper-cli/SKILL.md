---
name: instapaper-cli
description: >
  Use the `instapaper` CLI to interact with the Instapaper API for saving,
  managing, and reading articles. Trigger whenever the user mentions Instapaper,
  wants to save a URL, manage bookmarks, organize folders, or work with highlights.
  Also trigger for queries about reading progress, starring/archiving articles,
  or fetching article text content.
---

# Instapaper CLI Skill

This skill helps you use the `instapaper` CLI tool to interact with the [Instapaper](https://www.instapaper.com) API.

## Prerequisites

- The `instapaper` binary must be built or available in PATH
- Environment variables must be set:
  - `INSTAPAPER_OAUTH_CONSUMER_KEY` — OAuth consumer key
  - `INSTAPAPER_OAUTH_CONSUMER_SECRET` — OAuth consumer secret
  - `INSTAPAPER_USERNAME` — Instapaper username/email (for initial auth)
  - `INSTAPAPER_PASSWORD` — Instapaper password (for initial auth)
- Run `instapaper auth` first to authenticate and save OAuth tokens

## Command Reference

All commands output JSON by default.

### Authentication

| Command | Description |
|---------|-------------|
| `instapaper auth --username <email> --password <pass>` | Authenticate via xAuth and save OAuth tokens |

### Account

| Command | Description |
|---------|-------------|
| `instapaper verify-credentials` | Verify credentials and get current user info |

### Bookmarks

| Command | Description |
|---------|-------------|
| `instapaper list-bookmarks` | List unread bookmarks (options: `--limit`, `--folder-id`, `--tag`, `--have`, `--highlights`) |
| `instapaper add-bookmark --url <url>` | Add a new bookmark (options: `--title`, `--description`, `--folder-id`, `--archived`, `--tags`, `--content`) |
| `instapaper delete-bookmark <id>` | Permanently delete a bookmark |
| `instapaper star-bookmark <id>` | Star a bookmark |
| `instapaper unstar-bookmark <id>` | Unstar a bookmark |
| `instapaper archive-bookmark <id>` | Move bookmark to Archive |
| `instapaper unarchive-bookmark <id>` | Move bookmark to Unread |
| `instapaper move-bookmark <id> <folder-id>` | Move bookmark to a folder |
| `instapaper get-bookmark-text <id>` | Get bookmark's processed text HTML (option: `--instaparser-api-key`) |
| `instapaper update-read-progress <id> <progress>` | Update reading progress (0.0 to 1.0, option: `--progress-timestamp`) |

### Folders

| Command | Description |
|---------|-------------|
| `instapaper list-folders` | List user-created folders |
| `instapaper add-folder <title>` | Create a new folder |
| `instapaper delete-folder <id>` | Delete a folder |
| `instapaper set-folder-order <order>` | Re-order folders (format: `folder_id:position`, comma-separated) |

### Highlights

| Command | Description |
|---------|-------------|
| `instapaper list-highlights <bookmark-id>` | List highlights for a bookmark |
| `instapaper create-highlight <bookmark-id> <text>` | Create a new highlight (option: `--position`) |
| `instapaper delete-highlight <id>` | Delete a highlight |

## Examples

### Authenticate

Use the environment variable (recommended so the password doesn't appear in shell history):

```bash
export INSTAPAPER_PASSWORD="mypassword"
instapaper auth --username user@example.com
```

Or omit `--password` to enter it securely at the prompt:

```bash
instapaper auth --username user@example.com
```

### List unread bookmarks

```bash
instapaper list-bookmarks --limit 10
```

### List starred bookmarks

```bash
instapaper list-bookmarks --folder-id starred
```

### Save a URL

```bash
instapaper add-bookmark --url https://example.com/article --title "Article Title"
```

### Star a bookmark

```bash
instapaper star-bookmark 12345
```

### Archive a bookmark

```bash
instapaper archive-bookmark 12345
```

### Update reading progress

```bash
instapaper update-read-progress 12345 0.5
```

### Create a folder

```bash
instapaper add-folder "Work Articles"
```

### Highlight text from an article

```bash
instapaper create-highlight 12345 "This is an important passage"
```

## Folder IDs

Special folder IDs for `list-bookmarks`:
- `unread` — Unread articles (default)
- `starred` — Starred articles
- `archive` — Archived articles
- Numeric ID — User-created folder

## Tips

- OAuth tokens are stored in `~/.config/instapaper-cli/token.json`
- The `--have` parameter for `list-bookmarks` accepts comma-separated bookmark IDs to skip already-synced items
- Progress is a float between 0.0 (start) and 1.0 (complete)
- Tags for `add-bookmark` should be JSON: `[{"name": "Tag1"}, {"name": "Tag2"}]`
- Instapaper requires OAuth 1.0a with HMAC-SHA1 for all requests
