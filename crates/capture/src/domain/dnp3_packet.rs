use std::fmt;

use chrono::{DateTime, Utc};
use crate::domain::enums::*;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLinkHeader {
    pub source: u16,
    pub destination: u16,
    pub frame_length: u8,
    pub control: ControlField,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub struct TransportHeader {
    pub fir: bool,
    pub fin: bool,
    pub sequence: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Crob {
    pub code: u8,
    pub count: u8,
    pub on_time_ms: u32,
    pub off_time_ms: u32,
    pub status: u8,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum PointValue {
    #[default]
    None, // object carries no data (e.g. a read request)
    Bool(bool),
    Int(i32),
    Float(f32),
    Crob(Crob)
}

pub enum OpType {
    ///  not specified (value == 0)
    Nul,
    ///  pulse the output on (value == 1)
    PulseOn,
    ///  pulse the output off (value == 2)
    PulseOff,
    ///  latch the output on (value == 3)
    LatchOn,
    ///  latch the output off (value == 4)
    LatchOff,
    /// captures any value not defined in the enumeration
    Unknown(u8),
}

impl OpType {
    /// create the enum from the underlying value
    pub fn from(x: u8) -> Self {
        match x {
            0 => OpType::Nul,
            1 => OpType::PulseOn,
            2 => OpType::PulseOff,
            3 => OpType::LatchOn,
            4 => OpType::LatchOff,
            _ => OpType::Unknown(x),
        }
    }

    /// convert the enum to its underlying value
    pub fn as_str(&self) -> &str {
        match self {
            OpType::Nul => "Null",
            OpType::PulseOn => "PulseOn",
            OpType::PulseOff => "PulseOff",
            OpType::LatchOn => "LatchOn",
            OpType::LatchOff => "LatchOff",
            OpType::Unknown(_) => "Unknown",
        }
    }
}

impl fmt::Display for PointValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "-"),
            Self::Bool(v) => write!(f, "{v}"),
            Self::Int(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            Self::Crob(c) => write!(
                f,
                "{}, count={}, on={}ms, off={}ms, status={}",
                OpType::from(c.code).as_str(), c.count, c.on_time_ms, c.off_time_ms, c.status
            ),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Dnp3Point {
    pub index: u32,
    pub value: PointValue,
    pub flags: Option<PointFlags>,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Dnp3Object {
    pub group: u8,
    pub variation: u8,
    pub qualifier: Qualifier,
    pub points: Vec<Dnp3Point>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ApplicationHeader {
    pub fir: bool,
    pub fin: bool,
    pub confirm_requested: bool,
    pub unsolicited: bool,
    pub sequence: u8,
    pub function_code: FunctionCode,
    pub iin: Option<IinFlags>, // only on responses
}

/// The unit every component downstream of the adapter works with.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Dnp3Frame {
    pub captured_at: DateTime<Utc>,
    pub data_link: DataLinkHeader,
    pub transport: TransportHeader,
    pub application: ApplicationHeader,
    pub objects: Vec<Dnp3Object>,
}