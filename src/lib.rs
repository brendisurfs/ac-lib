//! reference for data: https://docs.google.com/document/d/1KfkZiIluXZ6mMhLWfDX1qAGbvhGRC3ZUzjVIt5FQpp4/pub
//! also referrence: https://github.com/rickwest/ac-remote-telemetry-client/blob/master/src/parsers/RTCarInfoParser.js

mod parser;

use std::{
    io,
    net::{ToSocketAddrs, UdpSocket},
    time::Duration,
};

use anyhow::{anyhow, bail};
use bytes::{BufMut, BytesMut};
use exponential_backoff::Backoff;
use parser::{Device, Event, Operation};

use crate::parser::{CAR_INFO_LEN, HANDSHAKE_RES_LEN, LAP_INFO_LEN};

/// Exponential backoff maximum attempts.
const MAX_ATTEMPTS: u32 = 3;

/// A Client connects to the remote Assetto Corsa UDP server,
/// allowing the user to receive UDP telemetry updates about the current session.
///
/// * `device`: what kind of device is this client running on
/// * `socket`: the socket for the client to run on.
pub struct Client {
    device: Device,
    socket: UdpSocket,
}

impl Client {
    /// creates a new Assetto Corsa UDP Client
    ///
    /// * `remote_addr`:  the addr the ACServer is running on
    /// * `device`:  the device this client is running on
    pub fn new<A>(remote_addr: A, device: Device) -> anyhow::Result<Self>
    where
        A: ToSocketAddrs,
    {
        // NOTE : (3/22/2025) this needs to be chosen by the OS, or else it will never pick up.
        // However, this may change if the setup is on ios.
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        let min_duration = Duration::from_secs(1);
        let max_duration = Duration::from_secs(10);

        let backoff = Backoff::new(MAX_ATTEMPTS, min_duration, max_duration);

        for duration in backoff {
            match socket.connect(&remote_addr) {
                Ok(()) => break,
                Err(why) => {
                    eprintln!("Error connecting: retrying...");

                    match duration {
                        Some(sleep_time) => std::thread::sleep(sleep_time),
                        None => return Err(anyhow!(why)),
                    }
                }
            }
        }

        Ok(Self { socket, device })
    }

    /// sends a message to the udp server.
    ///
    /// * `operation`: kind of op we want the udp server to update on.
    pub fn send_message(&self, operation: Operation) -> io::Result<usize> {
        let msg = self.build_udp_message(operation);
        self.socket.send(&msg)
    }

    /// receives the next event on the server.
    pub fn recv_raw_event_buffer(&self) -> anyhow::Result<(Event, [u8; 1024])> {
        // NOTE: The buffer we write to must be large enough, or else we may not get enough data.
        // TODO: calculate appropriate max size buffer to read into.
        let mut buf = [0u8; 1024];
        let read_size = self.socket.recv(&mut buf)?;

        let ac_event = match read_size {
            HANDSHAKE_RES_LEN => Event::HandshakeResponse,
            CAR_INFO_LEN => Event::CarInfo,
            LAP_INFO_LEN => Event::LapInfo,
            _ => bail!("No matching size found for message"),
        };

        Ok((ac_event, buf))
    }

    /// builds a message to be sent to the Assetto Corsa UDP server.
    ///
    /// * `op`: which operation to send
    /// * `device`: what kind of device is sending this message
    fn build_udp_message(&self, op: Operation) -> BytesMut {
        let mut msg = BytesMut::with_capacity(12);
        msg.put_i32_le(self.device as i32);
        msg.put_i32_le(1);
        msg.put_i32_le(op as i32);

        msg
    }
}

#[cfg(test)]
mod lib_tests {
    use crate::{Client, parser::Device};
    use std::net::UdpSocket;

    // Builds a test socket listener to confirm messages, bound to an OS-assigned port.
    fn build_socket_listener() -> UdpSocket {
        UdpSocket::bind("127.0.0.1:0").expect("failed to bind UDP socket.")
    }

    #[test]
    fn test_connect_to_remote() {
        let remote_socket = build_socket_listener();

        let remote_addr = remote_socket
            .local_addr()
            .expect("failed to get local addr");

        let client = Client::new(remote_addr, Device::default());
        assert!(client.is_ok(), "Expected client to connect");
    }

    #[test]
    fn test_send_handshake() {
        let remote_socket = build_socket_listener();

        let remote_addr = remote_socket
            .local_addr()
            .expect("failed to get local addr");

        let client =
            Client::new(remote_addr, Device::default()).expect("failed to connect to remote");

        let send_msg = client.send_message(crate::parser::Operation::Handshake);
        assert!(send_msg.is_ok(), "Expected message to be sent.");
        assert_eq!(send_msg.unwrap(), 12, "Sent bytes should be 12");
    }

    #[test]
    fn test_recv_message() {
        let remote_socket = build_socket_listener();

        let remote_addr = remote_socket
            .local_addr()
            .expect("failed to get local addr");

        let client =
            Client::new(remote_addr, Device::default()).expect("failed to connect to remote");

        let send_msg = client.send_message(crate::parser::Operation::Handshake);
        assert!(send_msg.is_ok(), "Expected message to be sent.");
        assert_eq!(send_msg.unwrap(), 12, "Sent bytes should be 12");
    }
}
