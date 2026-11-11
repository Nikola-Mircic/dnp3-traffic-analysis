use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(multicall = true, allow_negative_numbers = true)]
pub struct Line {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    /// Set a point value. An event is produced if the value changed (always with --force).
    /// bi/bo take true|false|1|0|on|off, counters an unsigned integer, ai/ao a float.
    #[command(allow_negative_numbers = true)]
    Set {
        ty: PointType,
        index: u16,
        value: String,
        #[arg(long)]
        force: bool,
    },
    /// Flip the binary input (or, if there is none, the binary output status) at INDEX
    Toggle { index: u16 },
    /// Set or clear an application IIN flag
    Iin { action: IinAction, flag: IinFlag },
    /// The next select/operate returns STATUS instead of running
    FailNext { status: Status },
    Show { what: ShowWhat },
    Exit,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum PointType {
    Bi,
    Ai,
    Counter,
    Frozen,
    Bo,
    Ao,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum IinAction {
    Set,
    Clear,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum IinFlag {
    NeedTime,
    DeviceTrouble,
    LocalControl,
    ConfigCorrupt,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum ShowWhat {
    Points,
    Iin,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Status {
    Timeout,
    NoSelect,
    FormatError,
    NotSupported,
    AlreadyActive,
    HardwareError,
    Local,
    TooManyOps,
    NotAuthorized,
    AutomationInhibit,
    ProcessingLimited,
    OutOfRange,
    DownstreamLocal,
    AlreadyComplete,
    Blocked,
    Canceled,
    BlockedOtherMaster,
    DownstreamFail,
    NonParticipating,
}