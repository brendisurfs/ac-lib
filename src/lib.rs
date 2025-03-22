#![allow(unused)]
//! This is a goofy little project to learn about displaying data from assetto corsa.
//! reference for data: https://docs.google.com/document/d/1KfkZiIluXZ6mMhLWfDX1qAGbvhGRC3ZUzjVIt5FQpp4/pub
//! also referrence: https://github.com/rickwest/ac-remote-telemetry-client/blob/master/src/parsers/RTCarInfoParser.js
//! notes for parsing:
//! int  are 32 bit little endian integers
//! float are 32 bit floating point numbers
//! bool are 8 bit boolean value

mod parser;
use std::{
    fs::File,
    io::{BufRead, BufReader, ErrorKind, Read},
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
    time::Duration,
};

use anyhow::bail;
use bytes::{BufMut, Bytes, BytesMut};
use parser::{Device, Handshake, MessageKind, Operation};
use tokio::{
    net::{ToSocketAddrs, UdpSocket},
    time::sleep,
};

const AC_VERSION: i32 = 1;

pub struct AcClient {
    socket: UdpSocket,
}

impl AcClient {
    pub async fn new<A>(remote_addr: A) -> anyhow::Result<Self>
    where
        A: ToSocketAddrs,
    {
        // NOTE: this needs to be chosen by the OS, or else it will never pick up.
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(remote_addr).await?;
        Ok(Self { socket })
    }
}

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     // build our socket.
//     let remote_addr = SocketAddr::from(([192, 168, 0, 135], 9996));
//     let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
//     socket.connect(remote_addr).await?;
//
//     // next send handshake
//     let handshake_msg = build_msg(Operation::Handshake, Device::IPhone);
//     socket.send(&handshake_msg).await?;
//
//     loop {
//         // guards our loop from just spinning hard i guess
//         socket.readable().await?;
//
//         // NOTE: The buffer we write to must be large enough, or else we may not get enough data.
//         let mut buf = vec![0u8; 1024];
//
//         match socket.try_recv(&mut buf) {
//             Ok(size) => {
//                 let event = MessageKind::from_bytes(size, &buf)?;
//                 println!("Event: {event:#?}");
//
//                 if let MessageKind::HandshakeResponse { .. } = event {
//                     // if we receive an Ok initial handshake, we send a new handshake signifying
//                     // what we want to listen to.
//                     let subscription = build_msg(Operation::SubscribeUpdate, Device::IPhone);
//                     let res = socket.send(&subscription).await?;
//                 }
//             }
//
//             Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
//                 continue;
//             }
//
//             Err(e) => {
//                 bail!(e)
//             }
//         }
//     }
//
//     Ok(())
// }

/// builds a message to send to the UDP server.
///
/// * `op`: the operation that we want to send out
/// * `device`: the type of device that is making the request.
fn build_msg(op: Operation, device: Device) -> BytesMut {
    let mut msg = BytesMut::with_capacity(12);
    msg.put_i32_le(device as i32);
    msg.put_i32_le(1);
    msg.put_i32_le(op as i32);

    msg
}
