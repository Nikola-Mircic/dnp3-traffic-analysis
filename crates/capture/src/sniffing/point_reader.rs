use chrono::{DateTime, Utc};
use crate::domain::dnp3_packet::{Crob, Dnp3Point, PointValue};
use crate::domain::enums::{PointFlags, PointType};
use crate::error::CaptureError;
use crate::sniffing::buffer_reader::BufferReader;

// The strategy interface.
pub trait PointReader {
    fn read_point(&self, index: u32, reader: &mut BufferReader) -> Option<Dnp3Point>;
}

fn millis(ms: u64) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp_millis(ms as i64)
}

fn new_point(index: u32) -> Dnp3Point {
    let mut p = Dnp3Point::default();
    p.index = index;
    p
}

// Config + helpers shared by the flag/value/time style readers.
struct Common {
    point_type: PointType,
    include_flags: bool,
    include_time: bool,
}

impl Common {
    fn new(point_type: PointType, include_flags: bool, include_time: bool) -> Self {
        Self { point_type, include_flags, include_time }
    }

    fn read_flags(&self, reader: &mut BufferReader) -> Option<PointFlags> {
        self.include_flags
            .then(|| PointFlags::from_u8(reader.u8(), &self.point_type))
    }

    fn read_time(&self, point: &mut Dnp3Point, reader: &mut BufferReader) {
        if self.include_time {
            point.timestamp = millis(reader.u48());
        }
    }
}

/// Value is carried inside the flags byte (binary and binary output points).
struct StateReader(Common);

impl PointReader for StateReader {
    fn read_point(&self, index: u32, reader: &mut BufferReader) -> Option<Dnp3Point> {
        let mut point = new_point(index);
        point.flags = self.0.read_flags(reader);
        point.value = PointValue::Bool(point.flags.as_ref()?.state);
        self.0.read_time(&mut point, reader);
        Some(point)
    }
}

struct Int16Reader(Common);

impl PointReader for Int16Reader {
    fn read_point(&self, index: u32, reader: &mut BufferReader) -> Option<Dnp3Point> {
        let mut point = new_point(index);
        point.flags = self.0.read_flags(reader);
        point.value = PointValue::Int(reader.u16() as i64);
        self.0.read_time(&mut point, reader);
        Some(point)
    }
}

struct Int32Reader(Common);

impl PointReader for Int32Reader {
    fn read_point(&self, index: u32, reader: &mut BufferReader) -> Option<Dnp3Point> {
        let mut point = new_point(index);
        point.flags = self.0.read_flags(reader);
        point.value = PointValue::Int(reader.u32() as i64);
        self.0.read_time(&mut point, reader);
        Some(point)
    }
}

struct Float32Reader(Common);

impl PointReader for Float32Reader {
    fn read_point(&self, index: u32, reader: &mut BufferReader) -> Option<Dnp3Point> {
        let mut point = new_point(index);
        point.flags = self.0.read_flags(reader);
        point.value = PointValue::Float(f32::from_bits(reader.u32()));
        self.0.read_time(&mut point, reader);
        Some(point)
    }
}

/// Control Relay Output Block (g12v1).
struct CrobReader;

impl PointReader for CrobReader {
    fn read_point(&self, index: u32, reader: &mut BufferReader) -> Option<Dnp3Point> {
        let mut point = new_point(index);
        // Struct fields are evaluated in written order, so this reads
        // code, count, on_time, off_time, status as on the wire.
        point.value = PointValue::Crob(Crob {
            code: reader.u8(),
            count: reader.u8(),
            on_time_ms: reader.u32(),
            off_time_ms: reader.u32(),
            status: reader.u8(),
        });
        Some(point)
    }
}

/// Device attributes (g0v248/250/252/254).
struct AttributeReader;

