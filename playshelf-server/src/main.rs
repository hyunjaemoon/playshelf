mod args;
mod igdb;

use axum::{
    routing::get,
    Router,
};
use chrono::{DateTime, Utc};
use dotenv::dotenv;
use igdb::manager::IGDBManager;

use crate::args::Args;
use clap::Parser;


async fn main_dev() {
    dotenv().ok();

    // Authenticate with Twitch before setting up the app
    let mut igdb_manager = IGDBManager::new();
    let expires_at = igdb_manager.authenticate().await.expect("Failed to authenticate with Twitch");
    let datetime = DateTime::<Utc>::from(expires_at);
    println!("Token expires at: {}\n", datetime.format("%Y-%m-%d %H:%M:%S UTC"));

    // TODO: Remove this after deployment
    let games = igdb_manager.get_games().await.expect("Failed to get game list");
    println!("Games: {:?}\n", games);
    let search_result = igdb_manager.search_games("Zelda".to_string()).await.expect("Failed to search for games");
    println!("Search result: {:?}\n", search_result);
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
        // build our application with a single route
        let app = Router::new().route("/", get(|| async { "Hello, World!" }));

        // run our app with hyper, listening globally on port 3000
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}