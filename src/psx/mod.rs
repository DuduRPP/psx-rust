use std::path::Path;

use anyhow::Result;

mod bios;
mod ram;
mod cpu;
mod interconnect;

use bios::Bios;
use cpu::Cpu;
use interconnect::Interconnect;

use self::ram::Ram;

pub fn run() -> Result<()> {
    let bios = Bios::new(&Path::new("./bios/scph1001.bin"))?;
    let ram = Ram::new();
    let inter = Interconnect::new(bios,ram);
    let mut cpu = Cpu::new(inter);

    loop {
        cpu.run_next_instruction();
    }
}
