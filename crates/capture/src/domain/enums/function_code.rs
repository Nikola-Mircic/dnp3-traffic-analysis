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