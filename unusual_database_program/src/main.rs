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

async fn write_request_to_udp(socket: &UdpSocket, request: &Request) -> Result<()> {
    let message = match &request.value {
        Some(value) => format!("{}={}", String::from_utf8_lossy(&request.key), String::from_utf8_lossy(&value)),
        None => String::from_utf8_lossy(&request.key).to_string(),
    };
    socket.send_to(message.as_bytes(), request.from).await?;
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
        println!("{}: {:?} {:?}", from, String::from_utf8(key.clone()), value.as_deref());
        match value {
            Some(_) if key == VERSION_KEY => {
                continue;
            }
            Some(value) => {
                state.insert(key, value);
            }
            None if key == VERSION_KEY => {
                let request = Request {
                    from,
                    key: VERSION_KEY.to_owned(),
                    value: Some(VERSION_VALUE.to_owned()),
                };
                write_request_to_udp(&socket, &request).await?;
            }
            None => {
                let request = Request {
                    from,
                    key: VERSION_KEY.to_owned(),
                    value: state.get(&key).cloned(),
                };
                write_request_to_udp(&socket, &request).await?;
            }
        }
        
    }
}
