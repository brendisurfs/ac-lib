use anyhow::bail;
use bytes::{BufMut, BytesMut};
use thiserror::Error;

pub(crate) const LAP_INFO_LEN: usize = 212;
pub(crate) const CAR_INFO_LEN: usize = 328;
pub(crate) const HANDSHAKE_RES_LEN: usize = 408;

/// module errors
#[derive(Error, Debug)]
enum ParserError {
    /// If a parsing function receives an incorrect buffer sizing
    #[error("Received incorrect size of buffer: {0}")]
    InvalidSliceSize(usize),

    #[error("Unable to convert bytes to Event")]
    BytesConversionFailed,
}

/// Trait that maps to converting into an event struct
trait IntoEvent {
    fn from_bytes(buf: &[u8]) -> Result<Self, ParserError>
    where
        Self: Sized;
}

#[derive(Debug, Copy, Clone, Default)]
/// An identifier for the current device this library is running on.
/// Currently not used by AC, but required anyway.
pub enum Device {
    #[default]
    IPhone = 0,
    IPad = 1,
    AndroidPhone = 2,
    AndroidTablet = 3,
}

#[derive(Debug, Clone, Copy)]
/// our requested action to listen to or inform the UDP server of.
pub enum Operation {
    Handshake = 0,
    SubscribeUpdate = 1,
    SubscribeSpot = 2,
    Dismiss = 3,
}

/// Walks a byte buffer left to right, handing out correctly-sized slices
/// and primitives without requiring manually computed offsets.
struct ByteCursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> ByteCursor<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    /// Returns the next `n` bytes and advances the cursor past them.
    fn take(&mut self, n: usize) -> &'a [u8] {
        let slice = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        slice
    }

    /// Advances the cursor by `n` bytes without returning them.
    fn skip(&mut self, n: usize) {
        self.pos += n;
    }

    fn i32(&mut self) -> anyhow::Result<i32> {
        Ok(i32::from_le_bytes(self.take(4).try_into()?))
    }

    fn u32(&mut self) -> anyhow::Result<u32> {
        Ok(u32::from_le_bytes(self.take(4).try_into()?))
    }

    fn f32(&mut self) -> anyhow::Result<f32> {
        Ok(f32::from_le_bytes(self.take(4).try_into()?))
    }

    fn bool(&mut self) -> anyhow::Result<bool> {
        parse_bool_from_bytes(self.take(1))
    }

    fn wheels(&mut self) -> anyhow::Result<[f32; 4]> {
        parse_f32_wheels(self.take(16))
    }

    fn xyz(&mut self) -> anyhow::Result<[f32; 3]> {
        let x = self.f32()?;
        let y = self.f32()?;
        let z = self.f32()?;
        Ok([x, y, z])
    }
}

#[derive(Debug)]
pub struct HandshakeResponse {
    pub car_name: String,
    pub driver_name: String,
    pub identifier: i32,
    pub version: i32,
    pub track_name: String,
    pub track_config: String,
}

impl IntoEvent for HandshakeResponse {
    fn from_bytes(buf: &[u8]) -> Result<HandshakeResponse, ParserError> {
        let mut cursor = ByteCursor::new(buf);

        let car_name = parse_utf8_chars(cursor.take(100));
        let driver_name = parse_utf8_chars(cursor.take(100));

        let identifier = cursor
            .i32()
            .map_err(|_| ParserError::BytesConversionFailed)?;

        let version = cursor
            .i32()
            .map_err(|_| ParserError::BytesConversionFailed)?;

        let track_name = parse_to_utf16_chars(cursor.take(100));
        let track_config = parse_to_utf16_chars(cursor.take(100));

        Ok(HandshakeResponse {
            car_name,
            driver_name,
            identifier,
            version,
            track_name,
            track_config,
        })
    }
}

