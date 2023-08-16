use std::{cmp, io, str};

use bytes::{Buf, BytesMut, BufMut};
use futures::{StreamExt, SinkExt};
use tokio::net::{TcpListener, TcpSocket};
use tokio_util::codec::{Decoder, Encoder, Framed};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SemicolonCodec {
    // Stored index of the next index to examine for a `\n` character.
    // This is used to optimize searching.
    // For example, if `decode` was called with `abc`, it would hold `3`,
    // because that is the next index to examine.
    // The next time `decode` is called with `abcde\n`, the method will
    // only look at `de\n` before returning.
    next_index: usize,

    /// The maximum length for a given line. If `usize::MAX`, lines will be
    /// read until a `\n` character is reached.
    max_length: usize,

    /// Are we currently discarding the remainder of a line which was over
    /// the length limit?
    is_discarding: bool,
}

#[derive(Debug)]
pub enum SemicolonCodecCodecError {
    /// The maximum line length was exceeded.
    MaxLineLengthExceeded,
    /// An IO error occurred.
    Io(io::Error),
}

impl SemicolonCodec {
    pub fn new() -> Self {
        SemicolonCodec {
            next_index: 0,
            max_length: usize::MAX,
            is_discarding: false,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        SemicolonCodec {
            next_index: 0,
            max_length: capacity,
            is_discarding: false,
        }
    }
}

impl From<io::Error> for SemicolonCodecCodecError {
    fn from(e: io::Error) -> SemicolonCodecCodecError {
        SemicolonCodecCodecError::Io(e)
    }
}

fn utf8(buf: &[u8]) -> Result<&str, io::Error> {
    str::from_utf8(buf)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Unable to decode input as UTF8"))
}

impl Decoder for SemicolonCodec {
    type Item = String;

    type Error = SemicolonCodecCodecError;

    fn decode(&mut self, buf: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            let read_to = cmp::min(self.max_length.saturating_add(1), buf.len());
            println!("read_to: {}", String::from_utf8(buf.clone().to_vec()).unwrap());
            let newline_offset = buf[self.next_index..read_to]
                .iter()
                .position(|b| *b == b';');

            match (self.is_discarding, newline_offset) {
                (true, Some(offset)) => {
                    // If we found a newline, discard up to that offset and
                    // then stop discarding. On the next iteration, we'll try
                    // to read a line normally.
                    buf.advance(offset + self.next_index + 1);
                    self.is_discarding = false;
                    self.next_index = 0;
                }
                (true, None) => {
                    // Otherwise, we didn't find a newline, so we'll discard
                    // everything we read. On the next iteration, we'll continue
                    // discarding up to max_len bytes unless we find a newline.
                    buf.advance(read_to);
                    self.next_index = 0;
                    if buf.is_empty() {
                        return Ok(None);
                    }
                }
                (false, Some(offset)) => {
                    // Found a line!
                    let newline_index = offset + self.next_index;
                    self.next_index = 0;
                    let line = buf.split_to(newline_index + 1);
                    let line = &line[..line.len() - 1];
                    let line = utf8(line)?;
                    return Ok(Some(line.to_string()));
                }
                (false, None) if buf.len() > self.max_length => {
                    // Reached the maximum length without finding a
                    // newline, return an error and start discarding on the
                    // next call.
                    self.is_discarding = true;
                    return Err(SemicolonCodecCodecError::MaxLineLengthExceeded);
                }
                (false, None) => {
                    // We didn't find a line or reach the length limit, so the next
                    // call will resume searching at the current offset.
                    self.next_index = read_to;
                    return Ok(None);
                }
            }
        }
    }
}

impl<T> Encoder<T> for SemicolonCodec
where
    T: AsRef<str>,
{
    type Error = SemicolonCodecCodecError;

    fn encode(&mut self, line: T, buf: &mut BytesMut) -> Result<(), SemicolonCodecCodecError> {
        let line = line.as_ref();
        buf.reserve(line.len() + 1);
        buf.put(line.as_bytes());
        buf.put_u8(b';');
        Ok(())
    }
}


#[tokio::test]
async fn test() {
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let addr = "0.0.0.0:8000".parse().unwrap();
    let socket = TcpSocket::new_v4().unwrap();
    let s = TcpSocket::connect(socket, addr).await.unwrap();
    println!("{:?}", s);
    let mut framed = Framed::new(s, SemicolonCodec::new());
    println!("connected");
    framed.send("I 1 1".to_string()).await.unwrap();

    framed.send("222".to_string()).await.unwrap();
}
