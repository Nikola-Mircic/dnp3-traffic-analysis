/// Control octet of the data link header.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ControlField {
    pub dir: bool,
    pub primary: bool,
    pub fcb: bool,
    pub fcv: bool,
    pub function_code: u8,
}

impl ControlField {
    pub fn from_u8(v: u8) -> Self {
        ControlField{
            dir: v & 0x80 != 0,
            primary: v & 0x40 != 0,
            fcb: v & 0x20 != 0,
            fcv: v & 0x10 != 0,
            function_code: v & 0xF
        }
    }
}