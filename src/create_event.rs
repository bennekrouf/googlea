extern crate google_calendar3 as calendar3;
use crate::setup_oauth_client;
use crate::ServerConfig;
use crate::TokenStore;
use calendar3::{CalendarHub, Result};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use log::{error, info};
use oauth2::TokenResponse;

pub async fn create_event(
    config: &ServerConfig,
    token_store: &TokenStore,
    user_id: &str,
    description: &str,
) -> Result<()> {
    info!("Starting create_event process for user: {}", user_id);

    let oauth_client = setup_oauth_client(config);
    info!("OAuth client setup completed");

    // Get the stored token or refresh if needed
    info!("Attempting to get token from token store");
    let token = match token_store
        .ensure_valid_token(user_id, &oauth_client.unwrap())
        .await
    {
        Ok(t) => {
            info!("Successfully retrieved token");
            t
        }
        Err(e) => {
            error!("Failed to get token: {:?}", e);
            return Ok(()); // Early return if token retrieval fails
        }
    };

    info!("Setting up HTTPS connector");
    // Fix the HTTPS connector creation
    let https = HttpsConnectorBuilder::new()
        .with_native_roots()
        .unwrap_or_else(|e| {
            error!("Failed to get native roots: {:?}", e);
            panic!("Failed to get native roots: {:?}", e);
        })
        .https_only()
        .enable_http1()
        .build();
    info!("HTTPS connector created successfully");

    // Create the hyper client with the HTTPS connector
    info!("Creating HTTP client");
    let client = Client::builder(TokioExecutor::new()).build(https);

    // Create the hub with the authenticated client
    info!("Creating Calendar Hub with token");
    let token_secret = token.access_token().secret().to_owned();
    info!(
        "Token available (first 10 chars): {}...",
        token_secret.chars().take(10).collect::<String>()
    );

    let hub = CalendarHub::new(client, token_secret);
    info!("Calendar Hub created successfully");

    // Create the event details
    info!("Creating event object with description: {}", description);
    let event = google_calendar3::api::Event {
        summary: Some(description.to_string()),
        start: Some(google_calendar3::api::EventDateTime {
            date_time: Some(
                chrono::Utc::now()
                    .checked_add_signed(chrono::Duration::hours(1))
                    .unwrap(),
            ),
            time_zone: Some("UTC".to_string()),
            date: None,
        }),
        end: Some(google_calendar3::api::EventDateTime {
            date_time: Some(
                chrono::Utc::now()
                    .checked_add_signed(chrono::Duration::hours(2))
                    .unwrap(),
            ),
            time_zone: Some("UTC".to_string()),
            date: None,
        }),
        description: Some(format!("Created by CLI tool: {}", description)),
        location: None,
        ..Default::default()
    };
    info!("Event object created successfully");

    // Insert the event into the primary calendar
    info!("Sending event creation request to Google Calendar API...");
    let insert_result = hub.events().insert(event, "primary").doit().await;
    info!("Received response from Calendar API");

    match insert_result {
        Ok((_response, event)) => {
            info!("Event created successfully!");
            info!("Event ID: {}", event.id.unwrap_or_default());
            info!("HTML link: {}", event.html_link.unwrap_or_default());
            Ok(())
        }
        Err(e) => {
            error!("Failed to create event. Error details:");
            error!("Error type: {:?}", std::any::type_name_of_val(&e));
            error!("Error message: {}", e);
            error!("Full error: {:?}", e);
            Err(e)
        }
    }
}
