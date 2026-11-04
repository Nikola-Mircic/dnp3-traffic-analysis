use std::{collections::HashMap, time::Duration};

use anyhow::{anyhow, bail};
use clap::Parser;
use channel::dnp3::app::attr::{AllAttributes, AttrSet};
use channel::dnp3::app::control::{
    CommandStatus, ControlCode, Group12Var1, Group41Var1, Group41Var2, Group41Var3, Group41Var4,
    OpType,
};
use channel::dnp3::app::FunctionCode;

pub(crate) use channel::dnp3::app::Variation;

use channel::dnp3::master::{
    AssociationHandle, Classes, CommandBuilder, CommandMode, CommandSupport, EventClasses, Headers,
    PollHandle, ReadRequest, TaskError, TimeSyncProcedure, WriteError,
};
use crate::cli::cmd::{Cmd, Line, Op, PollCmd, UnsolAction};
use crate::cli::logger::Logger;
use crate::cli::variation::VariationParser;

impl From<Op> for OpType {
    fn from(op: Op) -> Self {
        match op {
            Op::LatchOn => OpType::LatchOn,
            Op::LatchOff => OpType::LatchOff,
            Op::PulseOn => OpType::PulseOn,
            Op::PulseOff => OpType::PulseOff,
        }
    }
}

pub struct Ctx {
    pub assoc: AssociationHandle,
    pub polls: HashMap<u8, (PollHandle, u64)>, // class -> (handle, interval_ms), for `poll list`
    pub logger: Logger,
}

impl Ctx {
    pub fn new(assoc: AssociationHandle, logger: Logger) -> Self {
        Self { assoc, polls: HashMap::new(), logger }
    }

    /// Returns false when the REPL should exit.
    pub async fn exec_line(&mut self, line: &str) -> bool {
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

        self.logger.log("CMD", format!("{cmd:?}")); // timestamp just BEFORE the wire
        match self.dispatch(cmd).await {
            Ok(()) => self.logger.log("DONE", "ok"),
            Err(e) => {
                println!("error: {e}");
                self.logger.log("ERR", e);
            }
        }
        true
    }

