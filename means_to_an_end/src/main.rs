use anyhow::Result;
use byteorder::{BigEndian, ByteOrder};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

struct Price {
    timestamp: i32,
    price: i32,
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    loop {
        let mut socket = listener.accept().await?.0;
        tokio::spawn(async move {
            let mut s = vec![];
            let (reader, writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut writer = BufWriter::new(writer);

            loop {
                let mut buffer = [0; 9];
                match reader.read_exact(&mut buffer).await {
                    Ok(0) => return,
                    Ok(_) => {
                        if buffer[0] as char == 'I' {
                            let timestamp = BigEndian::read_i32(&buffer[1..5]);
                            let price = BigEndian::read_i32(&buffer[5..9]);
                            println!("I {} {}", timestamp, price);
                            s.push(Price { timestamp, price });
                        } else if buffer[0] as char == 'Q' {
                            let mintime = BigEndian::read_i32(&buffer[1..5]);
                            let maxtime = BigEndian::read_i32(&buffer[5..9]);
                            println!("Q {} {}", mintime, maxtime);
                            let filter_s: Vec<i32> = s
                                .iter()
                                .filter(|p| p.timestamp >= mintime && p.timestamp <= maxtime)
                                .map(|p| p.price)
                                .collect();
                            if filter_s.len() as i32 == 0 {
                                let response = (0 as i32).to_be_bytes();
                                writer.write_all(&response).await.unwrap();
                                writer.flush().await.unwrap();
                                continue;
                            }
                            let avg_price = filter_s.iter().sum::<i32>() / filter_s.len() as i32;
                            let response = avg_price.to_be_bytes();
                            writer.write_all(&response).await.unwrap();
                            writer.flush().await.unwrap();
                        }
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                        return
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod test {
    use byteorder::{BigEndian, ByteOrder};
    use tokio::{io::{BufWriter, BufReader, AsyncWriteExt, AsyncReadExt}, net::TcpSocket};

    #[tokio::test]
    async fn test() {
        let addr = "127.0.0.1:8000".parse().unwrap();
        let socket = TcpSocket::new_v4().unwrap();
        let mut listener = socket.connect(addr).await.unwrap();
        let (reader, writer) = listener.split();
        let mut writer = BufWriter::new(writer);
        let mut reader = BufReader::new(reader);

        // Insert
        let mut buffer = [0; 9];
        buffer[0] = 'I' as u8;
        BigEndian::write_i32(&mut buffer[1..5], 1);
        BigEndian::write_i32(&mut buffer[5..9], 100);
        writer.write_all(&buffer).await.unwrap();
        
        // continue insert
        let mut buffer2 = [0; 9];
        buffer2[0] = 'I' as u8;
        BigEndian::write_i32(&mut buffer2[1..5], 2);
        BigEndian::write_i32(&mut buffer2[5..9], 200);
        writer.write_all(&buffer2).await.unwrap();

        // Query
        let mut buffer3 = [0; 9];
        buffer3[0] = 'Q' as u8;
        BigEndian::write_i32(&mut buffer3[1..5], 1);
        BigEndian::write_i32(&mut buffer3[5..9], 2);
        writer.write_all(&buffer3).await.unwrap();
        writer.flush().await.unwrap();

        let mut buffer4 = [0; 4];
        reader.read_exact(&mut buffer4).await.unwrap();
        let avg_price = i32::from_be_bytes(buffer4);
        assert_eq!(avg_price, 150);

        // query empty
        let mut buffer5 = [0; 9];
        buffer5[0] = 'Q' as u8;
        BigEndian::write_i32(&mut buffer5[1..5], 3);
        BigEndian::write_i32(&mut buffer5[5..9], 4);
        writer.write_all(&buffer5).await.unwrap();
        writer.flush().await.unwrap();

        let mut buffer6 = [0; 4];
        reader.read_exact(&mut buffer6).await.unwrap();
        let avg_price = i32::from_be_bytes(buffer6);
        assert_eq!(avg_price, 0);
    }
}
