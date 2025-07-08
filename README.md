# packetio

A simple Rust crate for sending and receiving length-prefixed binary packets over any stream
that implements `Read + Write`. Uses [`bincode`](https://crates.io/crates/bincode) for serialization.

It lets you send structs and rust types over the network!

---

## Features

- Sends and receives length-prefixed packets to keep boundaries clear
- Generic over any stream implementing `Read + Write` (`TcpStream`, `BufReader`, etc.)
- Minimal API with a handy `PacketIO` trait for easy use
- Uses `bincode` for compact, efficient binary encoding

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
packetio = "0.1"
bincode = "2"
