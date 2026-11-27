use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(multicall = true, allow_negative_numbers = true)]
pub struct Line {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    Integrity,
    Scan {
        #[arg(value_parser = clap::value_parser!(u8).range(1..=3))]
        class: u8,
    },
    Read {
        group: u8,
        var: u8,
        start: Option<u16>,
        #[arg(requires = "start")]
        stop: Option<u16>,
    },
    #[command(subcommand)]
    Poll(PollCmd),
    Control {
        op: Op,
        #[arg(short = 'i', long, default_value_t = 0)]
        index: u16,
        #[arg(long)]
        sbo: bool,
        #[arg(long, default_value_t = 1)]
        count: u8,
        #[arg(long, default_value_t = 5000)]
        on_ms: u32,
        #[arg(long, default_value_t = 5000)]
        off_ms: u32,
    },
    #[command(allow_negative_numbers = true)]
    AnalogOut {
        value: f64,
        #[arg(short = 'i', long)]
        index: u16,
        #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..=4))]
        var: u8,
    },
    Unsol {
        action: UnsolAction,
        classes: Vec<u8>,
    },
    TimeSync {
        #[arg(long)]
        lan: bool,
    },
    DelayMeasure,
    ColdRestart,
    WarmRestart,
    Freeze {
        #[arg(long)]
        clear: bool,
        #[arg(long)]
        no_ack: bool,
    },
    ReadAttr,
    LinkStatus,
    Exit,
}

#[derive(Subcommand, Debug)]
pub enum PollCmd {
    Add { class: u8, interval_ms: u64 },
    Remove { class: u8 },
    List,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Op {
    LatchOn,
    LatchOff,
    PulseOn,
    PulseOff,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum UnsolAction {
    Enable,
    Disable,
}