//! # packetio
//!
//! A simple, efficient Rust crate for sending and receiving length-prefixed binary packets
//! over any `Read + Write` stream using `bincode` serialization.
//!
//! ## Example
//!
//! ```no_run
//! use std::net::TcpStream;
//! use packetio::PacketIO;
//!
//! let mut stream = TcpStream::connect("127.0.0.1:9000").unwrap();
//! stream.send_packet(MyPacket { ... }).unwrap(); // Mypacket or basically any other struct or rust type as long as it implements Encode and Decode traits (bincode)
//! let response: MyResponse = stream.recv_packet().unwrap();
//! ```
//!

use std::{
    error::Error,
    io::{Read, Write},
};

use bincode::{Decode, Encode};

pub trait PacketIO {
    fn send_packet<T: Encode>(&mut self, packet: T) -> Result<(), Box<dyn Error>>;
    fn recv_packet<T: Decode<()>>(&mut self) -> Result<T, Box<dyn Error>>;
}

impl<T: Read + Write> PacketIO for T {
    fn send_packet<U: Encode>(&mut self, packet: U) -> Result<(), Box<dyn Error>> {
        send_packet(packet, self)
    }

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
