use axum::{extract::{Query, State}, http::StatusCode, response::Json};
use std::sync::Arc;
use serde::Deserialize;

use crate::igdb::manager::{GameData, IGDBManager};

#[derive(Deserialize)]
pub struct SearchQuery {
    pub query: String,
}

/// Handler for GET /games endpoint
/// Returns a list of games from IGDB
pub async fn get_games_handler(
    State(manager): State<Arc<IGDBManager>>,
) -> (StatusCode, Json<serde_json::Value>) {
    // Convert error to String immediately to ensure Send trait
    let games_result: Result<Vec<GameData>, String> = manager
        .get_games()
        .await
        .map_err(|e| format!("Error fetching games: {}", e));

    match games_result {
        Ok(games) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "count": games.len(),
                "games": games
            })),
        ),
        Err(error_msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": error_msg
            })),
        ),
    }
}

/// Handler for GET /games/search endpoint
/// Searches for games by query string
pub async fn search_games_handler(
    State(manager): State<Arc<IGDBManager>>,
    Query(params): Query<SearchQuery>,
) -> (StatusCode, Json<serde_json::Value>) {
    let search_result: Result<Vec<GameData>, String> = manager
        .search_games(params.query)
        .await
        .map_err(|e| format!("Error searching games: {}", e));
    match search_result {
        Ok(games) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "count": games.len(),
                "games": games
            }))
        ),
        Err(error_msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": error_msg
            })),
        ),
    }
}