#[derive(Debug)]
pub struct CarInfo {
    pub identifier: char,
    pub size: i32,
    pub speed_kmh: f32,
    pub speed_mph: f32,
    pub speed_ms: f32,
    pub is_abs_enabled: bool,
    pub is_abs_in_action: bool,
    pub is_tc_in_action: bool,
    pub is_tc_enabled: bool,
    pub is_in_pit: bool,
    pub is_engine_limiter_on: bool,
    pub accg_vertical: f32,
    pub accg_horizontal: f32,
    pub accg_frontal: f32,
    pub lap_time: u32,
    pub last_lap: u32,
    pub best_lap: u32,
    pub lap_count: u32,
    pub gas: f32,
    pub brake: f32,
    pub clutch: f32,
    pub engine_rpm: f32,
    pub steer: f32,
    pub gear: i32,
    pub cg_height: f32,
    pub wheel_angular_speed: [f32; 4],
    pub slip_angle: [f32; 4],
    pub slip_angle_contact_patch: [f32; 4],
    pub slip_ratio: [f32; 4],
    pub tyre_slip: [f32; 4],
    pub nd_slip: [f32; 4],
    pub load: [f32; 4],
    pub dy: [f32; 4],
    pub mz: [f32; 4],
    pub tyre_dirty_level: [f32; 4],
    pub camber_rad: [f32; 4],
    pub tyre_radius: [f32; 4],
    pub tyre_loaded_radius: [f32; 4],
    pub suspension_height: [f32; 4],
    pub car_pos_normalized: f32,
    pub car_slope: f32,
    pub car_coordinates: [f32; 3],
}

impl IntoEvent for CarInfo {
    fn from_bytes(buf: &[u8]) -> anyhow::Result<Self> {
        if buf.len() != CAR_INFO_LEN {
            bail!("Incorrect buffer size");
        }
        let mut c = ByteCursor::new(buf);

        let identifier = parse_utf8_chars(c.take(4)).parse()?;
        let size = c.i32()?;
        let speed_kmh = c.f32()?;
        let speed_mph = c.f32()?;
        let speed_ms = c.f32()?;

        let is_abs_enabled = c.bool()?;
        let is_abs_in_action = c.bool()?;
        let is_tc_in_action = c.bool()?;
        let is_tc_enabled = c.bool()?;
        c.skip(2); // unused padding bytes (original ranges skipped 24..26)
        let is_in_pit = c.bool()?;
        let is_engine_limiter_on = c.bool()?;

        let accg_vertical = c.f32()?;
        let accg_horizontal = c.f32()?;
        let accg_frontal = c.f32()?;

        let lap_time = c.u32()?;
        let last_lap = c.u32()?;
        let best_lap = c.u32()?;
        let lap_count = c.u32()?;

        let gas = c.f32()?;
        let brake = c.f32()?;
        let clutch = c.f32()?;
        let engine_rpm = c.f32()?;
        let steer = c.f32()?;
        let gear = c.i32()?;
        let cg_height = c.f32()?;

        let wheel_angular_speed = c.wheels()?;
        let slip_angle = c.wheels()?;
        let slip_angle_contact_patch = c.wheels()?;
        let slip_ratio = c.wheels()?;
        let tyre_slip = c.wheels()?;
        let nd_slip = c.wheels()?;
        let load = c.wheels()?;
        let dy = c.wheels()?;
        let mz = c.wheels()?;
        let tyre_dirty_level = c.wheels()?;
        let camber_rad = c.wheels()?;
        let tyre_radius = c.wheels()?;
        let tyre_loaded_radius = c.wheels()?;
        let suspension_height = c.wheels()?;
        let car_pos_normalized = c.f32()?;
        let car_slope = c.f32()?;
        let car_coordinates = c.xyz()?;

        Ok(CarInfo {
            identifier,
            size,
            speed_kmh,
            speed_mph,
            speed_ms,
            is_abs_enabled,
            is_abs_in_action,
            is_tc_in_action,
            is_tc_enabled,
            is_in_pit,
            is_engine_limiter_on,
            accg_vertical,
            accg_horizontal,
            accg_frontal,
            lap_time,
            last_lap,
            best_lap,
            lap_count,
            gas,
            brake,
            clutch,
            engine_rpm,
            steer,
            gear,
            cg_height,
            wheel_angular_speed,
            slip_angle,
            slip_angle_contact_patch,
            slip_ratio,
            tyre_slip,
            nd_slip,
            load,
            dy,
            mz,
            tyre_dirty_level,
            camber_rad,
            tyre_radius,
            tyre_loaded_radius,
            suspension_height,
            car_pos_normalized,
            car_slope,
            car_coordinates,
        })
    }
}

