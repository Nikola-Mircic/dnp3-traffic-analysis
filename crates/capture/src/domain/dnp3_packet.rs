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

/// Replaces Python's `value: Any`.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PointValue {
    #[default]
    None, // object carries no data (e.g. a read request)
    Bool(bool),
    Int(i32),
    UInt(u32),
    Float(f32),
    //Time(DateTime<Utc>),
    //Crob(Crob),
    AnalogCommand { value: f32, status: u8 },
}

impl fmt::Display for PointValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "-"),
            Self::Bool(v) => write!(f, "{v}"),
            Self::Int(v) => write!(f, "{v}"),
            Self::UInt(v) => write!(f, "{v}"),
            Self::Float(v) => write!(f, "{v}"),
            //Self::Time(t) => write!(f, "{}", t.format("%Y-%m-%d %H:%M:%S%.3f")),
            // Self::Crob(c) => write!(
            //     f,
            //     "CROB(code=0x{:02X}, count={}, on={}ms, off={}ms, status={})",
            //     c.code, c.count, c.on_time_ms, c.off_time_ms, c.status
            // ),
            Self::AnalogCommand { value, status } => write!(f, "{value} (status {status})"),
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