use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;

use crate::igdb::manager::{IGDBManager, GameData};

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
        Ok(games) => {
            (StatusCode::OK, Json(serde_json::json!({
                "count": games.len(),
                "games": games
            })))
        }
        Err(error_msg) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": error_msg
                })),
            )
        }
    }
}

