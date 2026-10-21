use channel::dnp3::app::attr::{AttrProp, StringAttr};
use channel::dnp3::link::EndpointAddress;
use channel::dnp3::outstation::database::*;
use channel::dnp3::outstation::{Feature, OutstationApplication, OutstationConfig, OutstationHandle};

/// Result of `OutstationConfigBuilder::build()`.
pub struct OutstationSetup {
    /// Pass to `ServerChannel::add_outstation`.
    pub config: OutstationConfig,
    pub name_and_model: String,
    pub manufacturer: String,
    pub serial: String,
    points: PointLayout,
}

impl OutstationSetup {
    /// Creates all configured points in the outstation database.
    /// Call once, right after `add_outstation` returns the handle.
    pub fn apply(&self, handle: &OutstationHandle) {
        let p = &self.points;
        handle.transaction(|db| {
            for (i, (class, cfg)) in p.binary_inputs.iter().enumerate() {
                db.add(i as u16, *class, *cfg);
            }
            for (i, (class, cfg)) in p.analog_inputs.iter().enumerate() {
                db.add(i as u16, *class, *cfg);
            }
            for (i, (class, cfg)) in p.counters.iter().enumerate() {
                db.add(i as u16, *class, *cfg);
            }
            for (i, (class, cfg)) in p.frozen_counters.iter().enumerate() {
                db.add(i as u16, *class, *cfg);
            }
            for (i, (class, cfg)) in p.binary_outputs.iter().enumerate() {
                db.add(i as u16, *class, *cfg);
            }
            for (i, (class, cfg)) in p.analog_outputs.iter().enumerate() {
                db.add(i as u16, *class, *cfg);
            }

            db.define_attr(
                AttrProp::default(),
                StringAttr::DeviceManufacturersName.with_value(self.manufacturer.as_str())
            ).expect("Failed to define device manufacturers name");

            db.define_attr(
                AttrProp::default(),
                StringAttr::ProductNameAndModel.with_value(self.name_and_model.as_str())
            ).expect("Failed to define device product name and model");

            db.define_attr(
                AttrProp::default(),
                StringAttr::DeviceSerialNumber.with_value(self.serial.as_str())
            ).expect("Failed to define device serial number");
        });
    }
}

#[derive(Default)]
struct PointLayout {
    binary_inputs: Vec<(Option<EventClass>, BinaryInputConfig)>,
    analog_inputs: Vec<(Option<EventClass>, AnalogInputConfig)>,
    counters: Vec<(Option<EventClass>, CounterConfig)>,
    frozen_counters: Vec<(Option<EventClass>, FrozenCounterConfig)>,
    binary_outputs: Vec<(Option<EventClass>, BinaryOutputStatusConfig)>,
    analog_outputs: Vec<(Option<EventClass>, AnalogOutputStatusConfig)>,
}

pub struct OutstationConfigBuilder {
    local_addr: u16,  // this outstation
    remote_addr: u16, // the master
    event_buffer_size: u16,
    allow_unsolicited: bool,
    name_and_model: String,
    manufacturer: String,
    serial: String,
    points: PointLayout,
}

impl Default for OutstationConfigBuilder {
    fn default() -> Self {
        Self {
            local_addr: 2,
            remote_addr: 1,
            event_buffer_size: 10,
            allow_unsolicited: false,
            points: PointLayout::default(),
            name_and_model: String::from("Outstation simulator"),
            manufacturer: String::from("Nikola Mircic"),
            serial: String::from("1652003NM")
        }
    }
}

