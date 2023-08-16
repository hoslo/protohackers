use codec::ClientToServerCodec;
use futures::StreamExt;
use tokio::{net::TcpListener, io::AsyncReadExt};
use tokio_util::codec::FramedRead;

mod codec;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8000").await.unwrap();
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let (mut reader, _writer) = stream.split();
            // let mut framed = FramedRead::new(reader, ClientToServerCodec);
            loop {
                let mut buf = vec![0u8; 5];
                let n = reader.read_exact(&mut buf).await.unwrap();
                println!("read {:?} bytes", buf);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                // let msg = framed.next().await;
                // if let Some(m) = msg {
                //     match m {
                //         Ok(msg) => match msg {
                //             codec::ClientToServerMessage::WantHeartbeat { interval } => {
                //                 if interval != 0 {
                //                     println!("WantHeartbeat: {}", interval);
                //                 }
                //             }
                //             _ => {}
                //         },
                //         Err(e) => {
                //             println!("err: {:?}", e);
                //         }
                //     }
                // }
            }
        });
    }
}
