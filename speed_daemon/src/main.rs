use codec::ClientToServerCodec;
use futures::StreamExt;
use tokio::net::TcpListener;
use tokio_util::codec::FramedRead;

mod codec;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let (reader, _writer) = stream.split();
            let mut framed = FramedRead::new(reader, ClientToServerCodec);
            loop {
                let msg = framed.next().await;
                if let Some(m) = msg {
                    match m {
                        Ok(msg) => match msg {
                            codec::ClientToServerMessage::WantHeartbeat { interval } => {
                                if interval != 0 {
                                    println!("WantHeartbeat: {}", interval);
                                }
                            }
                            _ => {}
                        },
                        Err(e) => {
                            println!("err: {:?}", e);
                        }
                    }
                }
            }
        });
    }
}
