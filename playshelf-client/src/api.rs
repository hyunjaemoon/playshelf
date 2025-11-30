use serde::{Deserialize, Serialize};

const API_BASE_URL: &str = "http://localhost:8081";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GameData {
    pub id: u64,
    pub name: String,
    pub platforms: Vec<String>,
    pub first_release_date: String,
    pub genres: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GamesResponse {
    pub count: usize,
    pub games: Vec<GameData>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Fetch all games from the server
pub async fn fetch_games() -> Result<Vec<GameData>, String> {
    let url = format!("{}/games", API_BASE_URL);
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch games: {}", e))?;

    if response.status().is_success() {
        let games_response: GamesResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        Ok(games_response.games)
    } else {
        let error_response: ErrorResponse = response
            .json()
            .await
            .unwrap_or(ErrorResponse {
                error: "Unknown error".to_string(),
            });
        Err(error_response.error)
    }
}

/// Search for games by query string
pub async fn search_games(query: String) -> Result<Vec<GameData>, String> {
    let url = format!("{}/games/search?query={}", API_BASE_URL, urlencoding::encode(&query));
    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to search games: {}", e))?;

    if response.status().is_success() {
        let games_response: GamesResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        Ok(games_response.games)
    } else {
        let error_response: ErrorResponse = response
            .json()
            .await
            .unwrap_or(ErrorResponse {
                error: "Unknown error".to_string(),
            });
        Err(error_response.error)
    }
}

