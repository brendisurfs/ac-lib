//! This is a goofy little project to learn about displaying data from assetto corsa.
//! reference for data: https://docs.google.com/document/d/1KfkZiIluXZ6mMhLWfDX1qAGbvhGRC3ZUzjVIt5FQpp4/pub
//! also referrence: https://github.com/rickwest/ac-remote-telemetry-client/blob/master/src/parsers/RTCarInfoParser.js

mod parser;

use std::time::Duration;

use anyhow::anyhow;
use exponential_backoff::Backoff;
use parser::{Device, Event, Operation, build_udp_message};
use tokio::net::{ToSocketAddrs, UdpSocket};

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
    pub async fn new<A>(remote_addr: A, device: Device) -> anyhow::Result<Self>
    where
        A: ToSocketAddrs,
    {
        // NOTE : (3/22/2025) this needs to be chosen by the OS, or else it will never pick up.
        // However, this may change if the setup is on ios.
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;

        let min_duration = Duration::from_secs(1);
        let max_duration = Duration::from_secs(10);

        let backoff = Backoff::new(MAX_ATTEMPTS, min_duration, max_duration);

        for duration in backoff {
            match socket.connect(&remote_addr).await {
                Ok(()) => break,
                Err(why) => {
                    eprintln!("Error connecting: retrying...");

                    match duration {
                        Some(sleep_time) => tokio::time::sleep(sleep_time).await,
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
    pub async fn send_message(&self, operation: Operation) -> anyhow::Result<()> {
        let msg = build_udp_message(operation, self.device);
        self.socket.send(&msg).await?;

        Ok(())
    }

    /// receives the next event on the server.
    pub async fn recv_event(&self) -> anyhow::Result<Event> {
        // NOTE: The buffer we write to must be large enough, or else we may not get enough data.
        // TODO: calculate appropriate max size buffer to read into.
        let mut buf = vec![0u8; 1024];
        let read_size = self.socket.recv(&mut buf).await?;

        Event::from_bytes(read_size, &buf)
    }
}

#[cfg(test)]
mod lib_tests {
    use tokio::net::UdpSocket;

    // Builds a test socket listener to confirm messages.
    async fn build_socket_listener() {
        let listener = UdpSocket::bind("0.0.0.0:0").await.expect("failed to bind.");
    }

    #[tokio::test]
    async fn test_send_message() {
        todo!("Implement send message test");
    }

    #[tokio::test]
    async fn test_recv_event() {
        todo!("Implement receive event");
    }
}
