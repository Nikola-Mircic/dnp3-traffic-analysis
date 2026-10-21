use channel::dnp3::app::attr::AnyAttribute;
use channel::dnp3::app::measurement::*;
use channel::dnp3::master::{HeaderInfo, ReadHandler};

/// Prints every received point, like the Python SOEHandler.
pub struct SoeHandler;

impl SoeHandler {
    pub fn boxed() -> Box<dyn ReadHandler> {
        Box::new(Self)
    }
}

fn print_points<T>(
    info: &HeaderInfo,
    iter: &mut dyn Iterator<Item = (T, u16)>,
    value: impl Fn(&T) -> String,
) {
    let kind = if info.is_event { "EVENT" } else { "STATIC" };
    for (point, index) in iter {
        println!("[{}] {:?} -> {} : {}", kind, info.variation, index, value(&point));
    }
}


impl ReadHandler for SoeHandler {
    fn handle_binary_input(
        &mut self,
        info: HeaderInfo,
        iter: &mut dyn Iterator<Item = (BinaryInput, u16)>,
    ) {
        print_points(&info, iter, |p| p.value.to_string());
    }

    fn handle_binary_output_status(
        &mut self,
        info: HeaderInfo,
        iter: &mut dyn Iterator<Item = (BinaryOutputStatus, u16)>,
    ) {
        print_points(&info, iter, |p| p.value.to_string());
    }

    fn handle_counter(
        &mut self,
        info: HeaderInfo,
        iter: &mut dyn Iterator<Item = (Counter, u16)>,
    ) {
        print_points(&info, iter, |p| p.value.to_string());
    }

    fn handle_frozen_counter(
        &mut self,
        info: HeaderInfo,
        iter: &mut dyn Iterator<Item = (FrozenCounter, u16)>,
    ) {
        print_points(&info, iter, |p| p.value.to_string());
    }

    fn handle_analog_input(
        &mut self,
        info: HeaderInfo,
        iter: &mut dyn Iterator<Item = (AnalogInput, u16)>,
    ) {
        print_points(&info, iter, |p| p.value.to_string());
    }

    fn handle_analog_output_status(
        &mut self,
        info: HeaderInfo,
        iter: &mut dyn Iterator<Item = (AnalogOutputStatus, u16)>,
    ) {
        print_points(&info, iter, |p| p.value.to_string());
    }

    fn handle_device_attribute(&mut self, info: HeaderInfo, attr: AnyAttribute) {
        println!("Device attribute: {:?}", attr);
    }
}