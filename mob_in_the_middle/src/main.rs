use anyhow::Result;
use futures::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use fancy_regex::Regex;
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

async fn handle_client(socket: tokio::net::TcpStream) -> Result<()> {
    let up_socket = TcpStream::connect("chat.protohackers.com:16963").await?;

    let mut framed = Framed::new(socket, LinesCodec::new());

    let mut up_framed = Framed::new(up_socket, LinesCodec::new());

    loop {
        select! {
            line = framed.next() =>  {
                let message = line.ok_or(anyhow::Error::msg("Got EOF"))??;
                let message = replace_boguscoin(message);
                println!("Got down message: {}", message);
                up_framed.send(message).await?;
            }
            line = up_framed.next() => {
                let message = line.ok_or(anyhow::Error::msg("Got EOF"))??;
                let message = replace_boguscoin(message);
                println!("Got up message: {}", message);
                framed.send(message).await?;
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

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                println!("an error occured; error = {:?}", e);
            }
        });
    }
}
