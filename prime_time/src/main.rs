use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

fn is_prime(n: f64) -> bool {
    let n = n as u64;
    if n <= 1 {
        return false;
    }

    if n <= 3 {
        return true;
    }

    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }

    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }

    true
}

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    number: f64,
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
                                let res = Response {
                                    prime: is_prime(request.number),
                                    method: request.method,
                                };
                                if res.method != "isPrime".to_string() {
                                    writer.write_all(b"error").await.unwrap();
                                    writer.flush().await.unwrap();
                                    return;
                                } else {
                                    let res = serde_json::to_string(&res).unwrap();
                                    writer.write_all(res.as_bytes()).await.unwrap();
                                    writer.write_all(b"\n").await.unwrap();
                                    writer.flush().await.unwrap();
                                }
                            }
                            _ => {
                                writer.write_all(b"error").await.unwrap();
                                writer.flush().await.unwrap();
                                return;
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
