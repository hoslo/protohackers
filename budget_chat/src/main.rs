use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    sync::broadcast,
};

#[derive(Clone, Debug)]
struct Message {
    username: String,
    message: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    let (tx, _) = broadcast::channel(32);
    let users = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let mut socket = listener.accept().await?.0;
        let tx = tx.clone();
        let mut rx = tx.subscribe();
        let users = users.clone();
        tokio::spawn(async move {
            let (reader, writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut writer = BufWriter::new(writer);

            println!("New client connected");
            let welcome = "Welcome to budgetchat! What shall I call you?\n".to_string();
            writer.write_all(welcome.as_bytes()).await.unwrap();
            writer.flush().await.unwrap();

            let mut username = String::new();
            reader.read_line(&mut username).await.unwrap();
            let username = username.trim().to_string();
            if username.len() == 0 {
                return;
            }
            println!("{} has joined the room", username);
            if users.lock().unwrap().contains_key(&username) {
                let err_msg = format!("* The username {} is already taken\n", username);
                writer.write_all(err_msg.as_bytes()).await.unwrap();
                writer.flush().await.unwrap();
                return;
            }
            users.lock().unwrap().insert(username.clone(), true);
            let other_users = users
                .lock()
                .unwrap()
                .iter()
                .filter(|u| u.0 != &username)
                .map(|u| u.0.clone())
                .collect::<Vec<String>>();
            let list_user_msg = format!("* The room contains: {}\n", other_users.join(", "));
            println!("{}", list_user_msg);
            writer.write_all(list_user_msg.as_bytes()).await.unwrap();
            writer.flush().await.unwrap();

            let join_msg = format!("* {} has entered the room\n", username);
            let msg = Message {
                username: username.clone(),
                message: join_msg,
            };
            tx.send(msg).unwrap();

            let mut line = String::new();
            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            let leave_msg = format!("{} bob has left the room\n", username);
                            writer.write_all(leave_msg.as_bytes()).await.unwrap();
                            writer.flush().await.unwrap();
                            users.lock().unwrap().remove(&username);
                            return
                        }
                        let msg = Message {
                            username: username.clone(),
                            message: line.clone(),
                        };
                        tx.send(msg).unwrap();
                        line.clear();
                    }
                    result = rx.recv() => match result {
                        Ok(msg) => {
                            if msg.username == username {
                                continue;
                            }
                            let msg = format!("[{}]: {}", msg.username, msg.message);
                            writer.write_all(msg.as_bytes()).await.unwrap();
                            writer.flush().await.unwrap();
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                            return
                        }
                    }
                };
            }
        });
    }
}


#[cfg(test)]
mod test {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};


    #[tokio::test]
    async fn test() {
        // join the chat
        let mut socket = tokio::net::TcpStream::connect("127.0.0.1:8000").await.unwrap();
        let (reader, writer) = socket.split();
        let mut reader = tokio::io::BufReader::new(reader);
        let mut writer = tokio::io::BufWriter::new(writer);
        // read welcome message
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        assert_eq!(line, "Welcome to budgetchat! What shall I call you?\n");
        println!("line: {}", line);
        
        // send username
        writer.write_all("bob\n".as_bytes()).await.unwrap();
        writer.flush().await.unwrap();

        // read list of users
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        assert_eq!(line, "* The room contains: \n");

        // send message
        writer.write_all("hello\n".as_bytes()).await.unwrap();
        writer.flush().await.unwrap();

    }
}