use anyhow::bail;
use bytes::{BufMut, BytesMut};

#[derive(Debug, Copy, Clone)]
/// An identifier for the current device this library is running on.
pub enum Device {
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

// char identifier [0;4];
// int size;
// float speed_Kmh;
// float speed_Mph;
// float speed_Ms;
//
// bool isAbsEnabled;
// bool isAbsInAction;
// bool isTcInAction;
// bool isTcEnabled;
// bool isInPit;
// bool isEngineLimiterOn;
//
//
// float accG_vertical;
// float accG_horizontal;
// float accG_frontal;
//
// int lapTime;
// int lastLap;
// int bestLap;
// int lapCount;
//
// float gas;
// float brake;
// float clutch;
// float engineRPM;
// float steer;
// int gear;
// float cgHeight;
//
// float wheelAngularSpeed[4];
// float slipAngle[4];
// float slipAngle_ContactPatch[4];
// float slipRatio[4];
// float tyreSlip[4];
// float ndSlip[4];
// float load[4];
// float Dy[4];
// float Mz[4];
// float tyreDirtyLevel[4];
//
// float camberRAD[4];
// float tyreRadius[4];
// float tyreLoadedRadius[4];
// float suspensionHeight[4];
// float carPositionNormalized;
// float carSlope;
// float carCoordinates[3];
//
// the kind of message we can receive from the UDP server
// reference for parsing: https://docs.google.com/spreadsheets/d/1PhWgG1B7cv38OEummTZOOItrE-yYRBpMI2nV92BfDFU/pubhtml?gid=0&single=true
#[derive(Debug)]
pub enum Event {
    HandshakeResponse {
        /// utf8
        car_name: String,
        /// utf8
        driver_name: String,
        identifier: i32,
        version: i32,
        /// utf16le
        track_name: String,
        /// utf16le
        track_config: String,
    },
    CarInfo {
        identifier: String,
        size: i32,
        speed_kmh: f32,
        speed_mph: f32,
        speed_ms: f32,
        // bool isAbsEnabled; 20
        is_abs_enabled: bool,
        // bool isAbsInAction; 21
        is_abs_in_action: bool,
        // bool isTcInAction; 22
        is_tc_in_action: bool,
        // bool isTcEnabled; 23
        is_tc_enabled: bool,
        // bool isInPit; 24
        is_in_pit: bool,
        // bool isEngineLimiterOn; 25
        is_engine_limiter_on: bool,

        // float accG_vertical;
        accg_vertical: f32,
        // float accG_horizontal;
        accg_horizontal: f32,
        // float accG_frontal;
        accg_frontal: f32,
    },
    LapInfo {
        car_id_num: i32,
        lap: i32,
        driver_name: String,
        car_name: String,
        time: i32,
    },
}

impl Event {
    pub fn from_bytes(size: usize, buf: &[u8]) -> anyhow::Result<Event> {
        let ev = match size {
            408 => Self::HandshakeResponse {
                car_name: parse_utf8_chars(&buf[0..100]),
                driver_name: parse_utf8_chars(&buf[100..200]),
                identifier: i32::from_le_bytes(buf[200..204].try_into()?),
                version: i32::from_le_bytes(buf[204..208].try_into()?),
                track_name: parse_utf16_chars(
                    &buf[208..308].iter().map(|v| *v as u16).collect::<Vec<_>>(),
                ),

                track_config: parse_utf16_chars(
                    &buf[308..408]
                        .iter()
                        .map(|v| *v as u16)
                        .map(|v| v.to_le())
                        .collect::<Vec<_>>(),
                ),
            },
            328 => Self::CarInfo {
                identifier: parse_utf8_chars(&buf[0..4]),
                size: i32::from_le_bytes(buf[4..8].try_into()?),
                speed_kmh: f32::from_le_bytes(buf[8..12].try_into()?),
                speed_mph: f32::from_le_bytes(buf[12..16].try_into()?),
                speed_ms: f32::from_le_bytes(buf[16..20].try_into()?),

                is_abs_enabled: parse_bool_from_bytes(&buf[20..21])?,
                is_abs_in_action: parse_bool_from_bytes(&buf[21..22])?,
                is_tc_in_action: parse_bool_from_bytes(&buf[22..23])?,
                is_tc_enabled: parse_bool_from_bytes(&buf[23..24])?,
                is_in_pit: parse_bool_from_bytes(&buf[26..27])?,
                is_engine_limiter_on: parse_bool_from_bytes(&buf[27..28])?,

                accg_vertical: f32::from_le_bytes(buf[28..32].try_into()?),
                accg_horizontal: f32::from_le_bytes(buf[32..36].try_into()?),
                accg_frontal: f32::from_le_bytes(buf[36..40].try_into()?),
            },

            212 => Self::LapInfo {
                car_id_num: i32::default(),
                time: i32::default(),
                lap: i32::default(),
                car_name: String::default(),
                driver_name: String::default(),
            },
            _ => bail!("No matching size found for message"),
        };
        Ok(ev)
    }
}

/// A central data structure that is used to communicate event subscriptions with the AC server.
///
/// * `identifier`:
/// * `version`:
/// * `operation_id`:
#[derive(Debug)]
pub struct Handshake {
    pub identifier: Device,
    pub version: i32,
    pub operation_id: Operation,
}

/// parses a bunch of chars from the UDP server and converts them to correct format (utf8).
///
/// * `buf`: the slice of data to convert to string.
pub fn parse_utf8_chars(buf: &[u8]) -> String {
    String::from_utf8_lossy(buf)
        .trim_matches('\0')
        .chars()
        .filter(|v| v.ne(&'\0') && v.ne(&'%'))
        .collect::<String>()
}

/// parses a buffer
///
/// * `buf`:
pub(crate) fn parse_utf16_chars(buf: &[u16]) -> String {
    String::from_utf16_lossy(buf)
        .chars()
        .filter(|v| v.ne(&'\0') && v.ne(&'%'))
        .collect::<String>()
}

/// extracts a bool from the given byte slice.
///
/// * `buf`: the buffer to extrac the bool from
/// * errors if the buffer cannot be made into a u8 from le bytes.
pub(crate) fn parse_bool_from_bytes(buf: &[u8]) -> anyhow::Result<bool> {
    Ok(!matches!(u8::from_le_bytes(buf.try_into()?), 0))
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
