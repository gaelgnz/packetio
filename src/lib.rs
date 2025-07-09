//! # packetio
//!
//! A simple, efficient Rust crate for sending and receiving length-prefixed binary packets
//! over any `Read + Write` stream using `bincode` serialization.
//!
//! ## Example
//!
//! ```no_run
//! writer.send_packet(&test_struct).unwrap();

//! let mut size_buf = [0u8; 4];
//! reader.read_exact(&mut size_buf).unwrap();
//! let size = parsing::parse_length(size_buf);

//! let mut packet = vec![0u8; size];
//! reader.read_exact(&mut packet).unwrap();

//! let result: TestStruct = parsing::parse_packet(packet).unwrap(); // There you go!

//! ```
//!

use std::{
    error::Error,
    io::{Read, Write},
};

use bincode::{Decode, Encode};

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

/// NOTE! Packets send first its length, then the packet itself. So make sure you aren't parsing the length and the actual packet in bytes.
/// Example:
/// ```no_run
/// let stream = TcpStream::connect("127.0.0.1:8080")?;
///
/// let mut size_buf = [0u8; 4];
/// stream.read_exact(&mut size_buf)?; // First read the length (u32)
///
/// let size = packetio::parsing::parse_length(size_buf); // Parse the length of the packet
///
/// let mut buf = vec![0u8; size];
/// stream.read_exact(&mut buf)?; // Then read exactly the considered length
///
/// packetio::parse_packet(buffer)?;
/// assert_eq!(parsed_packet, vec![0x01, 0x02, 0x03]);
/// ```
///
/// This module provides more control over the parsing process. Which might be useful if you don't want recv_packet() and parse_packet() doing exactly what they do.
mod parsing {
    use super::*;

    #[allow(dead_code)]
    pub fn parse_packet<T: Decode<()>>(packet: Vec<u8>) -> Result<T, Box<dyn Error>> {
        let (decoded, _) = bincode::decode_from_slice(&packet, bincode::config::standard())?;
        Ok(decoded)
    }
    #[allow(dead_code)]
    pub fn parse_length(length: [u8; 4]) -> usize {
        let size = u32::from_be_bytes(length) as usize;
        size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use os_pipe::pipe;
    #[test]
    fn test_manual_parse_packet() {
        use crate::{PacketSender, parsing};

        let (mut reader, mut writer) = pipe().unwrap();

        #[derive(Encode, Decode, Debug, PartialEq)]
        struct TestStruct {
            field1: u8,
            field2: u16,
        }
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

        let result: TestStruct = parsing::parse_packet(packet).unwrap(); // There you go!

        assert_eq!(test_struct, result);
    }
}
