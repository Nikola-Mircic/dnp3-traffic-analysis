use crate::domain::enums::{PointFlags, PointType};
use crate::domain::models::dnp3_packet::{Crob, Dnp3Point, PointValue};
use crate::error::CaptureError;
use crate::sniffing::buffer_reader::BufferReader;
use chrono::{DateTime, Utc};

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

/// How (and whether) a time field trails the point.
#[derive(Clone, Copy)]
enum TimeKind {
    NoTime,
    /// 48-bit milliseconds since the epoch.
    Absolute,
    /// 16-bit millisecond offset from the last CTO object (g51).
    Relative,
}

#[derive(Clone, Copy)]
enum Width {
    W16,
    W32,
}

// Config + helpers shared by the flag/value/time style readers.
struct Common {
    point_type: PointType,
    include_flags: bool,
    time: TimeKind,
}

impl Common {
    fn new(point_type: PointType, include_flags: bool, time: TimeKind) -> Self {
        Self { point_type, include_flags, time }
    }

    fn read_flags(&self, reader: &mut BufferReader) -> Option<PointFlags> {
        self.include_flags
            .then(|| PointFlags::from_u8(reader.u8(), &self.point_type))
    }

    fn read_time(&self, point: &mut Dnp3Point, reader: &mut BufferReader) {
        match self.time {
            TimeKind::NoTime => {}
            TimeKind::Absolute => point.timestamp = millis(reader.u48()),
            // A relative time is an offset from the most recent g51 (CTO) object.
            TimeKind::Relative => {
                reader.u16();
            }
        }
    }
}

/// Header-only objects: "any variation" requests (vNN.0) and class objects (g60).
/// They carry no point data, so nothing is consumed from the buffer.
struct NoDataReader;

