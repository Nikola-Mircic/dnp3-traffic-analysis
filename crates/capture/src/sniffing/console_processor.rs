use crate::domain::dnp3_packet::Dnp3Frame;
use crate::domain::packet_processor::PacketProcessor;

pub struct ConsoleProcessor;

impl PacketProcessor for ConsoleProcessor {
    fn process_packet(&self, frame: &Dnp3Frame) -> () {
        let sender = match frame.data_link.control.dir {
            true => "MASTER",
            false => "OUTSTATION"
        };

        println!("- {:?} from {:?}/{:?}", frame.application.function_code, sender, frame.data_link.source);
        for object in &frame.objects {
            println!(" - Group and variation: {:?}.{:?}", object.group, object.variation);
            for point in &object.points {
                print!("\t - {:?}: {:?}", point.index, point.value);
                if point.flags.is_some() {
                    print!(" {:?}", point.flags.as_ref().unwrap().active());
                }
                if point.timestamp.is_some() {
                    print!(" {:?}", point.timestamp.as_ref().unwrap());
                }
                println!();
            }
        }
    }
}