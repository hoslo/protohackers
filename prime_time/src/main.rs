use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader, BufWriter, AsyncWriteExt};

fn is_prime(n: f32) -> bool {
    let n = n as u32;
    if n <= 1 {
        return false;
    }

    let sqrt_n = (n as f64).sqrt() as u32;

    for i in 2..=sqrt_n {
        if n % i == 0 {
            return false;
        }
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
    prime: bool,
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
                                    prime: is_prime(request.number),
                                    method: request.method,
                                };
                                if res.method != "isPrime".to_string() {
                                    let res = Response{
                                        prime: false,
                                        method: "isPrime".to_string(),
                                    };
                                    let res = serde_json::to_string(&res).unwrap();
                                    writer.write_all(res.as_bytes()).await.unwrap();
                                    writer.write_all(b"\n").await.unwrap();
                                    writer.flush().await.unwrap();
                                } else {
                                    let res = serde_json::to_string(&res).unwrap();
                                    writer.write_all(res.as_bytes()).await.unwrap();
                                    writer.write_all(b"\n").await.unwrap();
                                    writer.flush().await.unwrap();
                                }
                            }
                            _ => {
                                let res = Response{
                                    prime: false,
                                    method: "isPrime".to_string(),
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
