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