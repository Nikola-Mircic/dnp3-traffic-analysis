use crate::sniffing::buffer_reader::BufferReader;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Qualifier {
    StartStop8(u8, u8),     // 0x00
    StartStop16(u16, u16),  // 0x01
    #[default]
    AllObjects,             // 0x06
    Count8(u8),             // 0x07
    Count16(u16),           // 0x08
    CountWithIndex8(u8),    // 0x17
    CountWithIndex16(u16),  // 0x28
    Other(u8),
}

impl Qualifier {
    pub fn from_reader(reader: &mut BufferReader) -> Qualifier {
        let qualifier_val = reader.u8();

        match qualifier_val {
            0x00 => Qualifier::StartStop8(reader.u8(), reader.u8()),
            0x01 => Qualifier::StartStop16(reader.u16(), reader.u16()),
            0x06 => Qualifier::AllObjects,
            0x07 => Qualifier::Count8(reader.u8()),
            0x08 => Qualifier::Count16(reader.u16()),
            0x17 => Qualifier::CountWithIndex8(reader.u8()),
            0x28 => Qualifier::CountWithIndex16(reader.u16()),
            _ => Qualifier::Other(qualifier_val),
        }
    }
}