mod level;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use std::env;
use std::fs::{read, read_dir};
use std::net::IpAddr;
use std::os::linux::fs;
use std::os::unix::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use serde::de::Unexpected::Str;
use serde_json::Value::Array;
use tokio::sync::Mutex;
use tokio::task;
use tokio::time::{Instant, sleep};
use crate::level::Level;

#[tokio::main]
async fn main() {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:42069".to_string());

    let listener = TcpListener::bind(&addr).await.unwrap();

    let addr_list: Arc<Mutex<Vec<IpAddr>>> = Arc::new(Mutex::new(Vec::new()));
    let mut cleared_at = Arc::new(Instant::now());

    let addr_clone = addr_list.clone();
    let clear = task::spawn(async move {
        loop {
            sleep(Duration::from_secs(5*60)).await;
            addr_clone.lock().await.clear();
        }
    });
    loop {
        let current_addr_list = addr_list.clone();
        let (mut socket, _) = listener.accept().await.unwrap();

        tokio::spawn( async move {
            let mut buffer = vec![0; (1024*1024)*20];

            loop {
                let n = socket.read(&mut buffer).await.unwrap();
                if n == 0 {
                    return
                }

                match buffer[0] {
                    0 => {
                        let version = [2];
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
                            if index < ((page - 1) * 4) as usize {
                                continue
                            }
                            let level = Level::load_level(path.unwrap().path().as_path()).await;
                            level_array.push(level);
                            println!("{}", index);
                            if index >= ((page - 1) * 4 + 3) as usize {
                                break;
                            }
                        }

                        let answer = serde_json::to_vec(&level_array).unwrap();
                        socket.write_all(&answer).await.expect("failed to respond with level array");
                    }
                    2 => {
                        let data = buffer[1..n].to_vec();
                        let mut name = String::new();
                        data.iter().for_each(|c| name.push(*c as char));

                        let answer = read(format!("./level_data/{}", name)).unwrap();

                        println!("{}", answer.len());

                        socket.write_all(&answer).await.expect("failed to respond")
                    }
                    3 => {
                        let data = buffer[1..n].to_vec();

                        let level_option: Option<Level> = serde_json::from_slice(&data)
                            .unwrap_or_else(|_| return None);

                        match level_option {
                            None => {
                                let _ = socket.write(&[0 as u8; 1]);
                            }
                            Some(mut level) => {
                                let mut is_present = false;
                                let mut ips = current_addr_list.lock().await;
                                for ip in ips.iter() {
                                    if socket.peer_addr().unwrap().ip() == *ip {
                                        is_present = true;
                                    }
                                }
                                let mut buffer = [0_u8; 1];
                                if is_present {
                                    buffer[0] = 2;
                                    let _ = socket.write_all(&buffer).await;
                                } else {
                                    level.author = socket.peer_addr().expect("").ip().to_string();
                                    let response = level.save_level().await;
                                    buffer[0] = response;
                                    if response == 0  {
                                        let _ = socket.write_all(&buffer).await;
                                    } else {
                                        ips.push(socket.peer_addr().unwrap().ip());
                                        let _ = socket.write_all(&buffer).await;
                                    }
                                }
                                drop(ips)
                            }
                        }
                    }
                    4 => {
                        let name = String::from_utf8(
                            buffer[1..10].to_vec().into_iter().filter(|x| x != &0_u8).collect::<Vec<u8>>()
                        ).expect("name");
                        let data = buffer[10..n].to_vec();

                        let size_as_u32 = Level::save_level_data(format!("./level_data/{}", name), format!("./levels/{}", name),&data).await;
                        let size: [u8; 4] = size_as_u32.to_be_bytes();
                        println!("{} {} {} {}", size[0], size[1], size[2], size[3]);

                        socket.write_all(&size).await.unwrap();
                    }
                    _ => {}
                }
            }
        });
    }
}
