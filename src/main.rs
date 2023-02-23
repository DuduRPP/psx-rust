use anyhow::Result;

mod cpu;
mod bios;
mod interconnect;
mod instruction;
mod psx;

fn main() -> Result<()>{
    psx::run()?;
    Ok(())
}
