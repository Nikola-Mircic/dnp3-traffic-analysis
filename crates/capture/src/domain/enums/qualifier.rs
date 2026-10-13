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