impl PointReader for AttributeReader {
    fn read_point(&self, index: u32, reader: &mut BufferReader) -> Option<Dnp3Point> {
        let mut point = new_point(index);
        point.value = read_attribute(reader);
        Some(point)
    }
}

fn state(t: PointType, time: bool) -> Box<dyn PointReader> {
    Box::new(StateReader(Common::new(t, true, time)))
}
fn int16(t: PointType, flags: bool, time: bool) -> Box<dyn PointReader> {
    Box::new(Int16Reader(Common::new(t, flags, time)))
}
fn int32(t: PointType, flags: bool, time: bool) -> Box<dyn PointReader> {
    Box::new(Int32Reader(Common::new(t, flags, time)))
}
fn float32(t: PointType, flags: bool, time: bool) -> Box<dyn PointReader> {
    Box::new(Float32Reader(Common::new(t, flags, time)))
}

pub fn reader_for(group: u8, var: u8) -> Result<Box<dyn PointReader>, CaptureError> {
    use PointType::{Analog, Binary, BinaryOutput, Counter};

    Ok(match (group, var) {
        // Binary input (static + events)
        (1, 2) | (2, 1) => state(Binary, false),
        (2, 2)          => state(Binary, true),

        // Binary output (static + events)
        (10, 2) | (11, 1) => state(BinaryOutput, false),
        (11, 2)           => state(BinaryOutput, true),

        // Binary output command (CROB)
        (12, 1) => Box::new(CrobReader),

        // Counters (static, frozen, events)
        (20, 1) | (21, 1) | (22, 1) | (23, 1) => int32(Counter, true,  false),
        (20, 2) | (21, 2) | (22, 2) | (23, 2) => int16(Counter, true,  false),
        (20, 5)                               => int32(Counter, false, false),
        (20, 6)                               => int16(Counter, false, false),
        (21, 5) | (23, 5)                     => int32(Counter, true,  true),
        (21, 6) | (23, 6)                     => int16(Counter, true,  true),

        // Analog input static + events without time
        (30, 1) | (32, 1) => int32(Analog, true, false),
        (30, 2)           => int16(Analog, true, false),
        (30, 5)           => float32(Analog, true, false),

        // Analog input events with time
        (32, 3) => int32(Analog, true, true),
        (32, 4) => int16(Analog, true, true),
        (32, 7) => float32(Analog, true, true),

        // Device attributes
        (0, 248) | (0, 250) | (0, 252) | (0,254) => Box::new(AttributeReader),

        _ => return Err(CaptureError::FormatError(
            format!("Unsupported group and variation: {group}.{var}"),
        )),
    })
}

fn read_attribute(reader: &mut BufferReader) -> PointValue {
    let type_code = reader.u8();
    let len = reader.u8() as usize;

    decode_attribute(type_code, reader.read(len).to_vec())
}

fn decode_attribute(type_code: u8, data: Vec<u8>) -> PointValue {
    const VSTR: u8 = 1;
    const UINT: u8 = 2;
    const INT: u8 = 3;
    const FLT: u8 = 4;

    match type_code {
        VSTR => PointValue::String(
            String::from_utf8_lossy(&data).trim_end_matches('\0').to_string(),
        ),
        UINT if (1..=8).contains(&data.len()) => {
            let mut buf = [0u8; 8];
            buf[..data.len()].copy_from_slice(&data);
            PointValue::UInt(u64::from_le_bytes(buf))
        }
        INT if (1..=8).contains(&data.len()) => {
            let fill = if data[data.len() - 1] & 0x80 != 0 { 0xFF } else { 0x00 };
            let mut buf = [fill; 8];
            buf[..data.len()].copy_from_slice(&data);
            PointValue::Int(i64::from_le_bytes(buf))
        }
        FLT if data.len() == 4 => {
            PointValue::Float(f32::from_le_bytes([data[0], data[1], data[2], data[3]]))
        }
        _ => PointValue::Bytes(data), // OSTR, BSTR, TIME, unknown types
    }
}
