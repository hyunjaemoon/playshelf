use serde::{Deserialize, Serialize};

/// Base URL for the IGDB API
const IGDB_URL: &str = "https://api.igdb.com";

/// Represents a game from the IGDB API
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Game {
    /// Unique identifier for the game
    pub id: u64,
    /// Name of the game
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// List of platform IDs this game is available on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platforms: Option<Vec<u64>>,
    /// Unix timestamp of the first release date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_release_date: Option<u64>,
    /// List of genre IDs associated with this game
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genres: Option<Vec<u64>>,
}

/// Represents a platform from the IGDB API
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Platform {
    /// Unique identifier for the platform
    pub id: u64,
    /// Name of the platform
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Represents a genre from the IGDB API
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Genre {
    /// Unique identifier for the genre
    pub id: u64,
    /// Name of the genre
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Represents a search result item from the IGDB API
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SearchItem {
    /// Unique identifier for the search item
    pub id: u64,
    /// Game ID associated with this search item (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game: Option<u64>,
    /// Name of the search item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
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
    pub fn new(client_id: String, access_token: String) -> Self {
        Self {
            client_id,
            access_token,
            client: reqwest::Client::new(),
        }
    }

    /// Makes an authenticated POST request to the IGDB API
    async fn make_request(
        &self,
        endpoint: &str,
        body: String,
    ) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
        let url = format!("{}/{}", IGDB_URL, endpoint);
        let response = self
            .client
            .post(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", &self.access_token))
            .body(body)
            .send()
            .await?;
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
    async fn get_platforms_by_ids(&self, ids: Vec<u64>) -> Result<Vec<Platform>, Box<dyn std::error::Error>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let ids_str = self.format_ids(&ids);
        let body = format!("fields name; where id = {};", ids_str);
        let response = self.make_request("v4/platforms", body).await?;
        let platforms: Vec<Platform> = response.json().await?;
        Ok(platforms)
    }

    /// Retrieves a single game by its ID
    async fn get_game_by_id(&self, id: u64) -> Result<Game, Box<dyn std::error::Error>> {
        let body = format!("fields *; where id = {};", id);
        let response = self.make_request("v4/games", body).await?;
        let mut games: Vec<Game> = response.json().await?;
        match games.pop() {
            Some(game) => Ok(game),
            None => Err(format!("No game found for ID: {}", id).into()),
        }
    }

    /// Retrieves genre information by a list of genre IDs
    async fn get_genres_by_ids(&self, ids: Vec<u64>) -> Result<Vec<Genre>, Box<dyn std::error::Error>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let ids_str = self.format_ids(&ids);
        let body = format!("fields name; where id = {};", ids_str);
        let response = self.make_request("v4/genres", body).await?;
        let genres: Vec<Genre> = response.json().await?;
        Ok(genres)
    }

    /// Retrieves all games from the IGDB API
    pub async fn get_games(&self) -> Result<Vec<Game>, Box<dyn std::error::Error>> {
        let body = "fields name,platforms,first_release_date,genres;".to_string();
        let response = self.make_request("v4/games", body).await?;
        let games: Vec<Game> = response.json().await?;

        // Print game details with platform and genre information
        for game in &games {
            let platforms = self
                .get_platforms_by_ids(game.platforms.clone().unwrap_or_default())
                .await?;
            println!("Game: {:?}", game.name.clone().unwrap_or_default());
            println!(
                "Platforms: {:?}",
                platforms
                    .iter()
                    .map(|p| p.name.clone().unwrap_or_default())
                    .collect::<Vec<_>>()
            );
            println!(
                "First Release Date: {:?}",
                game.first_release_date.unwrap_or_default()
            );
            let genres = self
                .get_genres_by_ids(game.genres.clone().unwrap_or_default())
                .await
                .unwrap_or_default();
            println!(
                "Genres: {:?}",
                genres
                    .iter()
                    .map(|g| g.name.clone().unwrap_or_default())
                    .collect::<Vec<_>>()
            );
        }
        Ok(games)
    }

    /// Searches for games by query string
    pub async fn search_games(&self, query: String) -> Result<Vec<Game>, Box<dyn std::error::Error>> {
        let body = format!("search \"{}\"; fields game,name; limit 10;", query);
        let response = self.make_request("v4/search", body).await?;
        let search_items: Vec<SearchItem> = response.json().await?;

        let mut games: Vec<Game> = Vec::new();
        for search_item in &search_items {
            if let Some(game_id) = search_item.game {
                let game = self.get_game_by_id(game_id).await?;
                println!("Game: {:?}", game.name.clone().unwrap_or_default());
                games.push(game);
            }
        }
        Ok(games)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the format_ids helper method
    #[test]
    fn test_format_ids() {
        let manager = IGDBManager::new("test_client".to_string(), "test_token".to_string());
        
        // Test with multiple IDs
        let ids = vec![1, 2, 3, 4];
        assert_eq!(manager.format_ids(&ids), "(1,2,3,4)");
        
        // Test with single ID
        let ids = vec![42];
        assert_eq!(manager.format_ids(&ids), "(42)");
        
        // Test with empty vector
        let ids: Vec<u64> = vec![];
        assert_eq!(manager.format_ids(&ids), "");
    }
}