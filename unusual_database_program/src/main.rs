use std::{net::SocketAddr, collections::HashMap};
use anyhow::{Result, Ok};
use tokio::net::UdpSocket;

struct Request {
    from: SocketAddr,
    key: Vec<u8>,
    value: Option<Vec<u8>>,
}

async fn read_request_from_udp(socket: &UdpSocket) -> Result<Option<Request>> {
    let mut buffer = [0; 1024];
    let (size, from) = socket.recv_from(&mut buffer).await?;
    let message = buffer[0..size].to_owned();

    let mut iter = message.splitn(2, |x| *x == b'=');

    let Some(key) = iter.next() else {
        return Ok(None);
    };
    let value = iter.next();
    
    Ok(Some(Request {
        from,
        key: key.to_owned(),
        value: value.map(ToOwned::to_owned),
    }))
}

async fn write_request_to_udp(socket: &UdpSocket, from: SocketAddr, key: &[u8], value: &[u8]) -> Result<()> {
    let message = format!("{}={}", String::from_utf8(key.to_owned())?, String::from_utf8(value.to_owned())?);
    socket.send_to(message.as_bytes(), from).await?;
    Ok(())
}

const VERSION_KEY: &[u8] = b"version";

const VERSION_VALUE: &[u8] =
    concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION")).as_bytes();

#[tokio::main]
async fn main() -> Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:8000").await?;
    let mut state = HashMap::new();
    loop {
        let Some(Request { from, key, value }) = read_request_from_udp(&socket).await? else {
            continue;
        };
        if let Some(v) = value.clone() {
            println!("{}: {:?} {:?}", from, String::from_utf8(key.clone()), String::from_utf8(v));
        } else {
            println!("{}: {:?}", from, String::from_utf8(key.clone()));
        }
        match value {
            Some(_) if key == VERSION_KEY => {
                continue;
            }
            Some(value) => {
                state.insert(key, value);
            }
            None if key == VERSION_KEY => {
                write_request_to_udp(&socket, from, &key, VERSION_VALUE).await?;
            }
            None => {
                let Some(value) = state.get(&key) else { continue };
                write_request_to_udp(&socket, from, &key, value).await?;
            }
        }
        
    }
}
