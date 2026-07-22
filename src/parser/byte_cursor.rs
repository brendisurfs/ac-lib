use anyhow::bail;

use crate::parser::ParserError;

/// Walks a byte buffer left to right, handing out correctly-sized slices
/// and primitives without requiring manually computed offsets.
pub(super) struct ByteCursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> ByteCursor<'a> {
    pub(super) fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    /// Returns the next `n` bytes and advances the cursor past them.
    pub(super) fn take(&mut self, n: usize) -> &'a [u8] {
        let slice = &self.buf[self.pos..self.pos + n];
        self.pos += n;
        slice
    }

    /// Advances the cursor by `n` bytes without returning them.
    pub(super) fn skip(&mut self, n: usize) {
        self.pos += n;
    }

    pub(super) fn i32(&mut self) -> Result<i32, ParserError> {
        self.take(4)
            .try_into()
            .map(i32::from_le_bytes)
            .map_err(|e| ParserError::I32ConversionFailed(e.to_string()))
    }

    pub(super) fn u32(&mut self) -> Result<u32, ParserError> {
        self.take(4)
            .try_into()
            .map(u32::from_le_bytes)
            .map_err(|e| ParserError::U32ConversionFailed(e.to_string()))
    }

    pub(super) fn f32(&mut self) -> Result<f32, ParserError> {
        self.take(4)
            .try_into()
            .map(f32::from_le_bytes)
            .map_err(|e| ParserError::F32ConversionFailed(e.to_string()))
    }

    pub(super) fn bool(&mut self) -> Result<bool, ParserError> {
        parse_bool_from_bytes(self.take(1))
            .map_err(|e| ParserError::BoolConversionFailed(e.to_string()))
    }

    pub(super) fn wheels(&mut self) -> Result<[f32; 4], ParserError> {
        parse_f32_wheels(self.take(16))
            .map_err(|e| ParserError::WheelsConversionFailed(e.to_string()))
    }

    pub(super) fn xyz(&mut self) -> Result<[f32; 3], ParserError> {
        let x = self.f32()?;
        let y = self.f32()?;
        let z = self.f32()?;
        Ok([x, y, z])
    }
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
    if buf.len() < 16 {
        bail!("Incorrect buffer size: {:?}", buf.len());
    }

    let front_left = f32::from_le_bytes(buf[0..4].try_into()?);
    let front_right = f32::from_le_bytes(buf[4..8].try_into()?);
    let back_left = f32::from_le_bytes(buf[8..12].try_into()?);
    let back_right = f32::from_le_bytes(buf[12..16].try_into()?);

    Ok([front_left, front_right, back_left, back_right])
}

#[cfg(test)]
mod cursor_tests {
    use crate::parser::byte_cursor::parse_f32_wheels;

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
        // parse_f32_wheels reads exactly 16 bytes (4 f32s); one byte short
        // must be rejected rather than silently truncated/panicking.
        let buf = vec![0u8; 15];

        let res = parse_f32_wheels(&buf);
        assert!(res.is_err(), "Error boundary should be caught");
    }
}
