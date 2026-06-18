use clap::{Parser, Subcommand};
use instapaper_cli::client::ApiClient;
use instapaper_cli::commands::{account, auth, bookmarks, folders, highlights};

const DEFAULT_BASE_URL: &str = "https://www.instapaper.com";

#[derive(Parser)]
#[command(name = "instapaper-cli")]
#[command(about = "CLI for Instapaper API")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, default_value = DEFAULT_BASE_URL, env = "INSTAPAPER_BASE_URL")]
    base_url: String,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Authenticate via xAuth and save OAuth tokens")]
    Auth {
        #[arg(
            long,
            env = "INSTAPAPER_USERNAME",
            help = "Instapaper username (email)"
        )]
        username: String,
        #[arg(
            long,
            env = "INSTAPAPER_PASSWORD",
            help = "Instapaper password (will prompt securely if not provided)"
        )]
        password: Option<String>,
    },
    #[command(about = "Verify credentials and get current user")]
    VerifyCredentials,
    #[command(about = "List unread bookmarks")]
    ListBookmarks {
        #[arg(long, help = "Number of bookmarks (1-500, default 25)")]
        limit: Option<i32>,
        #[arg(long, help = "Folder: unread, starred, archive, or folder_id")]
        folder_id: Option<String>,
        #[arg(long, help = "Filter by tag name")]
        tag: Option<String>,
        #[arg(long, help = "Comma-separated bookmark IDs already synced")]
        have: Option<String>,
        #[arg(long, help = "Dash-separated highlight IDs already synced")]
        highlights: Option<String>,
    },
    #[command(about = "Add a new bookmark")]
    AddBookmark {
        #[arg(long, help = "URL to save (required unless using private source)")]
        url: Option<String>,
        #[arg(long, help = "Title of the bookmark")]
        title: Option<String>,
        #[arg(long, help = "Description or summary")]
        description: Option<String>,
        #[arg(long, help = "Folder ID to save to")]
        folder_id: Option<i64>,
        #[arg(long, help = "Archive the bookmark on add")]
        archived: bool,
        #[arg(long, help = "Tags as JSON array: [{'name': 'Tag'}]")]
        tags: Option<String>,
        #[arg(long, help = "Full HTML content of the page")]
        content: Option<String>,
        #[arg(long, help = "Mark as private with source label")]
        is_private_from_source: Option<String>,
        #[arg(
            long,
            help = "Resolve redirects (default true)",
            default_value = "true"
        )]
        resolve_final_url: bool,
    },
    #[command(about = "Delete a bookmark permanently")]
    DeleteBookmark {
        #[arg(help = "Bookmark ID")]
        bookmark_id: i64,
    },
    #[command(about = "Star a bookmark")]
    StarBookmark {
        #[arg(help = "Bookmark ID")]
        bookmark_id: i64,
    },
    #[command(about = "Unstar a bookmark")]
    UnstarBookmark {
        #[arg(help = "Bookmark ID")]
        bookmark_id: i64,
    },
    #[command(about = "Archive a bookmark")]
    ArchiveBookmark {
        #[arg(help = "Bookmark ID")]
        bookmark_id: i64,
    },
    #[command(about = "Unarchive a bookmark (move to Unread)")]
    UnarchiveBookmark {
        #[arg(help = "Bookmark ID")]
        bookmark_id: i64,
    },
    #[command(about = "Move a bookmark to a folder")]
    MoveBookmark {
        #[arg(help = "Bookmark ID")]
        bookmark_id: i64,
        #[arg(help = "Target folder ID")]
        folder_id: i64,
    },
    #[command(about = "Get bookmark's processed text HTML")]
    GetBookmarkText {
        #[arg(help = "Bookmark ID")]
        bookmark_id: i64,
        #[arg(long, help = "Instaparser API key")]
        instaparser_api_key: Option<String>,
    },
    #[command(about = "Update reading progress on a bookmark")]
    UpdateReadProgress {
        #[arg(help = "Bookmark ID")]
        bookmark_id: i64,
        #[arg(help = "Progress (0.0 to 1.0)")]
        progress: f64,
        #[arg(long, help = "Unix timestamp of progress")]
        progress_timestamp: Option<i64>,
    },
    #[command(about = "List user-created folders")]
    ListFolders,
    #[command(about = "Create a new folder")]
    AddFolder {
        #[arg(help = "Folder title")]
        title: String,
    },
    #[command(about = "Delete a folder")]
    DeleteFolder {
        #[arg(help = "Folder ID")]
        folder_id: i64,
    },
    #[command(about = "Re-order folders")]
    SetFolderOrder {
        #[arg(help = "Order as folder_id:position pairs, comma-separated (e.g., 100:1,200:2)")]
        order: String,
    },
    #[command(about = "List highlights for a bookmark")]
    ListHighlights {
        #[arg(help = "Bookmark ID")]
        bookmark_id: i64,
    },
    #[command(about = "Create a new highlight")]
    CreateHighlight {
        #[arg(help = "Bookmark ID")]
        bookmark_id: i64,
        #[arg(help = "Highlighted text")]
        text: String,
        #[arg(long, help = "Position in content (0-indexed, default 0)")]
        position: Option<i64>,
    },
    #[command(about = "Delete a highlight")]
    DeleteHighlight {
        #[arg(help = "Highlight ID")]
        highlight_id: i64,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let client = ApiClient::from_env(cli.base_url)?;

    match cli.command {
        Commands::Auth { username, password } => {
            let password = match password {
                Some(p) => p,
                None => rpassword::prompt_password("Instapaper password: ")
                    .map_err(|e| format!("failed to read password: {e}"))?,
            };
            let _ = auth::xauth_login(&client, &username, &password).await?;
            println!("Authentication successful!");
            println!("OAuth tokens saved to config directory.");
        }
        Commands::VerifyCredentials => {
            let user = account::verify_credentials(&client).await?;
            println!("{}", serde_json::to_string_pretty(&user)?);
        }
        Commands::ListBookmarks {
            limit,
            folder_id,
            tag,
            have,
            highlights,
        } => {
            let result = bookmarks::list_bookmarks(
                &client,
                limit,
                folder_id.as_deref(),
                tag.as_deref(),
                have.as_deref(),
                highlights.as_deref(),
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Commands::AddBookmark {
            url,
            title,
            description,
            folder_id,
            archived,
            tags,
            content,
            is_private_from_source,
            resolve_final_url,
        } => {
            let url = url.ok_or("URL is required unless using private source")?;
            let bookmark = bookmarks::add_bookmark(
                &client,
                &url,
                title.as_deref(),
                description.as_deref(),
                folder_id,
                if archived { Some(true) } else { None },
                tags.as_deref(),
                content.as_deref(),
                is_private_from_source.as_deref(),
                Some(resolve_final_url),
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&bookmark)?);
        }
        Commands::DeleteBookmark { bookmark_id } => {
            bookmarks::delete_bookmark(&client, bookmark_id).await?;
            println!("Bookmark {} deleted.", bookmark_id);
        }
        Commands::StarBookmark { bookmark_id } => {
            let bookmark = bookmarks::star_bookmark(&client, bookmark_id).await?;
            println!("{}", serde_json::to_string_pretty(&bookmark)?);
        }
        Commands::UnstarBookmark { bookmark_id } => {
            let bookmark = bookmarks::unstar_bookmark(&client, bookmark_id).await?;
            println!("{}", serde_json::to_string_pretty(&bookmark)?);
        }
        Commands::ArchiveBookmark { bookmark_id } => {
            let bookmark = bookmarks::archive_bookmark(&client, bookmark_id).await?;
            println!("{}", serde_json::to_string_pretty(&bookmark)?);
        }
        Commands::UnarchiveBookmark { bookmark_id } => {
            let bookmark = bookmarks::unarchive_bookmark(&client, bookmark_id).await?;
            println!("{}", serde_json::to_string_pretty(&bookmark)?);
        }
        Commands::MoveBookmark {
            bookmark_id,
            folder_id,
        } => {
            let bookmark = bookmarks::move_bookmark(&client, bookmark_id, folder_id).await?;
            println!("{}", serde_json::to_string_pretty(&bookmark)?);
        }
        Commands::GetBookmarkText {
            bookmark_id,
            instaparser_api_key,
        } => {
            let html =
                bookmarks::get_bookmark_text(&client, bookmark_id, instaparser_api_key.as_deref())
                    .await?;
            println!("{}", html);
        }
        Commands::UpdateReadProgress {
            bookmark_id,
            progress,
            progress_timestamp,
        } => {
            let timestamp = progress_timestamp.unwrap_or_else(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64
            });
            let bookmark =
                bookmarks::update_read_progress(&client, bookmark_id, progress, timestamp).await?;
            println!("{}", serde_json::to_string_pretty(&bookmark)?);
        }
        Commands::ListFolders => {
            let folder_list = folders::list_folders(&client).await?;
            println!("{}", serde_json::to_string_pretty(&folder_list)?);
        }
        Commands::AddFolder { title } => {
            let folder = folders::add_folder(&client, &title).await?;
            println!("{}", serde_json::to_string_pretty(&folder)?);
        }
        Commands::DeleteFolder { folder_id } => {
            folders::delete_folder(&client, folder_id).await?;
            println!("Folder {} deleted.", folder_id);
        }
        Commands::SetFolderOrder { order } => {
            let folder_list = folders::set_folder_order(&client, &order).await?;
            println!("{}", serde_json::to_string_pretty(&folder_list)?);
        }
        Commands::ListHighlights { bookmark_id } => {
            let highlight_list = highlights::list_highlights(&client, bookmark_id).await?;
            println!("{}", serde_json::to_string_pretty(&highlight_list)?);
        }
        Commands::CreateHighlight {
            bookmark_id,
            text,
            position,
        } => {
            let highlight =
                highlights::create_highlight(&client, bookmark_id, &text, position).await?;
            println!("{}", serde_json::to_string_pretty(&highlight)?);
        }
        Commands::DeleteHighlight { highlight_id } => {
            highlights::delete_highlight(&client, highlight_id).await?;
            println!("Highlight {} deleted.", highlight_id);
        }
    }

    Ok(())
}
