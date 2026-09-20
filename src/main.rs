use anyhow::Context;

use crate::{code::VMInstruction, vm::VM};

mod code;
mod vm;

fn main() -> anyhow::Result<()> {
    let mut vm: VM = VM::new();
    vm.load_program(vec![
        VMInstruction::PushImm(2),
        VMInstruction::PushImm(5),
        VMInstruction::AddUnchecked, // 5 + 2 = 7
        VMInstruction::PushImm(3),
        // 3 * 7 = 21
        VMInstruction::MulChecked,
        // -1 * 21 = -21
        VMInstruction::Neg,
        VMInstruction::PushImm(-4),
        VMInstruction::DebugStack,
        VMInstruction::Swap,
        VMInstruction::DebugStack,
        // -21 / -4 = 5
        VMInstruction::DivChecked,
        VMInstruction::PushImm(10),
        // 5 * 10 = 50
        VMInstruction::MulUnchecked,
        VMInstruction::PushImm(1),
        // 50 + 1 = 51
        VMInstruction::AddChecked,
        VMInstruction::PushImm(26),
        // 26 - 51 = -25
        VMInstruction::Sub,
        VMInstruction::Exit,
    ]);
    let r: i32 = vm.run().context("VM Run")?;
    println!("Result: {:?}", r);
    Ok(())
}
