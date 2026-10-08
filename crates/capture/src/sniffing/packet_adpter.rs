use crate::domain::dnp3_packet::{ApplicationHeader, ControlField, DataLinkHeader, Dnp3Frame, FunctionCode, IinFlags, TransportHeader};
use crate::domain::packet_adapter::PacketAdapter;
use crate::error::CaptureError;
use crate::sniffing::buffer_reader::BufferReader;

const DNP3_START_BYTES: [u8;2] = [0x05, 0x64];

pub struct PcapPacketAdpter {}

fn get_dnp3_slice(raw_packet_data: &[u8]) -> &[u8] {
    let start_idx = raw_packet_data.windows(2).position(|w| w == DNP3_START_BYTES).unwrap_or(raw_packet_data.len());
    &raw_packet_data[start_idx..]
}

fn parse_data_link_header(reader: &mut BufferReader) -> DataLinkHeader {
    let mut header = DataLinkHeader::default();

    header.frame_length = reader.u8();
    header.control = ControlField::from_u8(reader.u8());
    header.destination = reader.u16();
    header.source = reader.u16();

    header
}

fn parse_transport_header(reader: &mut BufferReader) -> TransportHeader {
    let mut header = TransportHeader::default();

    let value = reader.u8();

    header.fin = value & 0x80 == 1;
    header.fir = value & 0x40 == 1;
    header.sequence = value & 0x3F;

    header
}

fn parse_application_header(reader: &mut BufferReader, parse_iin: bool) -> ApplicationHeader {
    let mut header = ApplicationHeader::default();

    // Control octet
    let control = reader.u8();
    header.fir = control & 0x80 == 1;
    header.fin = control & 0x40 == 1;
    header.confirm_requested = control & 0x20 == 1;
    header.unsolicited = control & 0x10 == 0x10;
    header.sequence = control & 0x0F;

    header.function_code = FunctionCode::from_u8(reader.u8());

    if(parse_iin) {
        header.iin = Some(IinFlags::from_u16(reader.u16()));
    } else {
        header.iin = None;
    }

    header
}

impl PacketAdapter for PcapPacketAdpter {
    fn adapt(&self, raw: &[u8]) -> Result<Dnp3Frame, CaptureError> {
        let dnp3_data = get_dnp3_slice(raw);

        if dnp3_data.len() < 2 {
            return Err(CaptureError::FormatError("Not a DNP3 packet".to_string()))
        }

        let mut frame = Dnp3Frame::default();

        let mut reader = BufferReader::new(dnp3_data);

        reader.skip(2); // Skip 0x05 and 0x64 bytes

        frame.data_link = parse_data_link_header(&mut reader);
        frame.transport = parse_transport_header(&mut reader);
        frame.application = parse_application_header(&mut reader, !frame.data_link.control.dir);

        Ok(frame)
    }
}