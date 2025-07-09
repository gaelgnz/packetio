# packetio
![Crates.io Size](https://img.shields.io/crates/size/packetio)

A simple Rust crate for sending and receiving length-prefixed binary packets over any stream
that implements `Read` or `Write`. Uses [`bincode`](https://crates.io/crates/bincode) for serialization.

It lets you send structs and rust types over the network!

---

## Features

- Sends and receives length-prefixed packets to keep boundaries clear
- Generic over any stream implementing `Read` or `Write`)(`TcpStream`, `BufReader`, etc.)
- Minimal API with a handy `PacketSender` and `PacketReceiver` traits for easy use
- Uses `bincode` for compact, efficient binary encoding
- Lightweight and just straight forward!

---

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
packetio = "0.1"
bincode = "2"
