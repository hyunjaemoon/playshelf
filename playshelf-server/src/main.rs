mod igdb;

use axum::{
    routing::get,
    Router,
};
use dotenv::dotenv;
use igdb::IGDBManager;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
struct TwitchTokenResponse {
    access_token: String,
    expires_in: u64,
    token_type: String,
}

async fn authenticate_twitch() -> Result<TwitchTokenResponse, Box<dyn std::error::Error>> {
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
    
    println!("Successfully authenticated with Twitch!");
    println!("Token expires in {} seconds", token_response.expires_in);
    
    Ok(token_response)
}

// Migrate from axum to Dioxus
#[tokio::main]
async fn main() {
    dotenv().ok();

    // Authenticate with Twitch before setting up the app
    let _twitch_credentials = authenticate_twitch()
        .await
        .expect("Failed to authenticate with Twitch");

    // TODO: Setup the database using the IGDB API
    let client_id = env::var("TWITCH_CLIENT_ID")
        .expect("TWITCH_CLIENT_ID must be set in .env file");
    let igdb_manager = IGDBManager::new(client_id, _twitch_credentials.access_token);
    let _search_result = igdb_manager.search_games("Zelda".to_string()).await.expect("Failed to get game list");

    // build our application with a single route
    // let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run our app with hyper, listening globally on port 3000
    // let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    // axum::serve(listener, app).await.unwrap();
}