use super::bios::Bios;
use super::cpu::map;
use super::ram::Ram;

/// Responsible for connecting the bios to other peripherals
pub struct Interconnect {
    bios: Bios,
    ram: Ram,
}

impl Interconnect {
    pub fn new(bios: Bios, ram:Ram) -> Interconnect {
        Interconnect { bios, ram }
    }

    pub fn load32(&self, addr: u32) -> u32 {
        if addr % 4 != 0{
            panic!("Unaligned load32 address {:08x}",addr);
        }

        if let Some(offset) = map::BIOS.contains(addr) {
            return self.bios.load32(offset);
        }

        if let Some(offset) = map::RAM.contains(addr){
            return self.ram.load32(offset);
        }

        panic!("unhandled fetch32 at address {:08x}", addr);
    }

    pub fn store32(&mut self, addr: u32, val: u32){
        if addr % 4 != 0{
            panic!("Unaligned store32 address {:08x}",addr);
        }

        if let Some(offset) = map::MEMLCONTROL.contains(addr){
            match offset {
                0 => if val != 0x1f000000{
                    panic!("Bad expansion 1 base address: 0x{:08x}",val);
                }
                4 => if val != 0x1f802000{
                    panic!("Bad expansion 2 base address: 0x{:08x}",val);
                }
                _ => println!("unhandled write MEMLCONTROL register {:08x}",addr)
            }
            return ;
        }

        if let Some(_) = map::RAM_SIZE.contains(addr){
            println!("unhandled write RAM_SIZE register");
            return ; 
        }

        if let Some(offset) = map::RAM.contains(addr){
            println!("Escreveu na RAM: {:08x}", val);
            self.ram.store32(offset, val)
        }

        if let Some(_) = map::CACHE_CONTROL.contains(addr){
            println!("unhandled write CACHE_CONTROL register");
            return ; 
        }

        panic!("unhandled store32 at address {:08x}", addr);
    }
}
