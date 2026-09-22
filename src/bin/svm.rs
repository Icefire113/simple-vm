use anyhow::Context;
use simple_vm::{assembler::Assembled, vm::VM};

fn main() -> anyhow::Result<()> {
    let vm_bin_path = std::env::args().nth(1).context("No program file given")?;
    let bytes: Vec<u8> = std::fs::read(vm_bin_path).context("Failed to read file")?;

    let assembled = Assembled::from_bin(&bytes).context("Failed to load binary")?;

    let mut vm = VM::new();
    vm.load_program(assembled.program, assembled.entry);
    let r = vm.run().context("Failed to run")?;
    println!("Result: {:?}", r);

    Ok(())
}
