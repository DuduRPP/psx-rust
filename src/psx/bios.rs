use anyhow::{anyhow, Result};
use std::{fs::File, io::Read, path::Path};

/// PSX BIOS implementation
pub struct Bios {
    data: Vec<u8>,
}

impl Bios {
    const BIOS_SIZE: u64 = 512 * 1024;

    pub fn new(path: &Path) -> Result<Bios> {
        let file = File::open(path)?;
        let mut data = Vec::new();
        file.take(Bios::BIOS_SIZE).read_to_end(&mut data)?;
        if data.len() == Bios::BIOS_SIZE as usize {
            Ok(Bios { data })
        } else {
            Err(anyhow!("Invalid BIOS size"))
        }
    }

    /// Load 32 bits from the BIOS with some offset position
    pub fn load32(&self, offset: u32) -> u32 {
        let offset = offset as usize;

        let b0 = self.data[offset + 0] as u32;
        let b1 = self.data[offset + 1] as u32;
        let b2 = self.data[offset + 2] as u32;
        let b3 = self.data[offset + 3] as u32;

        b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
    }
}
