use std::fmt;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FunctionCode {
    Undefined(u8),
    // Control
    #[default]
    Confirm,
    Read,
    Write,
    Select,
    Operate,
    DirectOperate,
    DirectOperateNr,
    // Freeze
    Freeze,
    FreezeNr,
    FreezeClear,
    FreezeClearNr,
    // Restart
    ColdRestart,
    WarmRestart,
    // Configuration
    EnableUnsolicited,
    DisableUnsolicited,
    AssignClass,
    // Response
    Response,
    UnsolicitedResponse,
}

impl FunctionCode {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0x00 => Self::Confirm,
            0x01 => Self::Read,
            0x02 => Self::Write,
            0x03 => Self::Select,
            0x04 => Self::Operate,
            0x05 => Self::DirectOperate,
            0x06 => Self::DirectOperateNr,
            0x07 => Self::Freeze,
            0x08 => Self::FreezeNr,
            0x09 => Self::FreezeClear,
            0x0A => Self::FreezeClearNr,
            0x0D => Self::ColdRestart,
            0x0E => Self::WarmRestart,
            0x14 => Self::EnableUnsolicited,
            0x15 => Self::DisableUnsolicited,
            0x16 => Self::AssignClass,
            0x81 => Self::Response,
            0x82 => Self::UnsolicitedResponse,
            _ => Self::Undefined(v),
        }
    }
}

/// Internal indications, only present on responses.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IinFlags {
    // first octet
    pub broadcast: bool,
    pub class_1_events: bool,
    pub class_2_events: bool,
    pub class_3_events: bool,
    pub need_time: bool,
    pub local_control: bool,
    pub device_trouble: bool,
    pub device_restart: bool,
    // second octet
    pub no_func_code_support: bool,
    pub object_unknown: bool,
    pub parameter_error: bool,
    pub event_buffer_overflow: bool,
    pub already_executing: bool,
    pub config_corrupt: bool,
}

impl IinFlags {
    pub fn from_u16(value: u16) -> Self {
        let bit = |n: u8| value & (1 << n) != 0;

        Self {
            // first octet (IIN1)
            broadcast: bit(0),
            class_1_events: bit(1),
            class_2_events: bit(2),
            class_3_events: bit(3),
            need_time: bit(4),
            local_control: bit(5),
            device_trouble: bit(6),
            device_restart: bit(7),
            // second octet (IIN2)
            no_func_code_support: bit(8),
            object_unknown: bit(9),
            parameter_error: bit(10),
            event_buffer_overflow: bit(11),
            already_executing: bit(12),
            config_corrupt: bit(13),
            // bits 14 and 15 are reserved
        }
    }

    /// Names of the flags that are set (handy for logging).
    pub fn active(&self) -> Vec<&'static str> {
        [
            (self.broadcast, "broadcast"),
            (self.class_1_events, "class 1 events"),
            (self.class_2_events, "class 2 events"),
            (self.class_3_events, "class 3 events"),
            (self.need_time, "need time"),
            (self.local_control, "local control"),
            (self.device_trouble, "device trouble"),
            (self.device_restart, "device restart"),
            (self.no_func_code_support, "function code not supported"),
            (self.object_unknown, "object unknown"),
            (self.parameter_error, "parameter error"),
            (self.event_buffer_overflow, "event buffer overflow"),
            (self.already_executing, "already executing"),
            (self.config_corrupt, "config corrupt"),
        ]
            .into_iter()
            .filter(|(on, _)| *on)
            .map(|(_, name)| name)
            .collect()
    }
}

/// Control octet of the data link header.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ControlField {
    pub dir: bool,
    pub primary: bool,
    pub fcb: bool,
    pub fcv: bool,
    pub function_code: u8,
}

impl ControlField {
    pub fn from_u8(v: u8) -> Self {
        ControlField{
            dir: v & 0x80 != 0,
            primary: v & 0x40 != 0,
            fcb: v & 0x20 != 0,
            fcv: v & 0x10 != 0,
            function_code: v & 0xF
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Qualifier {
    StartStop8(u8, u8),       // 0x00
    StartStop16(u16, u16),      // 0x01
    #[default]
    AllObjects,       // 0x06
    Count8(u8),           // 0x07
    Count16(u16),          // 0x08
    CountWithIndex8(u8),  // 0x17
    CountWithIndex16(u16), // 0x28
    Other(u8),
}

pub enum PointType {
    Binary,
    Counter,
    Analog
}

/// Point quality bits. Which of these are meaningful depends on the point type.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PointFlags {
    // common
    pub online: bool,
    pub restart: bool,
    pub comm_lost: bool,
    pub remote_forced: bool,
    pub local_forced: bool,
    // binary inputs (STATE is stored as the value)
    pub chatter_filter: bool,
    pub state: bool,
    // analog inputs
    pub overrange: bool,
    pub ref_error: bool,
    // counters / frozen counters
    pub rollover: bool,
    pub discontinuity: bool,
}

impl PointFlags{
    pub fn from_u8(value: u8, point_type: &PointType) -> Self {
        let bit = |n: u8| value & (1 << n) != 0;

        // common bits 0..=4
        let mut flags = Self {
            online: bit(0),
            restart: bit(1),
            comm_lost: bit(2),
            remote_forced: bit(3),
            local_forced: bit(4),
            ..Self::default()
        };

        match point_type {
            PointType::Binary => {
                flags.chatter_filter = bit(5);
                // bit 6 reserved
                flags.state = bit(7);
            }
            PointType::Analog => {
                flags.overrange = bit(5);
                flags.ref_error = bit(6);
                // bit 7 reserved
            }
            PointType::Counter => {
                flags.rollover = bit(5);
                flags.discontinuity = bit(6);
                // bit 7 reserved
            }
        }

        flags
    }

    /// Names of the flags that are set (handy for logging).
    pub fn active(&self) -> Vec<&'static str> {
        [
            (self.online, "online"),
            (self.restart, "restart"),
            (self.comm_lost, "comm lost"),
            (self.remote_forced, "remote forced"),
            (self.local_forced, "local forced"),
            (self.chatter_filter, "chatter filter"),
            (self.state, "state"),
            (self.overrange, "overrange"),
            (self.ref_error, "reference error"),
            (self.rollover, "rollover"),
            (self.discontinuity, "discontinuity"),
        ]
            .into_iter()
            .filter(|(on, _)| *on)
            .map(|(_, name)| name)
            .collect()
    }
}

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
    Crob(Crob),
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
            Self::Crob(c) => write!(
                f,
                "CROB(code=0x{:02X}, count={}, on={}ms, off={}ms, status={})",
                c.code, c.count, c.on_time_ms, c.off_time_ms, c.status
            ),
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