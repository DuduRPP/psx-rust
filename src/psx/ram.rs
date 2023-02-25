/// PSX RAM state
pub struct Ram {
    /// RAM data buffer
    data: Vec<u8>,
}

impl Ram {
    pub fn new() -> Self {
        let data = vec![0xca; 2 * 1024 * 1024];
        Ram { data }
    }

    pub fn load8(&self, offset: u32) -> u8 {
        self.data[offset as usize]
    }

    /// Load little endian value from RAM data buffer
    pub fn load32(&self, offset: u32) -> u32 {
        let offset = offset as usize;

        let b0 = self.data[offset] as u32;
        let b1 = self.data[offset + 1] as u32;
        let b2 = self.data[offset + 2] as u32;
        let b3 = self.data[offset + 3] as u32;

        b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
    }

    pub fn store8(&mut self, offset: u32, val: u8) {
        self.data[offset as usize] = val;
    }

    /// Strore in RAM data buffer as little endian
    pub fn store32(&mut self, offset: u32, val: u32) {
        let offset = offset as usize;

        let b0 = val as u8;
        let b1 = (val >> 8) as u8;
        let b2 = (val >> 16) as u8;
        let b3 = (val >> 24) as u8;

        self.data[offset] = b0;
        self.data[offset + 1] = b1;
        self.data[offset + 2] = b2;
        self.data[offset + 3] = b3;
    }
}
