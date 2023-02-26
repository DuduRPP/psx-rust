use super::bios::Bios;
use super::cpu::map;
use super::ram::Ram;

/// Responsible for connecting the bios to other peripherals
pub struct Interconnect {
    bios: Bios,
    ram: Ram,
}

impl Interconnect {
    pub fn new(bios: Bios, ram: Ram) -> Interconnect {
        Interconnect { bios, ram }
    }

    pub fn load8(&self, addr: u32) -> u8 {
        let addr = map::mask_region(addr);

        if let Some(offset) = map::RAM.contains(addr) {
            return self.ram.load8(offset);
        }

        if let Some(offset) = map::BIOS.contains(addr) {
            return self.bios.load8(offset);
        }

        if let Some(_) = map::EXPANSION_1.contains(addr) {
            println!("Unhandled load8 at Expansion1 register {:08x}", addr);
            return 0xff;
        }

        panic!("unhandled load8 at address {:08x}", addr);
    }

    pub fn load32(&self, addr: u32) -> u32 {
        if addr % 4 != 0 {
            panic!("Unaligned load32 address {:08x}", addr);
        }

        let addr = map::mask_region(addr);

        if let Some(offset) = map::BIOS.contains(addr) {
            return self.bios.load32(offset);
        }

        if let Some(offset) = map::RAM.contains(addr) {
            return self.ram.load32(offset);
        }
        if let Some(offset) = map::IRQ_CONTROL.contains(addr) {
            println!("unhandled load IRQ control: {}",offset);
            return 0;
        }

        panic!("unhandled fetch32 at address {:08x}", addr);
    }

    pub fn store8(&mut self, addr: u32, val: u8) {
        let addr = map::mask_region(addr);

        if let Some(offset) = map::RAM.contains(addr) {
            return self.ram.store8(offset, val);
        }
        if let Some(offset) = map::EXPANSION_2.contains(addr) {
            println!("Unhandled write byte to Expansion2 register {:x}", offset);
            return;
        }

        panic!("unhandled store16 into address {:08x}", addr)
    }

    pub fn store16(&mut self, addr: u32, _val: u16) {
        if addr % 2 != 0 {
            panic!("Unaligned store16 address {:08x}", addr)
        }

        let addr = map::mask_region(addr);

        if let Some(offset) = map::SPU.contains(addr) {
            println!("Unhandled write half to SPU register {:x}", offset);
            return;
        }
        
        if let Some(offset) = map::TIMERS.contains(addr){
            println!("Unhandled write to time register: {:08x}",offset);
            return ;
        }

        panic!("unhandled store16 into address {:08x}", addr)
    }

    pub fn store32(&mut self, addr: u32, val: u32) {
        if addr % 4 != 0 {
            panic!("Unaligned store32 address {:08x}", addr);
        }

        let addr = map::mask_region(addr);

        if let Some(offset) = map::MEMLCONTROL.contains(addr) {
            match offset {
                0 => {
                    if val != 0x1f000000 {
                        panic!("Bad expansion 1 base address: 0x{:08x}", val);
                    }
                }
                4 => {
                    if val != 0x1f802000 {
                        panic!("Bad expansion 2 base address: 0x{:08x}", val);
                    }
                }
                _ => println!("unhandled write MEMLCONTROL register {:08x}", addr),
            }
            return;
        }

        if let Some(_) = map::RAM_SIZE.contains(addr) {
            println!("unhandled write RAM_SIZE register");
            return;
        }
        
        if let Some(offset) = map::IRQ_CONTROL.contains(addr) {
            println!("unhandled write IRQ control: {} <- {:08x}",offset,val);
            return;
        }

        if let Some(offset) = map::RAM.contains(addr) {
            self.ram.store32(offset, val);
            return;
        }

        if let Some(_) = map::CACHE_CONTROL.contains(addr) {
            println!("unhandled write CACHE_CONTROL register");
            return;
        }

        panic!("unhandled store32 at address {:08x}", addr);
    }
}
