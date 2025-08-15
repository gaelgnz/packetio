//! # packetio
//!
//! A simple, efficient Rust crate for sending and receiving length-prefixed binary packets
//! over any `Read + Write` or `AsyncRead + AsyncWrite` stream using `bincode` serialization.
//!
//! ## Example (Sync)
//! ```no_run
//! writer.send_packet(&test_struct).unwrap();
//!
//! let mut size_buf = [0u8; 4];
//! reader.read_exact(&mut size_buf).unwrap();
//! let size = parsing::parse_length(size_buf);
//!
//! let mut packet = vec![0u8; size];
//! reader.read_exact(&mut packet).unwrap();
//!
//! let result: TestStruct = parsing::parse_packet(packet).unwrap();
//! ```
//!
//! ## Example (Async)
//! ```no_run
//! use tokio::net::TcpStream;
//! use packetio::AsyncPacketSender;
//!
//! let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
//! stream.send_packet_async(&my_packet).await?;
//! ```

use std::{
    error::Error,
    io::{Read, Write},
};

use bincode::{Decode, Encode};

//
// === SYNC TRAITS ===
//
pub trait PacketSender {
    fn send_packet<T: Encode>(&mut self, packet: T) -> Result<(), Box<dyn Error>>;
}

pub trait PacketReceiver {
    fn recv_packet<T: Decode<()>>(&mut self) -> Result<T, Box<dyn Error>>;
}

impl<T: Write> PacketSender for T {
    fn send_packet<U: Encode>(&mut self, packet: U) -> Result<(), Box<dyn Error>> {
        send_packet(packet, self)
    }
}

impl<T: Read> PacketReceiver for T {
    fn recv_packet<U: Decode<()>>(&mut self) -> Result<U, Box<dyn Error>> {
        recv_packet(self)
    }
}

pub fn send_packet<T: Encode, W: Write>(packet: T, writer: &mut W) -> Result<(), Box<dyn Error>> {
    let encoded_packet = bincode::encode_to_vec(&packet, bincode::config::standard())?;
    writer.write_all(&(encoded_packet.len() as u32).to_be_bytes())?;
    writer.write_all(&encoded_packet)?;
    Ok(())
}

pub fn recv_packet<T: Decode<()>, R: Read>(reader: &mut R) -> Result<T, Box<dyn Error>> {
    let mut len_bytes = [0; 4];
    reader.read_exact(&mut len_bytes)?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    let mut packet = vec![0; len];
    reader.read_exact(&mut packet)?;

    let (decoded, _) = bincode::decode_from_slice(&packet, bincode::config::standard())?;
    Ok(decoded)
}

//
// === ASYNC TRAITS ===
//
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[async_trait::async_trait]
pub trait AsyncPacketSender {
    async fn send_packet_async<T: Encode + Send>(&mut self, packet: T) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait::async_trait]
pub trait AsyncPacketReceiver {
    async fn recv_packet_async<T: Decode<()> + Send>(&mut self) -> Result<T, Box<dyn Error + Send + Sync>>;
}

#[async_trait::async_trait]
impl<T: AsyncWrite + Unpin + Send> AsyncPacketSender for T {
    async fn send_packet_async<U: Encode + Send>(&mut self, packet: U) -> Result<(), Box<dyn Error + Send + Sync>> {
        send_packet_async(packet, self).await
    }
}

#[async_trait::async_trait]
impl<T: AsyncRead + Unpin + Send> AsyncPacketReceiver for T {
    async fn recv_packet_async<U: Decode<()> + Send>(&mut self) -> Result<U, Box<dyn Error + Send + Sync>> {
        recv_packet_async(self).await
    }
}

pub async fn send_packet_async<T: Encode, W: AsyncWrite + Unpin>(
    packet: T,
    writer: &mut W
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let encoded_packet = bincode::encode_to_vec(&packet, bincode::config::standard())?;
    writer.write_all(&(encoded_packet.len() as u32).to_be_bytes()).await?;
    writer.write_all(&encoded_packet).await?;
    Ok(())
}

pub async fn recv_packet_async<T: Decode<()>, R: AsyncRead + Unpin>(
    reader: &mut R
) -> Result<T, Box<dyn Error + Send + Sync>> {
    let mut len_bytes = [0; 4];
    reader.read_exact(&mut len_bytes).await?;
    let len = u32::from_be_bytes(len_bytes) as usize;

    let mut packet = vec![0; len];
    reader.read_exact(&mut packet).await?;

    let (decoded, _) = bincode::decode_from_slice(&packet, bincode::config::standard())?;
    Ok(decoded)
}

//
// === PARSING UTILITIES ===
//
pub mod parsing {
    use super::*;

    #[allow(dead_code)]
    pub fn parse_packet<T: Decode<()>>(packet: Vec<u8>) -> Result<T, Box<dyn Error>> {
        let (decoded, _) = bincode::decode_from_slice(&packet, bincode::config::standard())?;
        Ok(decoded)
    }

    #[allow(dead_code)]
    pub fn parse_length(length: [u8; 4]) -> usize {
        u32::from_be_bytes(length) as usize
    }
}

//
// === TESTS ===
//
#[cfg(test)]
mod tests {
    use super::*;
    use os_pipe::pipe;

    #[derive(Encode, Decode, Debug, PartialEq)]
    struct TestStruct {
        field1: u8,
        field2: u16,
    }

    #[test]
    fn test_manual_parse_packet() {
        use crate::{PacketSender, parsing};

        let (mut reader, mut writer) = pipe().unwrap();

        let test_struct = TestStruct {
            field1: 1,
            field2: 2,
        };

        writer.send_packet(&test_struct).unwrap();

        let mut size_buf = [0u8; 4];
        reader.read_exact(&mut size_buf).unwrap();
        let size = parsing::parse_length(size_buf);

        let mut packet = vec![0u8; size];
        reader.read_exact(&mut packet).unwrap();

        let result: TestStruct = parsing::parse_packet(packet).unwrap();

        assert_eq!(test_struct, result);
    }
}
