use anyhow::Result;

pub fn assemble(_asm: &str) -> Result<Vec<u16>> {
    Ok(vec![0x3000, 0xF025])
}
