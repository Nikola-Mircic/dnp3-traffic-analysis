use std::str::FromStr;

use channel::dnp3::app::control::CommandStatus;
use channel::dnp3::app::measurement::*;
use channel::dnp3::outstation::database::*;
use clap::Parser;

use crate::application::SharedIin;
use crate::cli::cmd::{Cmd, IinAction, IinFlag, Line, PointType, ShowWhat, Status};
use crate::cli::logger::Logger;
use crate::control::FailNext;
use crate::util::now;

impl From<Status> for CommandStatus {
    fn from(s: Status) -> Self {
        match s {
            Status::Timeout => CommandStatus::Timeout,
            Status::NoSelect => CommandStatus::NoSelect,
            Status::FormatError => CommandStatus::FormatError,
            Status::NotSupported => CommandStatus::NotSupported,
            Status::AlreadyActive => CommandStatus::AlreadyActive,
            Status::HardwareError => CommandStatus::HardwareError,
            Status::Local => CommandStatus::Local,
            Status::TooManyOps => CommandStatus::TooManyOps,
            Status::NotAuthorized => CommandStatus::NotAuthorized,
            Status::AutomationInhibit => CommandStatus::AutomationInhibit,
            Status::ProcessingLimited => CommandStatus::ProcessingLimited,
            Status::OutOfRange => CommandStatus::OutOfRange,
            Status::DownstreamLocal => CommandStatus::DownstreamLocal,
            Status::AlreadyComplete => CommandStatus::AlreadyComplete,
            Status::Blocked => CommandStatus::Blocked,
            Status::Canceled => CommandStatus::Canceled,
            Status::BlockedOtherMaster => CommandStatus::BlockedOtherMaster,
            Status::DownstreamFail => CommandStatus::DownstreamFail,
            Status::NonParticipating => CommandStatus::NonParticipating,
        }
    }
}

pub struct Ctx {
    db: DatabaseHandle,
    iin: SharedIin,
    fail_next: FailNext,
    logger: Logger,
}

impl Ctx {
    pub fn new(db: DatabaseHandle, iin: SharedIin, fail_next: FailNext, logger: Logger) -> Self {
        Self { db, iin, fail_next, logger }
    }

    /// Returns false when the REPL should exit.
    pub fn exec_line(&mut self, line: &str) -> bool {
        let Some(args) = shlex::split(line) else {
            println!("bad quoting");
            return true;
        };
        if args.is_empty() {
            return true;
        }
        let cmd = match Line::try_parse_from(args) {
            Ok(l) => l.cmd,
            Err(e) => {
                let _ = e.print(); // clap prints usage / help
                return true;
            }
        };
        if matches!(cmd, Cmd::Exit) {
            return false;
        }

        self.logger.log("CMD", format!("{cmd:?}"));
        match self.dispatch(cmd) {
            Ok(()) => self.logger.log("DONE", "ok"),
            Err(e) => {
                println!("error: {e}");
                self.logger.log("ERR", e);
            }
        }
        true
    }

    fn dispatch(&mut self, cmd: Cmd) -> Result<(), String> {
        match cmd {
            Cmd::Set { ty, index, value, force } => self.set(ty, index, &value, force)?,
            Cmd::Toggle { index } => self.toggle(index)?,
            Cmd::Iin { action, flag } => {
                let on = matches!(action, IinAction::Set);
                let mut iin = self.iin.lock().unwrap();
                match flag {
                    IinFlag::NeedTime => iin.need_time = on,
                    IinFlag::DeviceTrouble => iin.device_trouble = on,
                    IinFlag::LocalControl => iin.local_control = on,
                    IinFlag::ConfigCorrupt => iin.config_corrupt = on,
                }
                println!("{flag:?} = {on}");
            }
            Cmd::FailNext { status } => {
                *self.fail_next.lock().unwrap() = Some(status.into());
                println!("the next select/operate will return {status:?}");
            }
            Cmd::Show { what } => match what {
                ShowWhat::Points => self.show_points(),
                ShowWhat::Iin => println!("{:#?}", *self.iin.lock().unwrap()),
            },
            Cmd::Exit => {} // handled in exec_line
        }
        Ok(())
    }