impl OutstationConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_local_addr(&mut self, addr: u16) -> &mut Self {
        self.local_addr = addr;
        self
    }

    pub fn set_remote_addr(&mut self, addr: u16) -> &mut Self {
        self.remote_addr = addr;
        self
    }

    /// Same buffer size for all event types.
    pub fn set_event_buffer_size(&mut self, size: u16) -> &mut Self {
        self.event_buffer_size = size;
        self
    }

    pub fn allow_unsolicited(&mut self) -> &mut Self {
        self.allow_unsolicited = true;
        self
    }

    pub fn disable_unsolicited(&mut self) -> &mut Self {
        self.allow_unsolicited = false;
        self
    }

    /// Group 1 var 2 static, group 2 var 2 event (same defaults as the Python version).
    pub fn add_binary_inputs(&mut self, count: usize, class: Option<EventClass>) -> &mut Self {
        let mut cfg = BinaryInputConfig::default();
        cfg.s_var = StaticBinaryInputVariation::Group1Var2;
        cfg.e_var = EventBinaryInputVariation::Group2Var2;
        self.points.binary_inputs.extend((0..count).map(|_| (class, cfg)));
        self
    }

    /// Group 30 var 5 static (f32 + flags), group 32 var 7 event (f32 + flags + time).
    pub fn add_analog_inputs(&mut self, count: usize, class: Option<EventClass>) -> &mut Self {
        let mut cfg = AnalogInputConfig::default();
        cfg.s_var = StaticAnalogInputVariation::Group30Var5;
        cfg.e_var = EventAnalogInputVariation::Group32Var7;
        self.points.analog_inputs.extend((0..count).map(|_| (class, cfg)));
        self
    }

    /// Group 20 var 1 static, group 22 var 1 event.
    pub fn add_counters(&mut self, count: usize, class: Option<EventClass>) -> &mut Self {
        let mut cfg = CounterConfig::default();
        cfg.s_var = StaticCounterVariation::Group20Var1;
        cfg.e_var = EventCounterVariation::Group22Var1;
        self.points.counters.extend((0..count).map(|_| (class, cfg)));
        self
    }

    /// Group 21 var 5 static, group 23 var 5 event.
    pub fn add_frozen_counters(&mut self, count: usize, class: Option<EventClass>) -> &mut Self {
        let mut cfg = FrozenCounterConfig::default();
        cfg.s_var = StaticFrozenCounterVariation::Group21Var5;
        cfg.e_var = EventFrozenCounterVariation::Group23Var5;
        self.points.frozen_counters.extend((0..count).map(|_| (class, cfg)));
        self
    }

    /// Group 10 var 2 static, group 11 var 2 event.
    pub fn add_binary_outputs(&mut self, count: usize, class: Option<EventClass>) -> &mut Self {
        let mut cfg = BinaryOutputStatusConfig::default();
        cfg.s_var = StaticBinaryOutputStatusVariation::Group10Var2;
        cfg.e_var = EventBinaryOutputStatusVariation::Group11Var2;
        self.points.binary_outputs.extend((0..count).map(|_| (class, cfg)));
        self
    }

    /// Group 40 var 1 static, group 42 var 1 event.
    pub fn add_analog_outputs(&mut self, count: usize, class: Option<EventClass>) -> &mut Self {
        let mut cfg = AnalogOutputStatusConfig::default();
        cfg.s_var = StaticAnalogOutputStatusVariation::Group40Var1;
        cfg.e_var = EventAnalogOutputStatusVariation::Group42Var1;
        self.points.analog_outputs.extend((0..count).map(|_| (class, cfg)));
        self
    }

    pub fn set_name_and_model(&mut self, name: &str) -> &mut Self {
        self.name_and_model = name.to_string();
        self
    }

    pub fn set_manufacturer(&mut self, manufacturer: &str) -> &mut Self {
        self.manufacturer = manufacturer.to_string();
        self
    }

    pub fn set_serial(&mut self, serial: &str) -> &mut Self {
        self.serial = serial.to_string();
        self
    }

    pub fn build(self) -> Result<OutstationSetup, String> {
        let local = EndpointAddress::try_new(self.local_addr).map_err(|e| e.to_string())?;
        let remote = EndpointAddress::try_new(self.remote_addr).map_err(|e| e.to_string())?;

        let mut config = OutstationConfig::new(
            local,
            remote,
            EventBufferConfig::all_types(self.event_buffer_size),
        );
        config.features.unsolicited = if self.allow_unsolicited {
            Feature::Enabled
        } else {
            Feature::Disabled
        };

        Ok(OutstationSetup {
            config,
            name_and_model: self.name_and_model,
            manufacturer: self.manufacturer,
            serial: self.serial,
            points: self.points,
        })
    }
}