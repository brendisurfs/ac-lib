mod byte_cursor;
use std::char::ParseCharError;

use thiserror::Error;

use crate::parser::byte_cursor::ByteCursor;

pub(crate) const LAP_INFO_LEN: usize = 212;
pub(crate) const CAR_INFO_LEN: usize = 328;
pub(crate) const HANDSHAKE_RES_LEN: usize = 408;

/// module errors
#[derive(Error, Debug)]
enum ParserError {
    /// If a parsing function receives an incorrect buffer sizing
    #[error("Received incorrect size of buffer: {0}")]
    IncorrectBufferSize(usize),

    #[error("i32 failed to convert: {0}")]
    I32ConversionFailed(String),

    #[error("u32 failed to convert: {0}")]
    U32ConversionFailed(String),

    #[error("f32 failed to convert: {0}")]
    F32ConversionFailed(String),

    #[error("bool failed to convert: {0}")]
    BoolConversionFailed(String),

    #[error("wheel stats failed to convert: {0}")]
    WheelsConversionFailed(String),

    #[error("Char failed to convert: {0}")]
    CharConversionFailed(String),
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

        let identifier = cursor.i32()?;
        let version = cursor.i32()?;

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
    fn from_bytes(buf: &[u8]) -> Result<Self, ParserError> {
        if buf.len() != CAR_INFO_LEN {
            return Err(ParserError::IncorrectBufferSize(buf.len()));
        }
        let mut c = ByteCursor::new(buf);

        let identifier = parse_utf8_chars(c.take(4))
            .parse()
            .map_err(|v: ParseCharError| {
                let err_str = v.to_string();
                ParserError::CharConversionFailed(err_str)
            })?;

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
    fn from_bytes(buf: &[u8]) -> Result<Self, ParserError> {
        if buf.len() != LAP_INFO_LEN {
            return Err(ParserError::IncorrectBufferSize(buf.len()));
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

#[cfg(test)]
mod parser_tests {

    use crate::parser::{CAR_INFO_LEN, CarInfo, IntoEvent};

    fn put_f32(buf: &mut [u8], offset: usize, val: f32) {
        buf[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
    }

    fn put_i32(buf: &mut [u8], offset: usize, val: i32) {
        buf[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
    }

    fn put_u32(buf: &mut [u8], offset: usize, val: u32) {
        buf[offset..offset + 4].copy_from_slice(&val.to_le_bytes());
    }

    // Builds a 328-byte CarInfo buffer where every distinct field/element is
    // a marker float/int equal to its own byte offset, so a wrong offset
    // reads a value that doesn't match its expected position.
    fn marker_car_info_buf() -> Vec<u8> {
        let mut buf = vec![0u8; CAR_INFO_LEN];
        // identifier: ASCII 'C' at byte 0, rest of the 4-byte field stays null
        buf[0] = b'C';

        put_i32(&mut buf, 4, 4); // size
        put_f32(&mut buf, 8, 8.0); // speed_kmh
        put_f32(&mut buf, 12, 12.0); // speed_mph
        put_f32(&mut buf, 16, 16.0); // speed_ms

        buf[20] = 1; // is_abs_enabled
        buf[21] = 1; // is_abs_in_action
        buf[22] = 1; // is_tc_in_action
        buf[23] = 1; // is_tc_enabled
        // 24, 25 = padding gap
        buf[26] = 1; // is_in_pit
        buf[27] = 1; // is_engine_limiter_on

        put_f32(&mut buf, 28, 28.0); // accg_vertical
        put_f32(&mut buf, 32, 32.0); // accg_horizontal
        put_f32(&mut buf, 36, 36.0); // accg_frontal

        put_u32(&mut buf, 40, 40); // lap_time
        put_u32(&mut buf, 44, 44); // last_lap
        put_u32(&mut buf, 48, 48); // best_lap
        put_u32(&mut buf, 52, 52); // lap_count

        put_f32(&mut buf, 56, 56.0); // gas
        put_f32(&mut buf, 60, 60.0); // brake
        put_f32(&mut buf, 64, 64.0); // clutch
        put_f32(&mut buf, 68, 68.0); // engine_rpm
        put_f32(&mut buf, 72, 72.0); // steer
        put_i32(&mut buf, 76, 76); // gear
        put_f32(&mut buf, 80, 80.0); // cg_height

        // 14 wheel groups of 16 bytes each start at 84, end at 84 + 14*16 = 308
        for i in 0..14 {
            let base = 84 + i * 16;
            for w in 0..4 {
                put_f32(&mut buf, base + w * 4, (base + w * 4) as f32);
            }
        }

        put_f32(&mut buf, 308, 308.0); // car_pos_normalized
        put_f32(&mut buf, 312, 312.0); // car_slope
        put_f32(&mut buf, 316, 316.0); // car_coordinates[0]
        put_f32(&mut buf, 320, 320.0); // car_coordinates[1]
        put_f32(&mut buf, 324, 324.0); // car_coordinates[2]

        buf
    }

    #[test]
    fn car_info_fields_land_on_documented_offsets() {
        let buf = marker_car_info_buf();
        let info = CarInfo::from_bytes(&buf).expect("328-byte buffer should parse");

        assert_eq!(info.size, 4);
        assert_eq!(info.speed_kmh, 8.0);
        assert_eq!(info.speed_mph, 12.0);
        assert_eq!(info.speed_ms, 16.0);

        assert!(info.is_abs_enabled);
        assert!(info.is_abs_in_action);
        assert!(info.is_tc_in_action);
        assert!(info.is_tc_enabled);
        assert!(info.is_in_pit);
        assert!(info.is_engine_limiter_on);

        assert_eq!(info.accg_vertical, 28.0);
        assert_eq!(info.accg_horizontal, 32.0);
        assert_eq!(info.accg_frontal, 36.0);

        assert_eq!(info.lap_time, 40);
        assert_eq!(info.last_lap, 44);
        assert_eq!(info.best_lap, 48);
        assert_eq!(info.lap_count, 52); // guards the historical off-by-10 bug

        assert_eq!(info.gas, 56.0);
        assert_eq!(info.brake, 60.0);
        assert_eq!(info.clutch, 64.0);
        assert_eq!(info.engine_rpm, 68.0);
        assert_eq!(info.steer, 72.0);
        assert_eq!(info.gear, 76);
        assert_eq!(info.cg_height, 80.0);

        assert_eq!(info.wheel_angular_speed, [84.0, 88.0, 92.0, 96.0]);
        assert_eq!(info.suspension_height, [292.0, 296.0, 300.0, 304.0]);

        assert_eq!(info.car_pos_normalized, 308.0);
        assert_eq!(info.car_slope, 312.0);
        assert_eq!(info.car_coordinates, [316.0, 320.0, 324.0]);
    }

    #[test]
    fn car_info_rejects_wrong_size_buffer() {
        let buf = vec![0u8; CAR_INFO_LEN - 1];
        assert!(CarInfo::from_bytes(&buf).is_err());
    }
}
