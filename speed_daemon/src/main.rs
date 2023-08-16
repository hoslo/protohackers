use codec::{ClientToServerCodec, ClientToServerMessage, ServerToClientMessage};
use futures::{SinkExt, StreamExt};
use tokio::{io::AsyncReadExt, net::TcpListener, sync::mpsc::Sender};
use tokio_util::codec::{FramedRead, FramedWrite};

mod codec;

async fn heartbeat(interval: u32, sender: Sender<ServerToClientMessage>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(
        interval as u64 * 100,
    ));
    loop {
        interval.tick().await;
        sender.send(ServerToClientMessage::Heartbeat).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let (reader, writer) = stream.into_split();
            let mut framed = FramedRead::new(reader, ClientToServerCodec);
            let (mut sender, mut receiver) = tokio::sync::mpsc::channel(100);
            let mut framed_write = FramedWrite::new(writer, codec::ServerToClientCodec);
            tokio::spawn(async move {
                while let Some(msg) = receiver.recv().await {
                    match msg {
                        ServerToClientMessage::Heartbeat => {
                            println!("send heartbeat");
                            framed_write
                                .send(ServerToClientMessage::Heartbeat)
                                .await
                                .unwrap();
                        }
                        _ => {}
                    }
                }
            });
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                let msg = framed.next().await;
                if let Some(m) = msg {
                    match m {
                        Ok(msg) => match msg {
                            ClientToServerMessage::WantHeartbeat { interval } => {
                                if interval != 0 {
                                    heartbeat(interval, sender.clone()).await;
                                }
                            }
                            ClientToServerMessage::IAmDispatcher { roads } => {
                                println!("IAmDispatcher: {:?}", roads);
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
