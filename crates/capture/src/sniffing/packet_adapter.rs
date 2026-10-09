use crate::domain::dnp3_packet::{ApplicationHeader, ControlField, DataLinkHeader, Dnp3Frame, Dnp3Object, Dnp3Point, FunctionCode, IinFlags, Qualifier, TransportHeader};
use crate::domain::packet_adapter::PacketAdapter;
use crate::error::CaptureError;
use crate::sniffing::buffer_reader::BufferReader;
use crate::sniffing::parsers::point_reader::{read_point, PointLayout};

const DNP3_START_BYTES: [u8;2] = [0x05, 0x64];

pub struct PcapPacketAdapter {}

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
        return Err(CaptureError::FormatError("Not a DNP3 link frame".to_string()));
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

fn get_dnp3_slice(raw_packet_data: &[u8]) -> Result<Vec<u8>, CaptureError> {
    let start_idx = raw_packet_data.windows(2).position(|w| w == DNP3_START_BYTES).unwrap_or(raw_packet_data.len());

    let dnp3_slice = &raw_packet_data[start_idx..];

    let clean = strip_crcs(&dnp3_slice);

    clean
}

fn parse_data_link_header(reader: &mut BufferReader) -> DataLinkHeader {
    let mut header = DataLinkHeader::default();

    header.frame_length = reader.u8();
    header.control = ControlField::from_u8(reader.u8());
    header.destination = reader.u16();
    header.source = reader.u16();

    header
}

fn parse_transport_header(reader: &mut BufferReader) -> TransportHeader {
    let mut header = TransportHeader::default();

    let value = reader.u8();

    header.fin = value & 0x80 != 0;
    header.fir = value & 0x40 != 0;
    header.sequence = value & 0x3F;

    header
}

fn parse_application_header(reader: &mut BufferReader, parse_iin: bool) -> ApplicationHeader {
    let mut header = ApplicationHeader::default();

    // Control octet
    let control = reader.u8();
    header.fir = control & 0x80 != 0;
    header.fin = control & 0x40 != 0;
    header.confirm_requested = control & 0x20 == 1;
    header.unsolicited = control & 0x10 == 0x10;
    header.sequence = control & 0x0F;

    header.function_code = FunctionCode::from_u8(reader.u8());

    if(parse_iin) {
        header.iin = Some(IinFlags::from_u16(reader.u16()));
    } else {
        header.iin = None;
    }

    header
}

fn parse_points(reader: &mut BufferReader, object: &Dnp3Object, info: PointLayout) -> Result<Vec<Dnp3Point>, CaptureError> {
    let mut points = Vec::new();

    match object.qualifier {
        Qualifier::StartStop8(start, stop) => {
            let (start_idx, stop_idx) = (start as u32, stop as u32);
            for i in start_idx..stop_idx+1 {
                let point = read_point(i, &info, reader);
                points.push(point.unwrap());
            }
        },
        Qualifier::StartStop16(start, stop) => {
            let (start_idx, stop_idx) = (start as u32, stop as u32);
            for i in start_idx..stop_idx+1 {
                let point = read_point(i, &info, reader);
                points.push(point.unwrap());
            }
        },
        Qualifier::Count8(count) => {
            let count_32 = count as u32;
            for i in 0..count_32 {
                let point = read_point(i, &info, reader);
                points.push(point.unwrap());
            }
        },
        Qualifier::Count16(count) => {
            let count_32 = count as u32;
            for i in 0..count_32 {
                let point = read_point(i, &info, reader);
                points.push(point.unwrap());
            }
        },
        Qualifier::CountWithIndex8(count) => {
            for i in 0..count {
                let index = reader.u8() as u32;
                let point = read_point(index, &info, reader);
                points.push(point.unwrap());
            }
        },
        Qualifier::CountWithIndex16(count) => {
            for i in 0..count {
                let index = reader.u16() as u32;
                let point = read_point(index, &info, reader);
                points.push(point.unwrap());
            }
        }
        _ => return Err(CaptureError::FormatError(format!("Unsupported qualifier {:?}", object.qualifier)))
    }

    Ok(points)
}

fn parse_objects(reader: &mut BufferReader) -> Vec<Dnp3Object> {
    let mut objects = Vec::new();

    while !reader.eof() {
        let mut object = Dnp3Object::default();
        object.group = reader.u8();
        object.variation = reader.u8();

        let qualifier_val = reader.u8();

        object.qualifier = match qualifier_val {
            0x00 => Qualifier::StartStop8(reader.u8(), reader.u8()),
            0x01 => Qualifier::StartStop16(reader.u16(), reader.u16()),
            0x06 => Qualifier::AllObjects,
            0x07 => Qualifier::Count8(reader.u8()),
            0x08 => Qualifier::Count16(reader.u16()),
            0x17 => Qualifier::CountWithIndex8(reader.u8()),
            0x28 => Qualifier::CountWithIndex16(reader.u16()),
            _ => Qualifier::Other(qualifier_val),
        };

        let point_info = PointLayout::from_gv(object.group, object.variation);

        match point_info {
            Ok(info) => {
                let points = parse_points(reader, &object, info);

                match points {
                    Ok(points) => object.points = points,
                    Err(error) => {
                        object.points = Vec::new();
                        println!("{}", error);
                        break;
                    },
                }
            },
            Err(error) => {
                println!("{}", error);
                break;
            }
        }
        objects.push(object);
    }

    objects
}

impl PacketAdapter for PcapPacketAdapter {
    fn adapt(&self, raw: &[u8]) -> Result<Dnp3Frame, CaptureError> {
        let dnp3_data = get_dnp3_slice(raw);

        let mut frame = Dnp3Frame::default();

        let mut reader = BufferReader::new(dnp3_data?.leak());

        reader.skip(2); // Skip 0x05 and 0x64 bytes

        frame.data_link = parse_data_link_header(&mut reader);
        frame.transport = parse_transport_header(&mut reader);
        frame.application = parse_application_header(&mut reader, !frame.data_link.control.dir);

        frame.objects = parse_objects(&mut reader);

        Ok(frame)
    }
}