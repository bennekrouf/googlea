use oauth2::basic::BasicTokenResponse;
use oauth2::reqwest::async_http_client;
use oauth2::TokenResponse;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
pub struct StoredToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
}

pub struct TokenStore {
    pub db: sled::Db,
}

impl TokenStore {
    pub fn new() -> Result<Self, sled::Error> {
        let db = sled::open("token_store")?;
        Ok(Self { db })
    }

    pub fn save_token(
        &self,
        user_id: &str,
        token: &BasicTokenResponse,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let stored_token = StoredToken {
            access_token: token.access_token().secret().clone(),
            refresh_token: token.refresh_token().map(|t| t.secret().clone()),
            expires_at: token.expires_in().map(|d| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    + d.as_secs()
            }),
        };

        let token_bytes = bincode::serialize(&stored_token)?;
        self.db.insert(user_id.as_bytes(), token_bytes)?;
        self.db.flush()?;

        Ok(())
    }

    fn load_token(&self, user_id: &str) -> Option<BasicTokenResponse> {
        let bytes = self.db.get(user_id.as_bytes()).ok()??;

        match bincode::deserialize::<StoredToken>(&bytes) {
            Ok(stored) => {
                // Check expiration
                if let Some(expires_at) = stored.expires_at {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    if now >= expires_at {
                        return None; // Token expired
                    }
                }
                Some(BasicTokenResponse::new(
                    oauth2::AccessToken::new(stored.access_token),
                    oauth2::basic::BasicTokenType::Bearer,
                    oauth2::EmptyExtraTokenFields {},
                ))
            }
            Err(_) => None,
        }
    }

    pub async fn ensure_valid_token(
        &self,
        user_id: &str,
        client: &oauth2::basic::BasicClient,
    ) -> Result<BasicTokenResponse, Box<dyn std::error::Error>> {
        if let Some(token) = self.load_token(user_id) {
            if let Some(expires_in) = token.expires_in() {
                if expires_in.as_secs() < 300 {
                    // Less than 5 minutes remaining
                    if let Some(refresh_token) = token.refresh_token() {
                        let new_token = client
                            .exchange_refresh_token(refresh_token)
                            .request_async(async_http_client)
                            .await?;
                        self.save_token(user_id, &new_token)?;
                        return Ok(new_token);
                    }
                }
            }
            Ok(token)
        } else {
            Err("No token found".into())
        }
    }
}
