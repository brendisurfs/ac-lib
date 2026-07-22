use bytes::{BufMut, BytesMut};

use crate::parser::{Device, Operation};

/// builds a message to be sent to the Assetto Corsa UDP server.
///
/// * `op`: which operation to send
/// * `device`: what kind of device is sending this message
pub(crate) fn build_udp_message(op: Operation, device: Device) -> BytesMut {
    let mut msg = BytesMut::with_capacity(12);
    msg.put_i32_le(device as i32);
    msg.put_i32_le(1);
    msg.put_i32_le(op as i32);

    msg
}
