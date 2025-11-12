use serde::{Deserialize, Serialize};

const IGDB_URL: &str = "https://api.igdb.com";

#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platforms: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_release_date: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genres: Option<Vec<u64>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Platform {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Genre {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

pub struct IGDBManager {
    client_id: String,
    access_token: String,
}

impl IGDBManager {
    pub fn new(client_id: String, access_token: String) -> Self {
        Self { client_id, access_token }
    }

    async fn get_platforms_by_ids(&self, ids: Vec<u64>) -> Result<Vec<Platform>, Box<dyn std::error::Error>> {
        let url = format!("{}/v4/platforms", IGDB_URL);
        let client = reqwest::Client::new();
        let ids_str = format!(
            "({});",
            ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
        let response = client
            .post(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", &self.access_token))
            .body(format!("fields name; where id = {};", ids_str))
            .send()
            .await?;
        let platforms: Vec<Platform> = response.json().await?;
        Ok(platforms)
    }

    async fn get_genres_by_ids(&self, ids: Vec<u64>) -> Result<Vec<Genre>, Box<dyn std::error::Error>> {
        let url = format!("{}/v4/genres", IGDB_URL);
        let client = reqwest::Client::new();
        let ids_str = format!(
            "({});",
            ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
        let response = client
            .post(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", &self.access_token))
            .body(format!("fields name; where id = {};", ids_str))
            .send()
            .await?;
        let genres: Vec<Genre> = response.json().await?;
        Ok(genres)
    }

    // TODO: Implement pagination
    pub async fn get_games(&self) -> Result<Vec<Game>, Box<dyn std::error::Error>> {
        // Declare games endpoint URL 
        let url = format!("{}/v4/games", IGDB_URL);

        let client = reqwest::Client::new();
        let response = client
            .post(&url)
            .header("Client-ID", &self.client_id)
            .header("Authorization", format!("Bearer {}", &self.access_token))
            .body("fields name,platforms,first_release_date,genres;")
            .send()
            .await?;

        let games: Vec<Game> = response.json().await?;

        for game in &games {
            let platforms = self.get_platforms_by_ids(game.platforms.clone().unwrap_or_default()).await?;
            println!("Game: {:?}", game.name.clone().unwrap_or_default());
            println!("Platforms: {:?}", platforms.iter().map(|p| p.name.clone().unwrap_or_default()).collect::<Vec<_>>());
            println!("First Release Date: {:?}", game.first_release_date.unwrap_or_default());
            let genres = self.get_genres_by_ids(game.genres.clone().unwrap_or_default()).await.unwrap_or(Vec::new());
            println!("Genres: {:?}", genres.iter().map(|g| g.name.clone().unwrap_or_default()).collect::<Vec<_>>());
        }
        Ok(games)
    }
}