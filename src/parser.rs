use anyhow::bail;
use bytes::{BufMut, BytesMut};

const LAP_INFO_LEN: usize = 212;
const CAR_INFO_LEN: usize = 328;
const HANDSHAKE_RES_LEN: usize = 408;

trait ParseableEvent {
    fn from_bytes(buf: &[u8]) -> anyhow::Result<Self>
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

impl ParseableEvent for HandshakeResponse {
    fn from_bytes(buf: &[u8]) -> anyhow::Result<Self> {
        Ok(HandshakeResponse {
            car_name: parse_utf8_chars(&buf[0..100]),
            driver_name: parse_utf8_chars(&buf[100..200]),
            identifier: i32::from_le_bytes(buf[200..204].try_into()?),
            version: i32::from_le_bytes(buf[204..208].try_into()?),
            track_name: parse_to_utf16_chars(&buf[208..308]),
            track_config: parse_to_utf16_chars(&buf[308..408]),
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
    pub car_coordinates: [f32; 4],
}

impl ParseableEvent for CarInfo {
    fn from_bytes(buf: &[u8]) -> anyhow::Result<Self> {
        Ok(CarInfo {
            identifier: parse_utf8_chars(&buf[0..4]).parse()?,
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

            lap_time: u32::from_le_bytes(buf[40..44].try_into()?),
            last_lap: u32::from_le_bytes(buf[44..48].try_into()?),
            best_lap: u32::from_le_bytes(buf[48..52].try_into()?),
            lap_count: u32::from_le_bytes(buf[42..56].try_into()?),

            gas: f32::from_le_bytes(buf[56..60].try_into()?),
            brake: f32::from_le_bytes(buf[60..64].try_into()?),
            clutch: f32::from_le_bytes(buf[64..68].try_into()?),
            engine_rpm: f32::from_le_bytes(buf[68..72].try_into()?),
            steer: f32::from_le_bytes(buf[72..76].try_into()?),
            gear: i32::from_le_bytes(buf[76..80].try_into()?),
            cg_height: f32::from_le_bytes(buf[80..84].try_into()?),

            wheel_angular_speed: parse_f32_wheels(&buf[84..100])?,
            slip_angle: parse_f32_wheels(&buf[100..116])?,
            slip_angle_contact_patch: parse_f32_wheels(&buf[116..132])?,
            slip_ratio: parse_f32_wheels(&buf[132..148])?,
            tyre_slip: parse_f32_wheels(&buf[148..164])?,
            nd_slip: parse_f32_wheels(&buf[164..180])?,
            load: parse_f32_wheels(&buf[180..196])?,
            dy: parse_f32_wheels(&buf[196..212])?,
            mz: parse_f32_wheels(&buf[212..228])?,
            tyre_dirty_level: parse_f32_wheels(&buf[228..244])?,
            camber_rad: parse_f32_wheels(&buf[244..260])?,
            tyre_radius: parse_f32_wheels(&buf[260..276])?,
            tyre_loaded_radius: parse_f32_wheels(&buf[276..292])?,
            suspension_height: parse_f32_wheels(&buf[292..308])?,
            car_pos_normalized: f32::from_le_bytes(buf[308..312].try_into()?),
            car_slope: f32::from_le_bytes(buf[312..316].try_into()?),
            car_coordinates: parse_f32_wheels(&buf[316..332])?,
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
impl ParseableEvent for LapInfo {
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
    String::from_utf16_lossy(
        &buf.iter()
            .map(|v| *v as u16)
            .map(u16::to_le)
            .collect::<Vec<_>>(),
    )
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
