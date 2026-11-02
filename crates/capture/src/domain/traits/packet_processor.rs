use crate::domain::models::dnp3_packet::Dnp3Frame;

pub trait PacketProcessor {
    fn process_packet(&self, pkt: &Dnp3Frame) -> ();
}