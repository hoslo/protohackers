mod state;

use std::{net::SocketAddr, sync::Arc};

use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use state::State;
use tokio::sync::Mutex;
use tokio::{net::TcpStream, select};
use tokio_util::codec::{Framed, LinesCodec};

use crate::state::Event;

async fn handle_client(
    stream: TcpStream,
    addr: SocketAddr,
    state: Arc<Mutex<State>>,
) -> Result<()> {
    let mut framed = Framed::new(stream, LinesCodec::new());

    framed
        .send("Welcome to budgetchat! What shall I call you?")
        .await?;

    let name = framed.next().await.ok_or(anyhow::Error::msg("Got EOF"))??;

    if name.is_empty() {
        framed.send(format!("Your name cannot be empty")).await?;

        return Err(anyhow::Error::msg("Illegal empty name"));
    } else if !name.chars().all(|char| {
        ('a'..='z').contains(&char) || ('A'..='Z').contains(&char) || ('0'..='9').contains(&char)
    }) {
        framed
            .send(format!(
                "Your name can only have alphanumeric characters (uppercase, lowercase, digits)"
            ))
            .await?;

        return Err(anyhow::Error::msg("Illegal characters in name"));
    }

    let online = {
        let state = state.lock().await;
        state.get_present_names().join(", ")
    };

    framed
        .send(format!("* The room contains: {}", online))
        .await?;

    let result = handle_joined(framed, addr, name.clone(), state.clone()).await;

    {
        let mut state = state.lock().await;
        state.remove_client(name);
    }

    result
}

async fn handle_joined(
    mut framed: Framed<TcpStream, LinesCodec>,
    addr: SocketAddr,
    name: String,
    state: Arc<Mutex<State>>,
) -> Result<()> {
    let mut receiver = state.lock().await.add_client(name.clone())?;

    loop {
        select! {
            item = framed.next() => {
                #[cfg(debug_assertions)]
                println!("{addr} --> {item:?}");

                let message = item.ok_or(anyhow::Error::msg("Got EOF"))??;
                let message = message;

                {
                    let state = state.lock().await;
                    state.boardcast_message(name.clone(), message);
                }
            }
            event = receiver.recv() => {
                println!("event: {:?}", event);
                let event = event.ok_or(anyhow::Error::msg("Somohow all senders dropped?"))?;

                match event {
                    Event::NewUser(name) => {
                        let join_msg = format!("* {} has entered the room", name);
                        println!("{} <-- {}", addr, join_msg);
                        framed.send(join_msg).await?;
                    }
                    Event::NewMessage(name, message) => {
                        let msg = format!("[{}]: {}", name, message);
                        println!("{} <-- {}", addr, msg);
                        framed.send(msg).await?;
                    }
                    Event::UserLeft(name) => {
                        let left_msg = format!("* {} has left the room", name);
                        println!("{} <-- {}", addr, left_msg);
                        framed.send(left_msg).await?;
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    let state = Arc::new(Mutex::new(State::default()));
    loop {
        let socket = listener.accept().await?.0;
        let addr = socket.peer_addr()?;
        socket.set_nodelay(true)?;
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, addr, state).await {
                println!("an error occured; error = {:?}", e);
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
        let mut socket = tokio::net::TcpStream::connect("127.0.0.1:8000")
            .await
            .unwrap();
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

        {
            // join the chat
            let mut socket = tokio::net::TcpStream::connect("127.0.0.1:8000")
                .await
                .unwrap();
            let (reader, writer) = socket.split();
            let mut reader = tokio::io::BufReader::new(reader);
            let mut writer = tokio::io::BufWriter::new(writer);
            // read welcome message
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            assert_eq!(line, "Welcome to budgetchat! What shall I call you?\n");
            println!("line: {}", line);

            // send username
            writer.write_all("alice\n".as_bytes()).await.unwrap();
            writer.flush().await.unwrap();

            // read list of users
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            assert_eq!(line, "* The room contains: bob\n");

            // send message
            writer.write_all("hello\n".as_bytes()).await.unwrap();
            writer.flush().await.unwrap();
        }

        // read message
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        println!("line: {}", line);
        assert_eq!(line, "* alice has entered the room\n");

        // read message
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        println!("line: {}", line);
        assert_eq!(line, "[alice] hello\n");
    }
}
