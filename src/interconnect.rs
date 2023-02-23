use crate::bios::Bios;
use crate::cpu::map;

pub struct Interconnect{
    bios: Bios,
}

impl Interconnect{
    pub fn new(bios:Bios) -> Interconnect{
        Interconnect { bios }
    }

    pub fn load32(&self, addr:u32) -> u32 {
        if let Some(offset) = map::BIOS.contains(addr){
            return self.bios.load32(offset);
        }

        panic!("unhandled fetch32 at address {:08x}",addr);
    }
}
