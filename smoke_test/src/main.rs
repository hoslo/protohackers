use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()>  {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    loop {
        let mut socket = listener.accept().await?.0;
        tokio::spawn(async move {
            let (mut reader, mut writer) = socket.split();
            tokio::io::copy(&mut reader, &mut writer).await.unwrap();
        });
    }
}
