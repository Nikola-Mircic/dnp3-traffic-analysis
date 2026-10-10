use chrono::{DateTime, Utc};
use crate::domain::dnp3_packet::{Dnp3Point, PointFlags, PointType, PointValue};
use crate::error::CaptureError;
use crate::sniffing::buffer_reader::BufferReader;
pub enum ValueType{
    State,
    Int16,
    Int32,
    Float32,
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
            (10, 2) => Ok(PointLayout::new(ValueType::State, PointType::Binary, true, false)),
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
        ValueType::Int16 => PointValue::Int(reader.u16() as i32),
        ValueType::Int32 => PointValue::Int(reader.u32() as i32),
        ValueType::Float32 => PointValue::Float(f32::from_bits(reader.u32())),
    };

    if point_info.include_time {
        point.timestamp = millis(reader.u48());
    }

    Some(point)
}
