use pcap::Device;
use crate::domain::packet_adapter::PacketAdapter;
use crate::domain::packet_source::PacketSource;
use crate::error::CaptureError;
use crate::sniffing::live_packet_source::LivePacketSource;
use crate::sniffing::packet_adapter::PcapPacketAdapter;

mod domain;
mod sniffing;
pub mod error;

fn select_device() -> Device {
    let devices = Device::list().unwrap();

    println!("Please select a device");

    for index in 0..devices.len() {
        let device = devices.get(index).unwrap();
        println!("{:?}: {:?}",index, device.desc.as_ref().unwrap());
    }

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    let mut selected = input.trim().parse::<usize>().unwrap_or(0);

    if(selected >= devices.len()) {
        selected = 0;
    }

    devices.get(selected).unwrap().clone()
}

fn process_packets<T>(source: &mut T)
where T: PacketSource {
    let adapter = PcapPacketAdapter{};
    loop {
        match source.next_packet() {
            Ok(packet) => {
                let dnp3_start_idx = packet.windows(2).position(|pair| pair[0] == 0x05 && pair[1] == 0x64).unwrap_or(0);
                let dnp3_slice = &packet[dnp3_start_idx..];

                println!("Adapting frame of: {:?} bytes", dnp3_slice.len());
                let frame = adapter.adapt(dnp3_slice);
                match frame {
                    Ok(frame) => {
                        println!("Adapted frame: {:?}", frame);
                    },

                    Err(e) => {println!("Error: {:?}", e);},
                }
            }
            Err(CaptureError::PcapError(pcap::Error::TimeoutExpired)) => continue,
            Err(e) => { eprintln!("capture error: {:?}", e); break; }
        }
    }
}

fn main() {
    let device = select_device();
    let name = device.desc.clone().unwrap();

    println!("Using device: {:?}", name);

    let mut source = LivePacketSource::from_dev(&device);

    process_packets(&mut source);
}
