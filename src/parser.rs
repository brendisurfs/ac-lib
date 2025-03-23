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
// float gas [56-60];
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
        identifier: char,
        size: i32,
        speed_kmh: f32,
        speed_mph: f32,
        speed_ms: f32,
        is_abs_enabled: bool,
        is_abs_in_action: bool,
        is_tc_in_action: bool,
        is_tc_enabled: bool,
        is_in_pit: bool,
        is_engine_limiter_on: bool,
        accg_vertical: f32,
        accg_horizontal: f32,
        accg_frontal: f32,
        /// unit: ms
        lap_time: u32,
        /// unit: ms
        last_lap: u32,
        /// Unit: ms
        best_lap: u32,
        lap_count: u32,

        // NOTE: ratio
        gas: f32,

        // NOTE: ratio
        brake: f32,

        // NOTE: ratio
        clutch: f32,
        engine_rpm: f32,
        steer: f32,

        /// NOTE: 0=R, 1=N, 2=1st, etc.
        gear: i32,
        cg_height: f32,
        /// each 4x4 for each wheel
        wheel_angular_speed: [f32; 4],
        slip_angle: [f32; 4],
        slip_angle_contact_patch: [f32; 4],
        slip_ratio: [f32; 4],
        tyre_slip: [f32; 4],
        nd_slip: [f32; 4],
        load: [f32; 4],
        dy: [f32; 4],
        mz: [f32; 4],
        tyre_dirty_level: [f32; 4],
        camber_rad: [f32; 4],
        tyre_radius: [f32; 4],
        tyre_loaded_radius: [f32; 4],
        suspension_height: [f32; 4],

        car_pos_normalized: f32,
        car_slope: f32,
        car_coordinates: [f32; 4],
    },
    LapInfo {
        car_id_num: i32,
        lap: i32,
        time: i32,
        car_name: String,
        driver_name: String,
    },
}

impl Event {
    /// converts the bytes received, depending on the size, to an Event.
    /// Will return an Error if a byte size is not matched.
    ///
    /// * `size`: the size of the bytes received.
    /// * `buf`: the buffer of bytes to parse.
    pub fn from_bytes(size: usize, buf: &[u8]) -> anyhow::Result<Event> {
        let ev = match size {
            408 => Self::HandshakeResponse {
                car_name: parse_utf8_chars(&buf[0..100]),
                driver_name: parse_utf8_chars(&buf[100..200]),
                identifier: i32::from_le_bytes(buf[200..204].try_into()?),
                version: i32::from_le_bytes(buf[204..208].try_into()?),
                track_name: parse_to_utf16_chars(&buf[208..308]),
                track_config: parse_to_utf16_chars(&buf[308..408]),
            },
            328 => Self::CarInfo {
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
pub fn parse_utf8_chars(buf: &[u8]) -> String {
    String::from_utf8_lossy(buf)
        .chars()
        .filter(|v| v.ne(&'\0') && v.ne(&'%'))
        .collect::<String>()
}

/// parses a buffer
///
/// * `buf`:
pub(crate) fn parse_to_utf16_chars(buf: &[u8]) -> String {
    String::from_utf16_lossy(
        &buf.iter()
            .map(|v| *v as u16)
            .map(|v| v.to_le())
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
pub(crate) fn parse_bool_from_bytes(buf: &[u8]) -> anyhow::Result<bool> {
    let parsed_num = u8::from_le_bytes(buf.try_into()?);

    Ok(!matches!(parsed_num, 0))
}

/// parses a group of wheel stats from a buffer range.
///
/// * `buf`: the buffer to extract the ranges from.
pub(crate) fn parse_f32_wheels(buf: &[u8]) -> anyhow::Result<[f32; 4]> {
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
