use std::net::SocketAddr;
mod strict_lines_codec;
use anyhow::Result;
use fancy_regex::Regex;
use futures::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use strict_lines_codec::StrictLinesCodec;
use tokio::{net::TcpStream, select};
use tokio_util::codec::{Framed, FramedRead, FramedWrite, LinesCodec};

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
    let mut client_framed = Framed::new(client_stream, StrictLinesCodec::new());

    let server_stream = TcpStream::connect("chat.protohackers.com:16963").await?;
    let mut server_framed = Framed::new(server_stream, StrictLinesCodec::new());

    loop {
        select! {
            client_message = client_framed.next() => {
                match client_message {
                    Some(Ok(message)) => {
                        let message = hack_boguscoin_message(&message);
                        #[cfg(debug_assertions)]
                        println!("{addr} --> {message}");
                        server_framed.send(message).await?;
                    },
                    Some(Err(e)) => {
                        println!("{}: error reading from client: {}", addr, e);
                        return Err(e.into());
                    },
                    None => {
                        println!("{}: client disconnected", addr);
                    }
                }
            },
            server_message = server_framed.next() => {
                match server_message {
                    Some(Ok(message)) => {
                        #[cfg(debug_assertions)]
                        println!("{addr} <-- {message}");
                        client_framed.send(message).await?;
                    },
                    Some(Err(e)) => {
                        println!("{}: error reading from server: {}", addr, e);
                        return Err(e.into());
                    },
                    None => {
                        println!("{}: server disconnected", addr);
                    }
                }
            }
        }
    }
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
