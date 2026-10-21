use chrono::{DateTime, Utc};
use crate::domain::dnp3_packet::{Crob, Dnp3Point, PointValue};
use crate::domain::enums::{PointType, PointFlags};
use crate::error::CaptureError;
use crate::sniffing::buffer_reader::BufferReader;
pub enum ValueType{
    State,
    Int16,
    Int32,
    Float32,
    Crob,
    Attribute
}

pub struct PointLayout {
    pub value_type: ValueType,
    pub point_type: PointType,
    pub include_flags: bool,
    pub include_time: bool,
}

impl PointLayout {
    fn new(value_type: ValueType, point_type: PointType, include_flags: bool, include_time: bool) -> PointLayout {
        PointLayout {value_type, point_type, include_flags, include_time}
    }

    pub fn from_gv(group: u8, var: u8) -> Result<PointLayout, CaptureError>{
        match (group, var){
            // Supported binary groups and variations
            (1, 2) => Ok(PointLayout::new(ValueType::State, PointType::Binary, true, false)),
            (2, 1) => Ok(PointLayout::new(ValueType::State, PointType::Binary, true, false)),
            (2, 2) => Ok(PointLayout::new(ValueType::State, PointType::Binary, true, true)),
            (10, 2) => Ok(PointLayout::new(ValueType::State, PointType::BinaryOutput, true, false)),
            // Supported counter static groups and variations
            (20, 1) => Ok(PointLayout::new(ValueType::Int32, PointType::Counter, true, false)),
            (20, 2) => Ok(PointLayout::new(ValueType::Int16, PointType::Counter, true, false)),
            (20, 5) => Ok(PointLayout::new(ValueType::Int32, PointType::Counter, false, false)),
            (20, 6) => Ok(PointLayout::new(ValueType::Int16, PointType::Counter, false, false)),
            // Supported frozen counter static groups and variations
            (21, 1) => Ok(PointLayout::new(ValueType::Int32, PointType::Counter, true, false)),
            (21, 2) => Ok(PointLayout::new(ValueType::Int16, PointType::Counter, true, false)),
            (21, 5) => Ok(PointLayout::new(ValueType::Int32, PointType::Counter, true, true)),
            (21, 6) => Ok(PointLayout::new(ValueType::Int16, PointType::Counter, true, true)),
            // Counter events
            (22, 1) => Ok(PointLayout::new(ValueType::Int32, PointType::Counter, true, false)),
            (22, 2) => Ok(PointLayout::new(ValueType::Int16, PointType::Counter, true, false)),
            // Frozen counter events
            (23, 1) => Ok(PointLayout::new(ValueType::Int32, PointType::Counter, true, false)),
            (23, 2) => Ok(PointLayout::new(ValueType::Int16, PointType::Counter, true, false)),
            (23, 5) => Ok(PointLayout::new(ValueType::Int32, PointType::Counter, true, true)),
            (23, 6) => Ok(PointLayout::new(ValueType::Int16, PointType::Counter, true, true)),
            // Analog input static
            (30, 1) => Ok(PointLayout::new(ValueType::Int32, PointType::Analog, true, false)),
            (30, 2) => Ok(PointLayout::new(ValueType::Int16, PointType::Analog, true, false)),
            (30, 5) => Ok(PointLayout::new(ValueType::Float32, PointType::Analog, true, false)),
            // Analog input events
            (32, 1) => Ok(PointLayout::new(ValueType::Int32, PointType::Analog, true, false)),
            (32, 3) => Ok(PointLayout::new(ValueType::Int32, PointType::Analog, true, true)),
            (32, 4) => Ok(PointLayout::new(ValueType::Int16, PointType::Analog, true, true)),
            (32, 7) => Ok(PointLayout::new(ValueType::Float32, PointType::Analog, true, true)),
            // Binary output events
            (11, 1) => Ok(PointLayout::new(ValueType::State, PointType::BinaryOutput, true, false)),
            (11, 2) => Ok(PointLayout::new(ValueType::State, PointType::BinaryOutput, true, true)),
            // Binary output command (CROB)
            (12, 1) => Ok(PointLayout::new(ValueType::Crob, PointType::BinaryOutput, false, false)),
            // Device attributes
            (0, 248) | (0, 250) | (0, 252) | (0, 254) => Ok(PointLayout::new(ValueType::Attribute, PointType::Analog, false, false)),
            _ => Err(CaptureError::FormatError(format!("Unsupported group and variation: {group}.{var}"))),
        }
    }
}

fn millis(ms: u64) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp_millis(ms as i64)
}

pub fn read_point(index: u32, point_info: &PointLayout, reader: &mut BufferReader) -> Option<Dnp3Point>{
    let mut point = Dnp3Point::default();

    point.index = index;

    if point_info.include_flags {
        point.flags = Some(PointFlags::from_u8(reader.u8(), &point_info.point_type));
    } else {
        point.flags = None
    }

    point.value = match point_info.value_type {
        ValueType::State => {
            PointValue::Bool(point.flags.clone()?.state)
        },
        ValueType::Int16 => PointValue::Int(reader.u16() as i64),
        ValueType::Int32 => PointValue::Int(reader.u32() as i64),
        ValueType::Float32 => PointValue::Float(f32::from_bits(reader.u32())),
        ValueType::Crob => PointValue::Crob(Crob {
            code: reader.u8(),
            count: reader.u8(),
            on_time_ms: reader.u32(),
            off_time_ms: reader.u32(),
            status: reader.u8(),
        }),
        ValueType::Attribute => read_attribute(reader),
    };

    if point_info.include_time {
        point.timestamp = millis(reader.u48());
    }

    Some(point)
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