#[derive(Debug)]
pub struct LapInfo {
    pub car_id_num: i32,
    pub lap: i32,
    pub time: i32,
    pub car_name: String,
    pub driver_name: String,
}
impl IntoEvent for LapInfo {
    fn from_bytes(buf: &[u8]) -> anyhow::Result<Self> {
        if buf.len() != LAP_INFO_LEN {
            bail!("Incorrect buffer size");
        }

        Ok(LapInfo {
            car_id_num: i32::default(),
            time: i32::default(),
            lap: i32::default(),
            car_name: String::default(),
            driver_name: String::default(),
        })
    }
}

// the kind of message we can receive from the UDP server
// reference for parsing: https://docs.google.com/spreadsheets/d/1PhWgG1B7cv38OEummTZOOItrE-yYRBpMI2nV92BfDFU/pubhtml?gid=0&single=true
#[derive(Debug)]
pub enum Event {
    HandshakeResponse,
    CarInfo,
    LapInfo,
}

/// A central data structure that is used to communicate event subscriptions with the AC server.
///
/// * `identifier`: the kind of device this client is running on.
/// * `version`: the AC version (apparently not used with the current UDP impl).
/// * `operation`: the Kind of the operation we want to request from the UDP socket.
#[derive(Debug)]
pub struct Handshake {
    pub identifier: Device,
    pub version: i32,
    pub operation: Operation,
}

/// parses a bunch of chars from the UDP server and converts them to correct format (utf8).
///
/// * `buf`: the slice of data to convert to string.
fn parse_utf8_chars(buf: &[u8]) -> String {
    String::from_utf8_lossy(buf)
        .chars()
        .filter(|v| v.ne(&'\0') && v.ne(&'%'))
        .collect::<String>()
}

/// parses a buffer
///
/// * `buf`:
fn parse_to_utf16_chars(buf: &[u8]) -> String {
    let converted_bytes = &buf
        .iter()
        .map(|v| u16::from(*v).to_le())
        .collect::<Vec<_>>();

    String::from_utf16_lossy(converted_bytes)
        .chars()
        .filter(|v| v.ne(&'\0') && v.ne(&'%'))
        .collect::<String>()
}

/// extracts a bool from the given byte slice.
///
/// * `buf`: the buffer to extrac the bool from
/// * errors if the buffer cannot be made into a u8 from le bytes.
fn parse_bool_from_bytes(buf: &[u8]) -> anyhow::Result<bool> {
    let parsed_num = u8::from_le_bytes(buf.try_into()?);

    Ok(!matches!(parsed_num, 0))
}

/// parses a group of wheel stats from a buffer range.
///
/// * `buf`: the buffer to extract the ranges from.
fn parse_f32_wheels(buf: &[u8]) -> anyhow::Result<[f32; 4]> {
    if buf.len() < 20 {
        bail!("Incorrect buffer size: {:?}", buf.len());
    }

    let front_left = f32::from_le_bytes(buf[0..4].try_into()?);
    let front_right = f32::from_le_bytes(buf[4..8].try_into()?);
    let back_left = f32::from_le_bytes(buf[8..12].try_into()?);
    let back_right = f32::from_le_bytes(buf[12..16].try_into()?);

    Ok([front_left, front_right, back_left, back_right])
}

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

#[cfg(test)]
mod parser_tests {

    use crate::parser::parse_f32_wheels;

    // Wheels parse on correct input
    #[test]
    fn wheels_parse_correct_size_buf() {
        let front_left: [u8; 5] = [10, 12, 14, 16, 18];
        let front_right: [u8; 5] = [10, 12, 14, 16, 18];
        let back_left: [u8; 5] = [10, 12, 14, 16, 18];
        let back_right: [u8; 5] = [10, 12, 14, 16, 18];

        let buf = [front_left, front_right, back_left, back_right].concat();

        let res = parse_f32_wheels(&buf);
        assert!(res.is_ok(), "Buffer should be correct size and parse.");
    }
    // Wheels dont parse on incorrect input and handles error bounds
    #[test]
    fn wheels_parse_incorrect_size_buf() {
        let front_left: [u8; 5] = [10, 12, 14, 16, 18];
        let front_right: [u8; 5] = [10, 12, 14, 16, 18];
        let back_left: [u8; 5] = [10, 12, 14, 16, 18];
        let back_right: [u8; 4] = [10, 12, 14, 16];

        let mut buf = [front_left, front_right, back_left].concat();
        buf.append(&mut back_right.to_vec());

        let res = parse_f32_wheels(&buf);
        assert!(res.is_err(), "Error boundary should be caught");
    }
}
