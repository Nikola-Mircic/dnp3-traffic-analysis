use crate::domain::dnp3_packet::Dnp3Frame;

pub trait PacketProcessor {
    fn process_packet(&self, pkt: &Dnp3Frame) -> ();
}