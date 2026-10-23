use std::{collections::HashMap, range::Range};

use anyhow::Context;
use simple_vm::{assembler::Assembler, code::VMInstruction};

#[derive(Debug, Clone, PartialEq, Eq)]
enum CodeBlockTerminator {
    /// A jump to an address, address may be `None` in the case of a `popjump`
    Jump(Option<usize>),
    Branch {
        taken: usize,
        not_taken: usize,
    },
    Return,
    Halt,
    Exit,
    EOF,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodeBlock {
    range: Range<usize>,
    terminator: Option<CodeBlockTerminator>,
}

impl CodeBlock {
    pub fn body<'a>(&self, program: &'a [VMInstruction]) -> &'a [VMInstruction] {
        let end = if self.terminator.is_some() {
            self.range.end
        } else {
            program.len()
        };
        &program[self.range.start..end]
    }
}

fn main() -> anyhow::Result<()> {
    let asm_path = std::env::args().nth(1).context("No sasm file given")?;
    let asm_text = std::fs::read_to_string(&asm_path).context("Failed to read file")?;
    let assembled = Assembler::new(&asm_text)
        .assemble()
        .context("Failed to assemble")?;

    let mut block_starts: Vec<usize> = vec![0, assembled.entry];
    for (i, instr) in assembled.program.iter().enumerate() {
        match instr {
            VMInstruction::Call(addr) => block_starts.push(*addr as usize),
            VMInstruction::Jump(addr)
            | VMInstruction::JumpIfFalse(addr)
            | VMInstruction::JumpIfTrue(addr)
            | VMInstruction::JumpIfZero(addr)
            | VMInstruction::JumpIfNotZero(addr) => {
                block_starts.push(*addr as usize);
                block_starts.push(i + 1);
            }
            VMInstruction::PopJump
            | VMInstruction::Halt
            | VMInstruction::Exit
            | VMInstruction::Return => block_starts.push(i + 1),
            _ => (),
        }
    }
    // Sort the block starts, remove duplicates, and remove any that are out of range
    block_starts.sort();
    block_starts.dedup();
    block_starts.retain(|e| *e < assembled.program.len());

    // addr -> block id map
    let starts: HashMap<usize, usize> = block_starts
        .iter()
        .copied()
        .enumerate()
        .map(|(id, addr)| (addr, id))
        .collect();
    dbg!(starts);

    let blocks: Vec<CodeBlock> = block_starts
        .iter()
        .enumerate()
        .map(|(i, &start)| {
            let end_pos = *block_starts.get(i + 1).unwrap_or(&assembled.program.len());
            let end_instr = assembled.program.get(end_pos - 1);
            CodeBlock {
                range: (start..(end_pos - 1)).into(),
                terminator: match end_instr {
                    Some(instr) => match instr {
                        VMInstruction::PopJump => Some(CodeBlockTerminator::Jump(None)),
                        VMInstruction::Jump(addr) => {
                            Some(CodeBlockTerminator::Jump(Some(*addr as usize)))
                        }
                        VMInstruction::JumpIfNotZero(addr)
                        | VMInstruction::JumpIfZero(addr)
                        | VMInstruction::JumpIfTrue(addr)
                        | VMInstruction::JumpIfFalse(addr) => Some(CodeBlockTerminator::Branch {
                            taken: *addr as usize,
                            not_taken: end_pos,
                        }),
                        VMInstruction::Return => Some(CodeBlockTerminator::Return),
                        VMInstruction::Exit => Some(CodeBlockTerminator::Exit),
                        VMInstruction::Halt => Some(CodeBlockTerminator::Halt),
                        _ if end_pos == assembled.program.len() => Some(CodeBlockTerminator::EOF),
                        _ => None,
                    },
                    None => Some(CodeBlockTerminator::EOF),
                },
            }
        })
        .collect();

    for block in &blocks {
        dbg!(block);
        dbg!(block.body(&assembled.program));
    }

    Ok(())
}
