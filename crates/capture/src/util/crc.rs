use crate::error::CaptureError;

/// CRC-16/DNP (poly 0x3D65 reflected, init 0x0000, final xor 0xFFFF)
fn crc16_dnp(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xA6BC } else { crc >> 1 };
        }
    }
    !crc
}

/// Takes a buffer starting at the `05 64` start bytes, verifies every CRC,
/// and returns the frame without them
pub fn strip_crcs(raw: &[u8]) -> Result<Vec<u8>, CaptureError> {
    const HEADER_LEN: usize = 10; // 8 bytes of header + 2-byte CRC
    const BLOCK_DATA: usize = 16; // user data comes in 16-byte blocks, each followed by a CRC

    if raw.len() < HEADER_LEN || raw[..2] != [0x05, 0x64] {
        return Err(CaptureError::NotDNP3Packet);
    }

    // Header CRC
    if crc16_dnp(&raw[..8]) != u16::from_le_bytes([raw[8], raw[9]]) {
        return Err(CaptureError::FormatError("Header CRC mismatch".to_string()));
    }

    // The length field counts ctrl + dest + src (5 bytes) + user data, not CRCs
    let length = raw[2] as usize;
    if length < 5 {
        return Err(CaptureError::FormatError(format!("Invalid length field: {length}")));
    }
    let user_len = length - 5;
    let total = HEADER_LEN + user_len + 2 * user_len.div_ceil(BLOCK_DATA);

    if raw.len() < total {
        return Err(CaptureError::FormatError(format!(
            "Truncated frame: expected {total} bytes, got {}",
            raw.len()
        )));
    }

    let mut clean = Vec::with_capacity(8 + user_len);
    clean.extend_from_slice(&raw[..8]);

    // Data blocks: up to 16 bytes of data + 2-byte CRC
    for block in raw[HEADER_LEN..total].chunks(BLOCK_DATA + 2) {
        let (data, crc) = block.split_at(block.len() - 2);
        if crc16_dnp(data) != u16::from_le_bytes([crc[0], crc[1]]) {
            return Err(CaptureError::FormatError("Data block CRC mismatch".to_string()));
        }
        clean.extend_from_slice(data);
    }

    Ok(clean)
}