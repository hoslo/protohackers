use std::collections::HashMap;

use tokio::sync::oneshot::Sender;

use crate::codec::ServerToClientMessage;

struct Position {
    timestamp: u32,
    mile: u16,
}


#[derive(Debug, Default)]
pub struct State {
    pub cameras: HashMap<(String, u16), Position>,
    pub clients: HashMap<u16, Sender<ServerToClientMessage>>,
}