use std::{sync::Arc, collections::HashMap, clone};
use anyhow::{Result, bail, Ok};

use tokio::sync::mpsc::{Sender, Receiver};

#[derive(Debug, Default)]
pub struct State {
    clients: HashMap<String, Sender<Event>>
}

impl State {
    pub fn add_client(&mut self, name: ClientName) -> Result<Receiver<Event>> {
        if self.clients.contains_key(&name) {
            bail!("* The username {} is already taken\n", name);
        }
        let event = Event::NewUser(name.clone());
        for sender in self.clients.values() {
            let _ = sender.send(event.clone());
        }

        let (sender, receiver) = tokio::sync::mpsc::channel(32);
        self.clients.insert(name, sender);
        Ok(receiver)
    }

    pub fn get_present_names(&self) -> Vec<String> {
        self.clients.keys().cloned().collect()
    }

    pub fn boardcast_message(&self, name: ClientName, message: Message) {
        let event = Event::NewMessage(name.clone(), message);
        for (client_name, sender) in self.clients.clone() {
            if client_name != name {
                let _ = sender.send(event.clone());
            }
        }
    }

    pub fn remove_client(&mut self, name: ClientName) {
        self.clients.remove(&name);
        let event = Event::UserLeft(name.clone());
        for (client_name, sender) in self.clients.clone() {
            if client_name != name {
                let _ = sender.send(event.clone());
            }
        }
    }
}

type ClientName = String;
type Message = String;

#[derive(Clone)]
pub enum Event {
    NewUser(ClientName),
    NewMessage(ClientName, Message),
    UserLeft(ClientName),
}