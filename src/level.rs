use std::path::Path;
use tokio::fs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Level {
    pub name: String,
    pub author: String,
    pub rate: i32
}

impl Level {
    pub async fn load_level(path: &Path) -> Level {
        let data = fs::read(path).await.expect("Unable to read file");
        let level: Level = serde_json::from_slice(&*data).unwrap();
        level
    }
}
