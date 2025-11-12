use serde::Deserialize;
use std::env;
use std::time::SystemTime;
use tokio::time::Duration;

#[derive(Debug, Clone)]
pub struct TwitchCredentials {
    client_id: String,
    access_token: String,
    expires_at: SystemTime,
}

impl TwitchCredentials {
    pub fn get_client_id_and_access_token(&self) -> (String, String) {
        (self.client_id.clone(), self.access_token.clone())
    }

    pub fn get_expires_at(&self) -> SystemTime {
        self.expires_at
    }
}

#[derive(Debug, Deserialize)]
struct TwitchTokenResponse {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

pub async fn authenticate_twitch() -> Result<TwitchCredentials, Box<dyn std::error::Error>> {
    let client_id = env::var("TWITCH_CLIENT_ID")
        .expect("TWITCH_CLIENT_ID must be set in .env file");
    let client_secret = env::var("TWITCH_CLIENT_SECRET")
        .expect("TWITCH_CLIENT_SECRET must be set in .env file");

    let url = format!(
        "https://id.twitch.tv/oauth2/token?client_id={}&client_secret={}&grant_type=client_credentials",
        client_id, client_secret
    );

    let response = reqwest::Client::new()
        .post(&url)
        .send()
        .await?;

    let token_response: TwitchTokenResponse = response.json().await?;
    
    Ok(TwitchCredentials {
        client_id,
        access_token: token_response.access_token,
        expires_at: SystemTime::now() + Duration::from_secs(token_response.expires_in),
        })
}