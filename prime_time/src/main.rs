use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader, BufWriter, AsyncWriteExt};

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

#[derive(Serialize, Deserialize, Debug)]
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
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    loop {
        let mut socket = listener.accept().await?.0;
        tokio::spawn(async move {
            let (reader, writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut writer = BufWriter::new(writer);

            loop {
                let mut buffer = String::new();
                match reader.read_line(&mut buffer).await {
                    Ok(0) => return,
                    Ok(_) => {
                        let r: Result<Request, _> = serde_json::from_str(&buffer);
                        println!("{:?}", r);
                        match r {
                            Ok(request) => {
                                let res = Response{
                                    is_prime: is_prime(request.number),
                                    method: request.method,
                                };
                                let res = serde_json::to_string(&res).unwrap();
                                writer.write_all(res.as_bytes()).await.unwrap();
                                writer.write_all(b"\n").await.unwrap();
                                writer.flush().await.unwrap();
                            }
                            _ => {
                                let res = Response{
                                    is_prime: false,
                                    method: "is_prime".to_string(),
                                };
                                let res = serde_json::to_string(&res).unwrap();
                                writer.write_all(res.as_bytes()).await.unwrap();
                                writer.write_all(b"\n").await.unwrap();
                                writer.flush().await.unwrap();
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return;
                    }
                }
            }
        });
    }
}
