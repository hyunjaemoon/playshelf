use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::igdb::manager::GameData;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct User {
    pub id: u128,
    pub username: String,
    pub name: String,
    pub description: String,
    pub games: Vec<GameData>,
}

impl User {
    pub fn new(username: String, name: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4().as_u128(),
            username,
            name,
            description,
            games: Vec::new(),
        }
    }

    pub fn add_game(&mut self, game: GameData) {
        self.games.push(game);
    }

    pub fn remove_game(&mut self, game: GameData) {
        self.games.retain(|g| g.id != game.id);
    }

    pub fn get_games(&self) -> &Vec<GameData> {
        &self.games
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_USERNAME: &str = "testuser";
    const TEST_NAME: &str = "Test User";
    const TEST_DESCRIPTION: &str = "Test description";

    #[test]
    fn test_user_new() {
        let user = User::new(
            TEST_USERNAME.to_string(),
            TEST_NAME.to_string(),
            TEST_DESCRIPTION.to_string(),
        );
        assert_eq!(user.username, TEST_USERNAME);
        assert_eq!(user.name, TEST_NAME);
        assert_eq!(user.description, TEST_DESCRIPTION);
    }

    #[test]
    fn test_user_golden_file() {
        use std::fs;
        use serde_json::json;

        // Read the golden file (located in workspace root, one level up from package root)
        let golden_file_path = "../sample_users.json";
        let golden_content = fs::read_to_string(golden_file_path)
            .expect("Failed to read golden file");
        
        // Parse the JSON structure
        let golden_json: serde_json::Value = serde_json::from_str(&golden_content)
            .expect("Failed to parse golden file JSON");
        
        // Extract users array
        let users_array = golden_json["users"].as_array()
            .expect("Golden file should contain a 'users' array");
        
        // Deserialize each user
        let users: Vec<User> = users_array
            .iter()
            .map(|user_json| serde_json::from_value::<User>(user_json.clone())
                .expect("Failed to deserialize user"))
            .collect();
        
        // Verify field values match the golden file
        assert_eq!(users.len(), 1, "Should have exactly one user");
        let user = &users[0];
        
        // Check user fields
        assert_eq!(user.username, "hyunjaemoon", "Username should match");
        assert_eq!(user.name, "Hyun Jae Moon's Library", "Name should match");
        assert_eq!(
            user.description,
            "A library of games that Hyun Jae Moon has played",
            "Description should match"
        );
        
        // Check games
        assert_eq!(user.games.len(), 2, "User should have 2 games");
        
        // Check first game
        let game1 = &user.games[0];
        assert_eq!(game1.id, 0, "First game id should be 0");
        assert_eq!(
            game1.name,
            "The Legend of Zelda: Breath of the Wild",
            "First game name should match"
        );
        assert_eq!(
            game1.first_release_date,
            "1488499200",
            "First game release date should match"
        );
        assert_eq!(
            game1.platforms,
            vec!["Nintendo Wii U", "Nintendo Switch"],
            "First game platforms should match"
        );
        assert_eq!(
            game1.genres,
            vec!["Action-Adventure", "Open-World"],
            "First game genres should match"
        );
        
        // Check second game
        let game2 = &user.games[1];
        assert_eq!(game2.id, 1, "Second game id should be 1");
        assert_eq!(game2.name, "Persona 5", "Second game name should match");
        assert_eq!(
            game2.first_release_date,
            "1473897600",
            "Second game release date should match"
        );
        assert_eq!(
            game2.platforms,
            vec![
                "PlayStation 4",
                "PlayStation 5",
                "PC",
                "Nintendo Switch",
                "Xbox Series X/S"
            ],
            "Second game platforms should match"
        );
        assert_eq!(
            game2.genres,
            vec!["Role-Playing", "Action-Adventure", "Visual Novel"],
            "Second game genres should match"
        );
        
        // Serialize back to JSON
        let serialized_json = json!({
            "users": users
        });
        
        // Compare with golden file (parse both as Value for comparison)
        let golden_value: serde_json::Value = serde_json::from_str(&golden_content)
            .expect("Failed to parse golden file");
        
        assert_eq!(serialized_json, golden_value, "Serialized users should match golden file");
    }
}