impl PointReader for NoDataReader {
    fn read_point(&self, index: u32, _reader: &mut BufferReader) -> Option<Dnp3Point> {
        Some(new_point(index)) // value stays PointValue::None
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

/// Double-bit binary (g3v2, g4v1-3). Bits 0-5 are the usual quality bits,
/// bits 6-7 are a 2-bit state: 0 = intermediate, 1 = off, 2 = on, 3 = indeterminate.
struct DoubleBitReader(Common);

impl PointReader for DoubleBitReader {
    fn read_point(&self, index: u32, reader: &mut BufferReader) -> Option<Dnp3Point> {
        let mut point = new_point(index);
        let raw = reader.u8();
        let mut flags = PointFlags::from_u8(raw, &self.0.point_type);
        flags.state = false; // bit 7 is half of the 2-bit state here, not a boolean
        point.flags = Some(flags);
        point.value = PointValue::UInt(((raw >> 6) & 0x03) as u64);
        self.0.read_time(&mut point, reader);
        Some(point)
    }
}

/// Integer points: optional flags, a 16/32-bit value (signed or not), optional time.
struct IntReader {
    common: Common,
    width: Width,
    signed: bool,
}

impl PointReader for IntReader {
    fn read_point(&self, index: u32, reader: &mut BufferReader) -> Option<Dnp3Point> {
        let mut point = new_point(index);
        point.flags = self.common.read_flags(reader);
        point.value = match (self.width, self.signed) {
            (Width::W16, false) => PointValue::UInt(reader.u16() as u64),
            (Width::W16, true) => PointValue::Int(reader.u16() as i16 as i64),
            (Width::W32, false) => PointValue::UInt(reader.u32() as u64),
            (Width::W32, true) => PointValue::Int(reader.u32() as i32 as i64),
        };
        self.common.read_time(&mut point, reader);
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

/// Binary output command event (g13v1/v2): one byte, bit 7 = commanded state,
/// bits 0-6 = command status code. Optionally followed by an absolute time.
/// The status code goes in `value`, the commanded state in `flags.state`.
struct CommandEventReader(Common);

impl PointReader for CommandEventReader {
    fn read_point(&self, index: u32, reader: &mut BufferReader) -> Option<Dnp3Point> {
        let mut point = new_point(index);
        let raw = reader.u8();
        point.flags = Some(PointFlags { state: raw & 0x80 != 0, ..PointFlags::default() });
        point.value = PointValue::UInt((raw & 0x7F) as u64);
        self.0.read_time(&mut point, reader);
        Some(point)
    }
}

/// Analog output command (g41v1/v2): setpoint followed by a control status byte.
/// The status byte is consumed but not stored (Dnp3Point has nowhere to put it).
struct AnalogOutputReader(Width);

impl PointReader for AnalogOutputReader {
    fn read_point(&self, index: u32, reader: &mut BufferReader) -> Option<Dnp3Point> {
        let mut point = new_point(index);
        point.value = match self.0 {
            Width::W16 => PointValue::Int(reader.u16() as i16 as i64),
            Width::W32 => PointValue::Int(reader.u32() as i32 as i64),
        };
        let _status = reader.u8();
        Some(point)
    }
}

/// Absolute time (g50v1) and common-time-of-occurrence (g51v1/v2): 48-bit ms.
struct TimeReader;

impl PointReader for TimeReader {
    fn read_point(&self, index: u32, reader: &mut BufferReader) -> Option<Dnp3Point> {
        let mut point = new_point(index);
        point.timestamp = millis(reader.u48());
        Some(point)
    }
}

/// Time delay (g52v1 coarse = seconds, g52v2 fine = milliseconds): u16.
struct TimeDelayReader;

impl PointReader for TimeDelayReader {
    fn read_point(&self, index: u32, reader: &mut BufferReader) -> Option<Dnp3Point> {
        let mut point = new_point(index);
        point.value = PointValue::UInt(reader.u16() as u64);
        Some(point)
    }
}

/// Device attributes (g0v209..g0v255).
struct AttributeReader;

impl PointReader for AttributeReader {
    fn read_point(&self, index: u32, reader: &mut BufferReader) -> Option<Dnp3Point> {
        let mut point = new_point(index);
        point.value = read_attribute(reader);
        Some(point)
    }
}

// ---- constructors -------------------------------------------------------

fn state(t: PointType, time: TimeKind) -> Box<dyn PointReader> {
    Box::new(StateReader(Common::new(t, true, time)))
}
fn double_bit(time: TimeKind) -> Box<dyn PointReader> {
    Box::new(DoubleBitReader(Common::new(PointType::Binary, true, time)))
}
fn command_event(time: TimeKind) -> Box<dyn PointReader> {
    Box::new(CommandEventReader(Common::new(PointType::BinaryOutput, false, time)))
}
fn int(t: PointType, width: Width, signed: bool, flags: bool, time: TimeKind) -> Box<dyn PointReader> {
    Box::new(IntReader { common: Common::new(t, flags, time), width, signed })
}
fn float32(t: PointType, flags: bool, time: TimeKind) -> Box<dyn PointReader> {
    Box::new(Float32Reader(Common::new(t, flags, time)))
}

pub fn reader_for(group: u8, var: u8) -> Result<Box<dyn PointReader>, CaptureError> {
    use PointType::{Analog, Binary, BinaryOutput, Counter};
    use TimeKind::{Absolute, NoTime, Relative};
    use Width::{W16, W32};

    Ok(match (group, var) {
        // "Any variation" requests and class objects: header only, no point data
        (1 | 2 | 3 | 4 | 10 | 11 | 12 | 13 | 20 | 21 | 22 | 23 | 30 | 32 | 34 | 40, 0)
        | (60, 1..=4) => Box::new(NoDataReader),

        // Binary input (static + events). Packed g1v1 not supported yet.
        (1, 2) | (2, 1) => state(Binary, NoTime),
        (2, 2)          => state(Binary, Absolute),
        (2, 3)          => state(Binary, Relative),

        // Double-bit binary input (static + events). Packed g3v1 not supported yet.
        (3, 2) | (4, 1) => double_bit(NoTime),
        (4, 2)          => double_bit(Absolute),
        (4, 3)          => double_bit(Relative),

        // Binary output (static + events)
        (10, 2) | (11, 1) => state(BinaryOutput, NoTime),
        (11, 2)           => state(BinaryOutput, Absolute),

        // Binary output command (CROB)
        (12, 1) => Box::new(CrobReader),

        // Binary output command events
        (13, 1) => command_event(NoTime),
        (13, 2) => command_event(Absolute),

        // Counters (static, frozen, events)
        (20, 1) | (21, 1) | (22, 1) | (23, 1) => int(Counter, W32, false, true,  NoTime),
        (20, 2) | (21, 2) | (22, 2) | (23, 2) => int(Counter, W16, false, true,  NoTime),
        (20, 5) | (21, 9)                     => int(Counter, W32, false, false, NoTime),
        (20, 6) | (21, 10)                    => int(Counter, W16, false, false, NoTime),
        (21, 5) | (23, 5)                     => int(Counter, W32, false, true,  Absolute),
        (21, 6) | (23, 6)                     => int(Counter, W16, false, true,  Absolute),

        // Analog input static + events without time
        (30, 1) | (32, 1) => int(Analog, W32, true, true,  NoTime),
        (30, 2) | (32, 2) => int(Analog, W16, true, true,  NoTime),
        (30, 3)           => int(Analog, W32, true, false, NoTime),
        (30, 4)           => int(Analog, W16, true, false, NoTime),
        (30, 5) | (32, 5) => float32(Analog, true, NoTime),

        // Analog input events with time
        (32, 3) => int(Analog, W32, true, true, Absolute),
        (32, 4) => int(Analog, W16, true, true, Absolute),
        (32, 7) => float32(Analog, true, Absolute),

        // Analog input deadband (unsigned, no flags)
        (34, 1) => int(Analog, W16, false, false, NoTime),
        (34, 2) => int(Analog, W32, false, false, NoTime),

        // Analog output status (with flags)
        (40, 1) => int(Analog, W32, true, true, NoTime),
        (40, 2) => int(Analog, W16, true, true, NoTime),

        // Analog output command (setpoint + control status)
        (41, 1) => Box::new(AnalogOutputReader(W32)),
        (41, 2) => Box::new(AnalogOutputReader(W16)),

        // Time and date, CTO, time delay
        (50, 1) | (51, 1) | (51, 2) => Box::new(TimeReader),
        (52, 1) | (52, 2)           => Box::new(TimeDelayReader),

        // Device attributes
        (0, 209..=243) | (0, 245..=250) | (0, 252 | 254 | 255) => Box::new(AttributeReader),

        _ => return Err(CaptureError::FormatError(
            format!("Unsupported group and variation: {group}.{var}"),
        )),
    })
}

// ---- attributes ----------------------------------------------------------

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