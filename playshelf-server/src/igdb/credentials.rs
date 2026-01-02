use serde::Deserialize;
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

async fn get_aws_secret(name: &str) -> String {
    // Authenticate with AWS
    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region("us-east-2") // Ensure this matches your SSM region
        .load()
        .await;
    let client = aws_sdk_ssm::Client::new(&config);

    let resp = client
        .get_parameter()
        .name(name)
        .with_decryption(true) // Crucial for SecureString
        .send()
        .await
        .expect("Failed to fetch parameter");

    resp.parameter()
        .and_then(|p| p.value())
        .unwrap_or("")
        .to_string()
}

pub async fn authenticate_twitch() -> Result<TwitchCredentials, Box<dyn std::error::Error + Send + Sync>> {
    let client_id = get_aws_secret("/playshelf/prod/twitch-client-id").await;
    let client_secret = get_aws_secret("/playshelf/prod/twitch-client-secret").await;

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