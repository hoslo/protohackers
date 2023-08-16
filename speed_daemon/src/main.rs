use codec::ClientToServerCodec;
use futures::StreamExt;
use tokio::net::TcpListener;
use tokio_util::codec::FramedRead;

mod codec;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:9999").await.unwrap();
    loop {

        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let mut framed = FramedRead::new(stream, ClientToServerCodec);
            loop {
                let msg = framed.next().await;
                if let Some(m) = msg {
                    match m {
                        Ok(msg) => {
                            println!("msg: {:?}", msg);
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
