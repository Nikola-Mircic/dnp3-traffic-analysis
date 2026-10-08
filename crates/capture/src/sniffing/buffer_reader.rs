use std::cmp::min;

pub struct BufferReader<'a> {
    buf: &'a[u8],
    pos: usize
}

impl<'a> BufferReader<'a> {
    pub fn new(buf: &'a[u8]) -> Self {
        BufferReader { buf, pos: 0 }
    }

    pub fn rest(&self) -> &'a[u8] {
        &self.buf[self.pos..]
    }

    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }

    pub fn read(&mut self, n: usize) -> &[u8] {
        let end_idx = self.pos + min(self.remaining(), n);
        let slice = &self.buf[self.pos..end_idx];
        self.pos = end_idx;
        slice
    }
    
    pub fn skip(&mut self, n: usize) {
        self.pos += min(self.remaining(), n);
    }

    pub fn u8(&mut self) -> u8{
        self.read(1)[0]
    }

    pub fn u16(&mut self) -> u16{
        u16::from_le_bytes(self.read(2).try_into().unwrap())
    }

    pub fn u32(&mut self) -> u32{
        u32::from_le_bytes(self.read(4).try_into().unwrap())
    }
}