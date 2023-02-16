mod commands;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use std::env;
use crate::commands::Commands;

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

                if buffer[0] == Commands::VersionCheck as u8 {
                    let version = "1.0".as_bytes();

                    socket.write_all(&version).await.expect("failed to write data")
                }
            }
        });
    }
}
