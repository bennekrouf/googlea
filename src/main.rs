mod create_event;
mod handle_auth;
mod token_handler; // Import your token handler module

use crate::create_event::create_event;
use crate::handle_auth::handle_auth;
use dotenv::dotenv;
use env_logger::{Builder, Env};
use handle_auth::ServerConfig;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use std::env;
use token_handler::TokenStore;
// use rustls::crypto::CryptoProvider;
use rustls::crypto::ring::default_provider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Builder::from_env(Env::default().default_filter_or("debug"))
        .format_timestamp_secs()
        .init();
    let provider = default_provider();
    provider
        .install_default()
        .expect("Failed to install crypto provider");
    // Install the default crypto provider before any TLS operations
    // CryptoProvider::install_default().expect("Failed to install crypto provider");

    let token_store = TokenStore::new()?;
    let mut args = std::env::args().skip(1);
    let config = ServerConfig::from_env()?;

    match args.next().as_deref() {
        Some("auth") => handle_auth(&token_store, &config).await?,
        Some("create-event") => {
            // Get the description from the command line
            match args.next() {
                Some(description) => {
                    println!("Attempting to create event: {}", description);
                    create_event(&config, &token_store, "default_user", &description).await?
                }
                None => {
                    println!("Error: Event description is required");
                    println!("Usage: cargo run -- create-event \"Your event description\"");
                }
            }
        }
        _ => {
            println!("Usage:");
            println!("  Authenticate: cargo run -- auth");
            println!("  Create event: cargo run -- create-event \"Your event description\"");
        }
    }
    Ok(())
}

fn get_env_var(key: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Try loading .env file first
    dotenv().ok();

    // Try getting from environment (includes both .env and export)
    env::var(key).map_err(|e| format!("Missing {key}: {e}").into())
}

fn setup_oauth_client(config: &ServerConfig) -> Result<BasicClient, Box<dyn std::error::Error>> {
    let client_id = get_env_var("GOOGLE_CLIENT_ID")?;
    let client_secret = get_env_var("GOOGLE_CLIENT_SECRET")?;

    let redirect_url = format!("http://localhost:{}/oauth/callback", config.port);

    Ok(BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?,
        Some(TokenUrl::new(
            "https://oauth2.googleapis.com/token".to_string(),
        )?),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url)?))
}
