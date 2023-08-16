use std::io;

use byteorder::{BigEndian, ByteOrder};
use tokio_util::codec::{Decoder, Encoder};

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

#[derive(Debug)]
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

pub struct ClientToServerCodec;

impl Decoder for ClientToServerCodec {
    type Item = ClientToServerMessage;

    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let t = src[0];
        match t {
            // plate message
            0x20 => {
                let str_len = src[1] as usize;
                let plate = String::from_utf8(src[2..2 + str_len].to_vec())?;
                let timestamp = BigEndian::read_u32(&src[2 + str_len..6 + str_len]);
                return Ok(Some(ClientToServerMessage::Plate { plate, timestamp }));
            }
            // want heartbeat message
            0x40 => {
                println!("want heartbeat message: {:?}", src.clone());
                let interval = BigEndian::read_u32(&src[1..5]);
                return Ok(Some(ClientToServerMessage::WantHeartbeat { interval }));
            }
            // I am camera message
            0x80 => {
                let road = BigEndian::read_u16(&src[1..3]);
                let mile = BigEndian::read_u16(&src[3..5]);
                let limit = BigEndian::read_u16(&src[5..7]);
                return Ok(Some(ClientToServerMessage::IAmCamera { road, mile, limit }));
            }
            // I am dispatcher message
            0x81 => {
                let mut roads = Vec::new();
                let mut i = 1;
                while i < src.len() {
                    roads.push(BigEndian::read_u16(&src[i..i + 2]));
                    i += 2;
                }
                return Ok(Some(ClientToServerMessage::IAmDispatcher { roads }));
            }
            _ => {
                return Err(anyhow::anyhow!("unknown message type"));
            }
        }
    }
}

pub struct ServerToClientCodec;

impl Encoder<ServerToClientMessage> for ServerToClientCodec {
    type Error = anyhow::Error;

    fn encode(
        &mut self,
        item: ServerToClientMessage,
        dst: &mut bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        match item {
            ServerToClientMessage::Error(msg) => {
                dst.extend_from_slice(&[0x10]);
                dst.extend_from_slice(msg.as_bytes());
            }
            ServerToClientMessage::Ticket {
                plate,
                road,
                mile1,
                timestamp1,
                mile2,
                timestamp2,
                speed,
            } => {
                dst.extend_from_slice(&[0x21]);
                dst.extend_from_slice(plate.as_bytes());
                BigEndian::write_u16(dst, road);
                BigEndian::write_u16(dst, mile1);
                BigEndian::write_u32(dst, timestamp1);
                BigEndian::write_u16(dst, mile2);
                BigEndian::write_u32(dst, timestamp2);
                BigEndian::write_u16(dst, speed);
            }
            ServerToClientMessage::Heartbeat => {
                dst.extend_from_slice(&[0x41]);
            }
        }
        Ok(())
    }
}
