use std::path::Path;

use anyhow::Result;

use crate::bios::Bios;
use crate::cpu::Cpu;
use crate::interconnect::Interconnect;

pub fn run() -> Result<()>{
    let bios = Bios::new(&Path::new("./bios/scph1001.bin"))?;
    let inter = Interconnect::new(bios);
    let mut cpu = Cpu::new(inter);
    
    loop{
        cpu.run_next_instruction();
    }
}
