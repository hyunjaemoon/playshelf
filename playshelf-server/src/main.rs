mod args;
mod handlers;
mod igdb;
mod user;

use axum::{
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use igdb::manager::{IGDBManager, GameData};
use std::sync::Arc;

use crate::args::Args;
use clap::Parser;

/// Prints a list of games in a formatted, readable way
fn print_game_data(games: &[GameData], title: &str) {
    println!("{} ({} games):\n", title, games.len());
    for (i, game) in games.iter().enumerate() {
        println!("{}. {}", i + 1, game.name);
        if !game.platforms.is_empty() {
            println!("   Platforms: {}", game.platforms.join(", "));
        }
        if !game.first_release_date.is_empty() && game.first_release_date != "0" {
            match game.first_release_date.parse::<i64>() {
                Ok(timestamp) => {
                    // IGDB first_release_date is a Unix timestamp in seconds
                    if let Some(release_date) = DateTime::<Utc>::from_timestamp(timestamp, 0) {
                        println!("   Release Date: {} (timestamp: {})", release_date.format("%Y-%m-%d"), timestamp);
                    } else {
                        println!("   Release Date: Invalid timestamp ({})", timestamp);
                    }
                }
                Err(e) => {
                    eprintln!("   Release Date: Failed to parse '{}' as timestamp: {}", game.first_release_date, e);
                }
            }
        }
        if !game.genres.is_empty() {
            println!("   Genres: {}", game.genres.join(", "));
        }
        println!();
    }
}

async fn main_dev() {
    dotenv().ok();

    // Authenticate with Twitch before setting up the app
    let mut igdb_manager = IGDBManager::new();
    let expires_at = igdb_manager.authenticate().await.expect("Failed to authenticate with Twitch");
    let datetime = DateTime::<Utc>::from(expires_at);
    println!("Token expires at: {}\n", datetime.format("%Y-%m-%d %H:%M:%S UTC"));

    // Get List of Games
    let games = igdb_manager.get_games().await.expect("Failed to get game list");
    print_game_data(&games, "Found games");

    // Search for Games
    let search_result = igdb_manager.search_games("Zelda".to_string()).await.expect("Failed to search for games");
    print_game_data(&search_result, "Search results for 'Zelda'");
}

// Migrate from axum to Dioxus
#[tokio::main]
async fn main() {
    let flags = Args::parse();
    if flags.dev {
        println!("Running in development mode\n");
        main_dev().await;
    } else {
        println!("Running in production mode\n");
        
        // Authenticate with Twitch before setting up the app
        dotenv().ok();
        let mut igdb_manager = IGDBManager::new();
        let expires_at = igdb_manager.authenticate().await.expect("Failed to authenticate with Twitch");
        let datetime = DateTime::<Utc>::from(expires_at);
        println!("Token expires at: {}\n", datetime.format("%Y-%m-%d %H:%M:%S UTC"));
        
        // Wrap IGDBManager in Arc to share across requests
        let igdb_manager = Arc::new(igdb_manager);
        
        // build our application with routes that have access to IGDBManager
        let app = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .route("/games", get(handlers::get_games_handler))
            .route("/games/search", get(handlers::search_games_handler))
            .with_state(igdb_manager);

        // run our app with hyper, listening globally on port 8080
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}