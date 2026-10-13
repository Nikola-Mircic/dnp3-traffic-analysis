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