    fn set(&mut self, ty: PointType, index: u16, value: &str, force: bool) -> Result<(), String> {
        let opts = if force {
            UpdateOptions::new(true, EventMode::Force)
        } else {
            UpdateOptions::detect_event()
        };
        let (f, t) = (Flags::ONLINE, now());
        let ok = match ty {
            PointType::Bi => {
                let v = parse_bool(value)?;
                self.db.transaction(|db| db.update(index, &BinaryInput::new(v, f, t), opts))
            }
            PointType::Bo => {
                let v = parse_bool(value)?;
                self.db.transaction(|db| db.update(index, &BinaryOutputStatus::new(v, f, t), opts))
            }
            PointType::Ai => {
                let v: f64 = parse(value)?;
                self.db.transaction(|db| db.update(index, &AnalogInput::new(v, f, t), opts))
            }
            PointType::Ao => {
                let v: f64 = parse(value)?;
                self.db.transaction(|db| db.update(index, &AnalogOutputStatus::new(v, f, t), opts))
            }
            PointType::Counter => {
                let v: u32 = parse(value)?;
                self.db.transaction(|db| db.update(index, &Counter::new(v, f, t), opts))
            }
            PointType::Frozen => {
                let v: u32 = parse(value)?;
                self.db.transaction(|db| db.update(index, &FrozenCounter::new(v, f, t), opts))
            }
        };
        if !ok {
            return Err(format!("no {ty:?} point at index {index}"));
        }
        println!("{ty:?}[{index}] = {value}");
        Ok(())
    }

    fn toggle(&mut self, index: u16) -> Result<(), String> {
        let opts = UpdateOptions::detect_event();
        let new_value = self.db.transaction(|db| {
            let bi: Option<BinaryInput> = db.get(index);
            if let Some(c) = bi {
                db.update(index, &BinaryInput::new(!c.value, Flags::ONLINE, now()), opts);
                return Some(!c.value);
            }
            let bo: Option<BinaryOutputStatus> = db.get(index);
            if let Some(c) = bo {
                db.update(index, &BinaryOutputStatus::new(!c.value, Flags::ONLINE, now()), opts);
                return Some(!c.value);
            }
            None
        });
        match new_value {
            Some(v) => {
                println!("binary[{index}] -> {v}");
                Ok(())
            }
            None => Err(format!("no binary point at index {index}")),
        }
    }

    fn show_points(&self) {
        let lines = self.db.transaction(|db| {
            let mut out = Vec::new();
            dump::<BinaryInput>(db, "bi", &mut out);
            dump::<AnalogInput>(db, "ai", &mut out);
            dump::<Counter>(db, "counter", &mut out);
            dump::<FrozenCounter>(db, "frozen", &mut out);
            dump::<BinaryOutputStatus>(db, "bo", &mut out);
            dump::<AnalogOutputStatus>(db, "ao", &mut out);
            out
        });
        for l in lines {
            println!("{l}");
        }
    }
}

/// Points are created at indices 0..n, so reading stops at the first missing one.
fn dump<T: std::fmt::Debug>(db: &Database, name: &str, out: &mut Vec<String>)
where
    Database: Get<T>,
{
    for i in 0..=u16::MAX {
        match Get::<T>::get(db, i) {
            Some(v) => out.push(format!("{name}[{i}] {v:?}")),
            None => break,
        }
    }
}

fn parse<T: FromStr>(s: &str) -> Result<T, String> {
    s.parse().map_err(|_| format!("'{s}' is not a valid value"))
}

fn parse_bool(s: &str) -> Result<bool, String> {
    match s.to_ascii_lowercase().as_str() {
        "1" | "true" | "on" => Ok(true),
        "0" | "false" | "off" => Ok(false),
        _ => Err(format!("'{s}' is not a boolean (use true|false|1|0|on|off)")),
    }
}