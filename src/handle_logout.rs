use crate::TokenStore;
use log::{error, info};

pub async fn handle_logout(token_store: &TokenStore, user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting logout process for user: {}", user_id);

    match token_store.remove_token(user_id) {
        Ok(_) => {
            info!("Successfully logged out user {}", user_id);
            println!("Logout successful. All tokens have been removed.");
            Ok(())
        },
        Err(e) => {
            error!("Failed to logout user {}: {:?}", user_id, e);
            Err(e)
        }
    }
}