    async fn dispatch(&mut self, cmd: Cmd) -> anyhow::Result<()> {
        match cmd {
            Cmd::Integrity => {
                self.assoc.read(ReadRequest::ClassScan(Classes::all())).await?;
            }
            Cmd::Scan { class } => {
                let ev = EventClasses::new(class == 1, class == 2, class == 3);
                self.assoc.read(ReadRequest::ClassScan(Classes::new(false, ev))).await?;
            }
            Cmd::Control { op, index, sbo, count, on_ms, off_ms } => {
                let crob = Group12Var1 {
                    code: ControlCode::from_op_type(op.into()),
                    count,
                    on_time: on_ms,
                    off_time: off_ms,
                    status: CommandStatus::Success,
                };
                let mode = if sbo {
                    CommandMode::SelectBeforeOperate
                } else {
                    CommandMode::DirectOperate
                };
                self.assoc
                    .operate(mode, CommandBuilder::single_header_u16(crob, index))
                    .await?;
            }
            Cmd::Poll(PollCmd::Add { class, interval_ms }) => {
                if class > 3 {
                    bail!("poll class must be 0..=3 (got {class})");
                }
                let req = ReadRequest::ClassScan(Classes::new(
                    class == 0,
                    EventClasses::new(class == 1, class == 2, class == 3),
                ));
                let h = self.assoc.add_poll(req, Duration::from_millis(interval_ms)).await?;
                self.polls.insert(class, (h, interval_ms));
            }
            Cmd::Poll(PollCmd::Remove { class }) => {
                if let Some((h, _)) = self.polls.remove(&class) {
                    h.remove().await?;
                }
            }
            Cmd::Poll(PollCmd::List) => {
                for (class, (_, ms)) in &self.polls {
                    println!("class {class}: every {ms} ms");
                }
            }
            Cmd::ReadAttr => {
                self.assoc
                    .read(ReadRequest::device_attribute(AllAttributes, AttrSet::Default))
                    .await?;
            }
            Cmd::Read { group, var, start, stop } => {
                let v = Variation::find(group, var).unwrap();
                let req = match start {
                    None => ReadRequest::all_objects(v), // qualifier 0x06
                    Some(s) => {
                        let e = stop.unwrap_or(s); // a lone index reads that single point
                        if s > e {
                            bail!("start ({s}) must not be greater than stop ({e})");
                        }
                        if e <= u8::MAX as u16 {
                            ReadRequest::one_byte_range(v, s as u8, e as u8) // qualifier 0x00
                        } else {
                            ReadRequest::two_byte_range(v, s, e) // qualifier 0x01
                        }
                    }
                };
                self.assoc.read(req).await?;
            }
            Cmd::AnalogOut { value, index, var } => {
                let st = CommandStatus::Success;
                let headers = match var {
                    1 => CommandBuilder::single_header_u16(
                        Group41Var1 { value: to_int(value, i32::MIN as f64, i32::MAX as f64)? as i32, status: st },
                        index,
                    ),
                    2 => CommandBuilder::single_header_u16(
                        Group41Var2 { value: to_int(value, i16::MIN as f64, i16::MAX as f64)? as i16, status: st },
                        index,
                    ),
                    3 => CommandBuilder::single_header_u16(Group41Var3 { value: value as f32, status: st }, index),
                    4 => CommandBuilder::single_header_u16(Group41Var4 { value, status: st }, index),
                    _ => bail!("--var must be 1..=4"),
                };
                self.assoc.operate(CommandMode::DirectOperate, headers).await?;
            }
            Cmd::Unsol { action, classes } => {
                let classes = if classes.is_empty() { vec![1, 2, 3] } else { classes };
                let mut headers = Headers::new();
                for c in classes {
                    headers = headers.add_all_objects(match c {
                        1 => Variation::Group60Var2,
                        2 => Variation::Group60Var3,
                        3 => Variation::Group60Var4,
                        _ => bail!("unsolicited classes must be 1, 2 or 3 (got {c})"),
                    });
                }
                let function = match action {
                    UnsolAction::Enable => FunctionCode::EnableUnsolicited,
                    UnsolAction::Disable => FunctionCode::DisableUnsolicited,
                };
                self.assoc.send_and_expect_empty_response(function, headers).await?;
            }
            Cmd::TimeSync { lan } => {
                let procedure = if lan { TimeSyncProcedure::Lan } else { TimeSyncProcedure::NonLan };
                self.assoc.synchronize_time(procedure).await?;
            }
            Cmd::DelayMeasure => {
                // The library only exposes DELAY_MEASURE as part of the non-LAN time sync. Sending the
                // function code through the generic "empty response" task puts the request on the wire,
                // but the g52v2 answer isn't empty, so the library reports UnexpectedResponseHeaders.
                // That outcome means the outstation DID answer, so it's treated as success here.
                match self
                    .assoc
                    .send_and_expect_empty_response(FunctionCode::DelayMeasure, Headers::new())
                    .await
                {
                    Ok(()) => {}
                    Err(WriteError::Task(TaskError::UnexpectedResponseHeaders)) => {
                        println!("delay measure answered (g52v2 response, not decoded by the library)");
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            Cmd::ColdRestart => {
                let delay = self.assoc.cold_restart().await?;
                println!("outstation restart delay: {} ms", delay.as_millis());
            }
            Cmd::WarmRestart => {
                let delay = self.assoc.warm_restart().await?;
                println!("outstation restart delay: {} ms", delay.as_millis());
            }
            Cmd::Freeze { clear, no_ack } => {
                let function = match (clear, no_ack) {
                    (false, false) => FunctionCode::ImmediateFreeze,
                    (false, true) => FunctionCode::ImmediateFreezeNoResponse,
                    (true, false) => FunctionCode::FreezeClear,
                    (true, true) => FunctionCode::FreezeClearNoResponse,
                };
                let headers = Headers::new().add_all_objects(Variation::Group20Var0);
                match self.assoc.send_and_expect_empty_response(function, headers).await {
                    Ok(()) => {}
                    // The outstation never answers the no-response variants, so the library waits for its
                    // response timeout and reports it. The request was sent, which is all that's wanted.
                    Err(WriteError::Task(TaskError::ResponseTimeout)) if no_ack => {
                        println!("no-response freeze sent (library waited for its response timeout)");
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            Cmd::LinkStatus => {
                self.assoc.check_link_status().await?;
                println!("link status: ok");
            }
            Cmd::Exit => {} // handled in exec_line
        }
        Ok(())
    }
}

/// Reject values that aren't whole numbers or don't fit the integer variation, instead of silently
/// truncating/saturating them (a test tool should send what was typed, or refuse).
fn to_int(value: f64, min: f64, max: f64) -> anyhow::Result<f64> {
    if value.fract() != 0.0 || value < min || value > max {
        bail!("{value} is not a whole number in {min}..={max} (use --var 3 or 4 for floating point)");
    }
    Ok(value)
}