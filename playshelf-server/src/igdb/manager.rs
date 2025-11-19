use std::time::SystemTime;

use super::credentials::authenticate_twitch;
use serde::{Deserialize, Serialize};

/// Base URL for the IGDB API
const IGDB_URL: &str = "https://api.igdb.com";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GameData {
    pub id: u64,
    pub name: String,
    pub platforms: Vec<String>,
    pub first_release_date: String,
    pub genres: Vec<String>,
}

/// Represents a game from the IGDB API
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct Game {
    /// Unique identifier for the game
    id: u64,
    /// Name of the game
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// List of platform IDs this game is available on
    #[serde(skip_serializing_if = "Option::is_none")]
    platforms: Option<Vec<u64>>,
    /// Unix timestamp of the first release date 
    #[serde(skip_serializing_if = "Option::is_none")]
    first_release_date: Option<u64>,
    /// List of genre IDs associated with this game
    #[serde(skip_serializing_if = "Option::is_none")]
    genres: Option<Vec<u64>>,
}

/// Represents a platform from the IGDB API
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct Platform {
    /// Unique identifier for the platform
    id: u64,
    /// Name of the platform
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

/// Represents a genre from the IGDB API
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct Genre {
    /// Unique identifier for the genre
    id: u64,
    /// Name of the genre
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

/// Manager for interacting with the IGDB API
/// 
/// This struct handles authentication and provides methods to query
/// games, platforms, genres, and search functionality.
pub struct IGDBManager {
    /// Twitch Client ID for IGDB API authentication
    client_id: String,
    /// Access token for IGDB API authentication
    access_token: String,
    /// Reusable HTTP client for making requests
    client: reqwest::Client,
}

impl IGDBManager {
    /// Creates a new IGDBManager instance
    pub fn new() -> Self {
        Self {
            client_id: String::new(),
            access_token: String::new(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn authenticate(&mut self) -> Result<SystemTime, Box<dyn std::error::Error + Send + Sync>> {
        let twtich_credentials = authenticate_twitch().await.expect("Failed to authenticate with Twitch");
        let (client_id, access_token) = twtich_credentials.get_client_id_and_access_token();
        self.client_id = client_id;
        self.access_token = access_token;
        Ok(twtich_credentials.get_expires_at())
    }

    /// Makes an authenticated POST request to the IGDB API
    async fn make_request(
        &self,
        endpoint: &str,
        body: String,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/{}", IGDB_URL, endpoint);
        let response = self
            .client
            .post(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", &self.access_token))
            .body(body)
            .send()
            .await?;
        
        // Check if the response is an error
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("IGDB API error ({}): {}", status, error_text).into());
        }
        
        Ok(response)
    }

    /// Formats a list of IDs into an IGDB API query format
    fn format_ids(&self, ids: &[u64]) -> String {
        if ids.is_empty() {
            return String::new();
        }
        format!(
            "({})",
            ids.iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }

    /// Retrieves platform information by a list of platform IDs
    async fn get_platforms_by_ids(&self, ids: Vec<u64>) -> Result<Vec<Platform>, Box<dyn std::error::Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let ids_str = self.format_ids(&ids);
        let body = format!("fields name; where id = {};", ids_str);
        let response = self.make_request("v4/platforms", body).await?;
        let platforms: Vec<Platform> = response.json().await?;
        Ok(platforms)
    }

    /// Retrieves genre information by a list of genre IDs
    async fn get_genres_by_ids(&self, ids: Vec<u64>) -> Result<Vec<Genre>, Box<dyn std::error::Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let ids_str = self.format_ids(&ids);
        let body = format!("fields name; where id = {};", ids_str);
        let response = self.make_request("v4/genres", body).await?;
        let genres: Vec<Genre> = response.json().await?;
        Ok(genres)
    }

    /// Converts a Game to GameData using pre-fetched platform and genre maps
    fn game_data_from_game_with_maps(
        &self,
        game: &Game,
        platform_map: &std::collections::HashMap<u64, String>,
        genre_map: &std::collections::HashMap<u64, String>,
    ) -> GameData {
        let platforms: Vec<String> = game
            .platforms
            .as_ref()
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| platform_map.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default();
        
        let genres: Vec<String> = game
            .genres
            .as_ref()
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| genre_map.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default();

        GameData {
            id: game.id.clone(),
            name: game.name.clone().unwrap_or_default(),
            platforms,
            first_release_date: game.first_release_date.clone().unwrap_or_default().to_string(),
            genres,
        }
    }
    
    async fn games_data_from_games(&self, games: Vec<Game>) -> Result<Vec<GameData>, Box<dyn std::error::Error + Send + Sync>> {
        // Collect all unique platform and genre IDs
        let mut platform_ids = std::collections::HashSet::new();
        let mut genre_ids = std::collections::HashSet::new();
        
        for game in &games {
            if let Some(platforms) = &game.platforms {
                platform_ids.extend(platforms.iter());
            }
            if let Some(genres) = &game.genres {
                genre_ids.extend(genres.iter());
            }
        }

        // Fetch all platforms and genres in parallel
        let (platforms_result, genres_result) = tokio::join!(
            self.get_platforms_by_ids(platform_ids.into_iter().collect()),
            self.get_genres_by_ids(genre_ids.into_iter().collect())
        );

        let platforms = platforms_result?;
        let genres = genres_result?;

        // Build lookup maps
        let platform_map: std::collections::HashMap<u64, String> = platforms
            .into_iter()
            .filter_map(|p| p.name.map(|name| (p.id, name)))
            .collect();
        
        let genre_map: std::collections::HashMap<u64, String> = genres
            .into_iter()
            .filter_map(|g| g.name.map(|name| (g.id, name)))
            .collect();

        // Convert games to GameData using the maps
        let games_data: Vec<GameData> = games
            .iter()
            .map(|game| self.game_data_from_game_with_maps(game, &platform_map, &genre_map))
            .collect();

        Ok(games_data)
    }

    /// Retrieves all games from the IGDB API
    pub async fn get_games(&self) -> Result<Vec<GameData>, Box<dyn std::error::Error + Send + Sync>> {
        let body = "fields name,platforms,first_release_date,genres;".to_string();
        let response = self.make_request("v4/games", body).await?;
        let games: Vec<Game> = response.json().await?;
        self.games_data_from_games(games).await
    }

    /// Searches for games by query string
    pub async fn search_games(&self, query: String) -> Result<Vec<GameData>, Box<dyn std::error::Error + Send + Sync>> {
        let body = format!("search \"{}\"; fields name,platforms,first_release_date,genres;", query);
        let response = self.make_request("v4/games", body).await?;
        let games: Vec<Game> = response.json().await?;
        self.games_data_from_games(games).await
    }
}