use std::path::Path;
use tokio::fs;
use serde::{Deserialize, Serialize};
use tokio::fs::OpenOptions;

#[derive(Serialize, Deserialize)]
pub struct Level {
    pub name: String,
    pub author: String,
    pub rate: i32,
    pub size: u64
}

impl Level {
    pub async fn load_level(path: &Path) -> Level {
        let data = fs::read(path).await.expect("Unable to read file");
        let level: Level = serde_json::from_slice(&*data).unwrap();
        level
    }

    pub async fn save_level(&self) -> u8 {
        let file = OpenOptions::new().read(true).open(format!("./levels/{}", self.name)).await;
        match file {
            Ok(_) => 0,
            Err(_) => {
                let json = serde_json::to_vec(self).expect("TAK");
                fs::write( format!("./levels/{}", self.name), json).await.unwrap();
                1
            }
        }
    }

    pub async fn save_level_data(path: String, path2: String, data: &Vec<u8>) {
        let file1 = OpenOptions::new().read(true).open(&path2).await;
        match file1 {
            Ok(file) => {},
            Err(_) => { return}
        }

        let file = OpenOptions::new().read(true).open(&path).await;
        match file {
            Ok(_) => {}
            Err(_) => {
                fs::write(path, data).await.unwrap();
            }
        }
    }
}
