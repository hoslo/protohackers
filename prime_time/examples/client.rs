use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader, BufWriter, AsyncWriteExt};
use tokio::net::{TcpSocket};

fn is_prime(i: f32) -> bool {
    let mut j = 2.0;
    while j < i {
        if i % j == 0.0 {
            return false;
        }
        j += 1.0;
    }
    true
}

#[derive(Serialize, Deserialize)]
struct Request {
    number: f32,
    method: String,
}

#[derive(Serialize, Deserialize)]
struct Response {
    is_prime: bool,
    method: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:8000".parse().unwrap();
    let socket = TcpSocket::new_v4().unwrap();
    let mut listener = socket.connect(addr).await.unwrap();
    let (reader, writer) = listener.split();
    let res = Request{
        number: 9.0,
        method: "is_prime".to_string(),
    };
    let res = serde_json::to_string(&res).unwrap();
    let mut writer = BufWriter::new(writer);
    writer.write_all(res.as_bytes()).await.unwrap();
    writer.write_all(b"\n").await.unwrap();
    writer.flush().await.unwrap();

    let mut reader = BufReader::new(reader);
    let mut buffer = String::new();
    reader.read_line(&mut buffer).await.unwrap();
    let r: Result<Response, _> = serde_json::from_str(&buffer);
    match r {
        Ok(response) => {
            println!("{}: {}", response.method, response.is_prime);
        }
        _ => {
            println!("Error");
        }
    }

    Ok(())
}