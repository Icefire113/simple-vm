use std::{fs::File, io::Write};

use anyhow::Context;
use simple_vm::assembler::Assembler;

fn main() -> anyhow::Result<()> {
    let asm_path = std::env::args().nth(1).context("No sasm file given")?;
    let bin_path = std::env::args().nth(2).context("No output file given")?;
    let asm_text = std::fs::read_to_string(&asm_path).context("Failed to read file")?;

    println!("Assembling {}", asm_path);
    let assembled = Assembler::new(&asm_text)
        .assemble()
        .context("Failed to assemble")?;

    println!("Writing program binary to {}", bin_path);
    let mut f = File::options()
        .read(true)
        .write(true)
        .create_new(true)
        .open(bin_path)
        .context("Failed to open output file")?;
    f.write(&assembled.to_bin()).context("Write bin content")?;

    Ok(())
}
