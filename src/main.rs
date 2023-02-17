mod level;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use std::env;
use std::fs::{read, read_dir};
use std::os::linux::fs;
use std::path::Path;
use serde::de::Unexpected::Str;
use crate::level::Level;

#[tokio::main]
async fn main() {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:42069".to_string());

    let listener = TcpListener::bind(&addr).await.unwrap();

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();

        tokio::spawn( async move {
            let mut buffer = vec![0; 1024];

            loop {
                let n = socket.read(&mut buffer).await.unwrap();
                if n == 0 {
                    return
                }

                match buffer[0] {
                    0 => {
                        let version = "1.0".as_bytes();
                        socket.write_all(&version).await.expect("failed to write data")
                    }
                    1 => {
                        let page = buffer[1];
                        if page == 0 {
                            return;
                        }

                        let paths = read_dir("./levels/").unwrap();
                        let mut level_array: Vec<Level> = vec![];
                        for (index, path) in paths.enumerate() {
                            if index < ((page - 1) * 5) as usize {
                                continue
                            }
                            let level = Level::load_level(path.unwrap().path().as_path()).await;
                            level_array.push(level);
                            if index > ((page - 1) * 5 + 5) as usize {
                                break;
                            }
                        }

                        let answer = serde_json::to_vec(&level_array).unwrap();
                        socket.write_all(&answer).await.expect("failed to respond with level array");
                    }
                    2 => {
                        let data = buffer[1..buffer.len()].to_vec()
                            .into_iter()
                            .filter(|x| x != &0_u8)
                            .collect::<Vec<u8>>();
                        let mut name = String::new();
                        data.iter().for_each(|c| name.push(*c as char));

                        let answer = read(format!("./level_data/{}", name)).unwrap();

                        println!("{}", answer.len());

                        socket.write_all(&answer).await.expect("failed to respond")
                    }
                    _ => {}
                }
            }
        });
    }
}
