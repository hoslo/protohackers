use std::net::SocketAddr;

use anyhow::Result;
use fancy_regex::Regex;
use futures::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use tokio::{net::TcpStream, select};
use tokio_util::codec::{Framed, LinesCodec};

lazy_static! {
    static ref REGEX_BOGUSCOIN: Regex = Regex::new(r"(?<= |^)7[a-zA-Z0-9]{25,34}(?= |$)").unwrap();
}

const TARGET_BOGUSCOIN: &str = "7YWHMfk9JZe0LM0g1ZauHuiSxhI";

fn replace_boguscoin(message: String) -> String {
    REGEX_BOGUSCOIN
        .replace_all(&message, TARGET_BOGUSCOIN)
        .to_string()
}

async fn handle_client(socket: TcpStream, addr: SocketAddr) -> Result<()> {
    let up_socket = TcpStream::connect("chat.protohackers.com:16963").await?;

    let mut framed = Framed::new(socket, LinesCodec::new());

    let mut up_framed = Framed::new(up_socket, LinesCodec::new());

    loop {
        select! {
            line = framed.next() =>  {
                match line {
                    Some(line) => {
                        match line {
                            Ok(message) => {
                                let message = replace_boguscoin(message);
                                #[cfg(debug_assertions)]
                                println!("{addr} --> {message}");
                                up_framed.send(message).await?;
                            }
                            Err(e) => {
                                return Err(e.into());
                            }
                        }
                 
                    }
                    None => {}
                }
            }
            line = up_framed.next() => {
                match line {
                    Some(line) => {
                        match line {
                            Ok(message) => {
                                let message = replace_boguscoin(message);
                                #[cfg(debug_assertions)]
                                println!("{addr} --> {message}");
                                framed.send(message).await?;
                            }
                            Err(e) => {
                                return Err(e.into());
                            }
                        }
                    }
                    None => {}
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
