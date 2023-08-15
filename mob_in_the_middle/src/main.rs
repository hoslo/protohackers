use std::net::SocketAddr;
mod strict_lines_codec;
use anyhow::Result;
use fancy_regex::Regex;
use futures::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use strict_lines_codec::StrictLinesCodec;
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

lazy_static! {
    static ref REGEX_BOGUSCOIN: Regex = Regex::new(r"(?<= |^)7[a-zA-Z0-9]{25,34}(?= |$)").unwrap();
}

const TARGET_BOGUSCOIN: &str = "7YWHMfk9JZe0LM0g1ZauHuiSxhI";

fn hack_boguscoin_message(message: &String) -> String {
    REGEX_BOGUSCOIN
        .replace_all(&message, TARGET_BOGUSCOIN)
        .to_string()
}

async fn handle_client(client_stream: TcpStream, addr: SocketAddr) -> Result<()> {
    let (client_read, client_write) = client_stream.into_split();
    let mut client_read = FramedRead::new(client_read, LinesCodec::new());
    let mut client_write = FramedWrite::new(client_write, LinesCodec::new());

    let server_stream = TcpStream::connect("chat.protohackers.com:16963").await?;
    let (server_read, server_write) = server_stream.into_split();
    let mut server_read = FramedRead::new(server_read, LinesCodec::new());
    let mut server_write = FramedWrite::new(server_write, LinesCodec::new());

    tokio::spawn(async move {
        while let Some(message) = client_read.next().await {
            match message {
                Ok(message) => {
                    #[cfg(debug_assertions)]
                    println!("{addr} --> {message}");

                    server_write.send(hack_boguscoin_message(&message)).await?;
                }
                Err(err) => {
                    // TODO: Abort server task
                    return Err(err);
                }
            }
        }

        Ok(())
    });

    tokio::spawn(async move {
        while let Some(message) = server_read.next().await {
            match message {
                Ok(message) => {
                    #[cfg(debug_assertions)]
                    println!("{addr} <-- {message}");

                    client_write.send(hack_boguscoin_message(&message)).await?;
                }
                Err(err) => {
                    // TODO: Abort client task
                    return Err(err);
                }
            }
        }

        Ok(())
    });

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    loop {
        let socket = listener.accept().await?.0;
        socket.set_nodelay(true)?;
        let addr = socket.peer_addr()?;
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, addr).await {
                println!("an error occured; error = {:?}", e);
            }
        });
    }
}
