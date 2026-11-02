use crate::domain::enums::*;
use crate::domain::models::dnp3_packet::*;
use crate::domain::models::packet_adapter::PacketAdapter;
use crate::error::CaptureError;
use crate::sniffing::buffer_reader::BufferReader;
use crate::sniffing::point_reader::reader_for;
use crate::util::crc::strip_crcs;

const DNP3_START_BYTES: [u8;2] = [0x05, 0x64];

pub struct PcapPacketAdapter {}

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

    if parse_iin {
        header.iin = Some(IinFlags::from_u16(reader.u16()));
    } else {
        header.iin = None;
    }

    header
}

fn parse_points(reader: &mut BufferReader, object: &Dnp3Object) -> Result<Vec<Dnp3Point>, CaptureError> {
    let mut points = Vec::new();

    let point_reader = match reader_for(object.group, object.variation) {
        Ok(pr) => pr,
        Err(e) => return Err(e),
    };

    match object.qualifier {
        Qualifier::StartStop8(start, stop) => {
            let (start_idx, stop_idx) = (start as u32, stop as u32);
            for i in start_idx..stop_idx+1 {
                let point = point_reader.read_point(i, reader);
                points.push(point.unwrap());
            }
        },
        Qualifier::StartStop16(start, stop) => {
            let (start_idx, stop_idx) = (start as u32, stop as u32);
            for i in start_idx..stop_idx+1 {
                let point = point_reader.read_point(i, reader);
                points.push(point.unwrap());
            }
        },
        Qualifier::Count8(count) => {
            let count_32 = count as u32;
            for i in 0..count_32 {
                let point = point_reader.read_point(i, reader);
                points.push(point.unwrap());
            }
        },
        Qualifier::Count16(count) => {
            let count_32 = count as u32;
            for i in 0..count_32 {
                let point = point_reader.read_point(i, reader);
                points.push(point.unwrap());
            }
        },
        Qualifier::CountWithIndex8(count) => {
            for i in 0..count {
                let index = reader.u8() as u32;
                let point = point_reader.read_point(index, reader);
                points.push(point.unwrap());
            }
        },
        Qualifier::CountWithIndex16(count) => {
            for i in 0..count {
                let index = reader.u16() as u32;
                let point = point_reader.read_point(index, reader);
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

        object.qualifier = Qualifier::from_reader(reader);

        if object.qualifier == Qualifier::AllObjects {
            objects.push(object);
            continue;
        }

        if object.group == 0 && object.variation == 254 {
            objects.push(object);
            continue;
        }

        let points = parse_points(reader, &object);

        if points.is_err() {
            object.points = Vec::new();
            println!("{}", points.err().unwrap());
            break;
        }

        object.points = points.unwrap();
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