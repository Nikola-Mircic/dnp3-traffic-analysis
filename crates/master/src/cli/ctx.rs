use std::{collections::HashMap, time::Duration};

use clap::Parser;
use channel::dnp3::app::attr::{AllAttributes, AttrSet};
use channel::dnp3::app::control::{CommandStatus, ControlCode, Group12Var1, OpType};
use channel::dnp3::master::{AssociationHandle, Classes, CommandBuilder, CommandMode, CommandSupport, EventClasses, PollHandle, ReadRequest};
use crate::cli::cmd::{Cmd, Line, Op, PollCmd};
use crate::cli::logger::Logger;

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
            // read, analog-out, unsol, time-sync, delay-measure, restarts,
            // freeze, link-status: see the mapping table in my previous message
            _ => anyhow::bail!("not implemented yet"),
        }
        Ok(())
    }
}