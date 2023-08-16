use std::net::SocketAddr;
pub mod semicolon_codec;
pub mod strict_lines_codec;
use anyhow::Result;
use bytes::BytesMut;
use fancy_regex::Regex;
use futures::{SinkExt, StreamExt};
use lazy_static::lazy_static;
use semicolon_codec::SemicolonCodec;
use strict_lines_codec::StrictLinesCodec;
use tokio::{net::{TcpListener, TcpSocket, TcpStream}, io::AsyncWriteExt};
use tokio_util::codec::{Framed, FramedRead, FramedWrite, LinesCodec, LengthDelimitedCodec};

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
    let mut client_read = FramedRead::new(client_read, StrictLinesCodec::new());
    let mut client_write = FramedWrite::new(client_write, StrictLinesCodec::new());

    let server_stream = TcpStream::connect("chat.protohackers.com:16963").await?;
    let (server_read, server_write) = server_stream.into_split();
    let mut server_read = FramedRead::new(server_read, StrictLinesCodec::new());
    let mut server_write = FramedWrite::new(server_write, StrictLinesCodec::new());

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

// #[tokio::main]
// async fn main() -> Result<()> {
//     let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
//     loop {
//         let socket = listener.accept().await?.0;
//         socket.set_nodelay(true)?;
//         let addr = socket.peer_addr()?;
//         tokio::spawn(async move {
//             if let Err(e) = handle_client(socket, addr).await {
//                 println!("an error occured; error = {:?}", e);
//             }
//         });
//     }
// }
#[tokio::main]
async fn main() {
    tokio::spawn(
        async move {
            let listener = TcpListener::bind("0.0.0.0:9999").await.unwrap();
            println!("222222");
            loop {
                println!("3333333");
                let (stream, _) = listener.accept().await.unwrap();
                tokio::spawn(async move {
                    let mut framed = Framed::new(stream, SemicolonCodec::new());
                    loop {
                        let msg = framed.next().await;
                        if let Some(m) = msg {
                            match m {
                                Ok(msg) => {
                                    println!("msg: {}", msg);
                                }
                                Err(e) => {
                                    println!("err: {:?}", e);
                                }
                            }
                        }
                    }
                });
            }
        }
    );
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let a = &b"3 3 3;"[..].iter().position(|b| *b == b';');
    println!("{:?}", a);
    let mut c = BytesMut::from(  &b"3 3 3;"[..]);
    println!("{:?}",  c.split_to(a.unwrap() + 1));
    let addr = "0.0.0.0:9999".parse().unwrap();
    let socket = TcpSocket::new_v4().unwrap();
    let mut s = TcpSocket::connect(socket, addr).await.unwrap();
    println!("{:?}",String::from_utf8(b";,".to_vec()));
    // let a = AnyDelimiterCodec::new(b";".to_vec(), b";,".to_vec());
    let mut framed = Framed::new(s, LinesCodec::new());
    println!("connected");
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        framed.send("I 1 1".to_string()).await.unwrap();

        framed.send("222".to_string()).await.unwrap();
    }
}
