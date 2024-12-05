use crate::setup_oauth_client;
use crate::TokenStore;
use dotenv::dotenv;
use google_calendar3::yup_oauth2;
use log::{error, info};
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, CsrfToken, Scope};
use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};
use warp::Filter;
// use oauth2::ClientId;
// use oauth2::ClientSecret;

pub struct ServerConfig {
    host: [u8; 4],
    pub port: u16,
    pub application_secret: yup_oauth2::ApplicationSecret,
    pub config_dir: String,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok();
        let port = env::var("OAUTH_CALLBACK_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()?;
        let host = [127, 0, 0, 1];
        let client_id = env::var("GOOGLE_CLIENT_ID")?;
        let client_secret = env::var("GOOGLE_CLIENT_SECRET")?;
        let application_secret = yup_oauth2::ApplicationSecret {
            client_id,
            client_secret,
            ..Default::default()
        };
        let config_dir = "~/.google-service-cli".to_string(); // Replace with your desired config directory

        Ok(ServerConfig {
            host,
            port,
            application_secret,
            config_dir,
        })
    }
}

pub async fn handle_auth(
    token_store: &TokenStore,
    config: &ServerConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // env_logger::init();
    info!("Starting OAuth authentication flow");

    let client = setup_oauth_client(config)?;

    info!("Server will listen on port {}", config.port);

    // Set up a temporary server to handle the callback
    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // Start server to handle the OAuth callback
    let routes = warp::path("oauth")
        .and(warp::path("callback"))
        .and(warp::query::<HashMap<String, String>>())
        .and_then(move |params: HashMap<String, String>| {
            let tx = tx.clone();

            async move {
                info!("Received callback with parameters: {:?}", params);
                if let Some(code) = params.get("code") {
                    info!("Got authorization code, length: {}", code.len());
                    let mut sender = tx.lock().await;
                    if let Some(tx) = sender.take() {
                        info!("Sending code through channel");
                        tx.send(code.to_string()).ok();
                    } else {
                        error!("Channel sender was already taken");
                    }
                } else {
                    error!("No code parameter in callback");
                }
                Ok::<_, Infallible>(warp::reply::html(
                    "Authentication successful! You can close this window.",
                ))
            }
        });

    let (addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown((config.host, config.port), async {
            shutdown_rx.await.ok();
        });

    info!("Server bound to address: {:?}", addr);

    let server_handle = tokio::spawn(server);
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    info!("Server started successfully");

    // Start the server in the background
    // let server_handle = tokio::spawn(server);

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/calendar".to_string(),
        ))
        .url();

    info!("Opening browser with URL: {}", auth_url);
    webbrowser::open(auth_url.as_str())?;

    info!("Waiting for authorization code...");

    // Wait for the callback
    let code = rx.await?;
    info!("Received authorization code, shutting down server");

    let _ = shutdown_tx.send(());

    // Wait for server to shut down
    server_handle.await?;
    info!("Server shut down successfully");

    // Exchange code for token
    let token = client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(async_http_client)
        .await?;

    token_store.save_token("default_user", &token)?;
    info!("Authentication completed successfully");
    Ok(())
}
