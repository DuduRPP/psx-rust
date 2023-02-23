use std::path::Path;

use anyhow::Result;

mod bios;
mod cpu;
mod interconnect;

use bios::Bios;
use cpu::Cpu;
use interconnect::Interconnect;

pub fn run() -> Result<()> {
    let bios = Bios::new(&Path::new("./bios/scph1001.bin"))?;
    let inter = Interconnect::new(bios);
    let mut cpu = Cpu::new(inter);

    loop {
        cpu.run_next_instruction();
    }
}
