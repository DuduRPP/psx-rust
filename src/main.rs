use anyhow::Result;

mod psx;

fn main() -> Result<()> {
    psx::run()?;
    Ok(())
}
