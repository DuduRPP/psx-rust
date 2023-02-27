use std::path::Path;

use anyhow::Result;

mod bios;
mod cpu;
mod interconnect;
mod ram;
mod dma;

use bios::Bios;
use cpu::Cpu;
use interconnect::Interconnect;

use self::{ram::Ram, dma::Dma};

pub fn run() -> Result<()> {
    let bios = Bios::new(&Path::new("./bios/scph1001.bin"))?;
    let ram = Ram::new();
    let dma  = Dma::new();
    let inter = Interconnect::new(bios, ram, dma);
    let mut cpu = Cpu::new(inter);

    loop {
        cpu.run_next_instruction();
    }
}
