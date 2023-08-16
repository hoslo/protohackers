

#[derive(Debug)]
pub enum ClientToServerMessage {
    // 0x20
    Plate { plate: String, timestamp: u32 },
    // 0x40
    WantHeartbeat { interval: u32 },
    // 0x80
    IAmCamera { road: u16, mile: u16, limit: u16 },
    // 0x81
    IAmDispatcher { roads: Vec<u16> },
}

#[derive(Debug, Clone)]
pub enum ServerToClientMessage {
    // 0x10
    Error(String),
    // 0x21
    Ticket {
        plate: String,
        road: u16,
        mile1: u16,
        timestamp1: u32,
        mile2: u16,
        timestamp2: u32,
        speed: u16,
    },
    // 0x41
    Heartbeat,
}

