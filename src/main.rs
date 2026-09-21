use anyhow::Context;

use crate::{
    assembler::{parser::Parser, tokenizer::tokenizer::Tokenizer},
    vm::VM,
};

mod assembler;
mod code;
mod vm;

fn main() -> anyhow::Result<()> {
    let asm_path = std::env::args().nth(1).context("No source file given")?;
    let asm_text = std::fs::read_to_string(asm_path).context("Failed to read file")?;

    let tokens: Vec<assembler::tokenizer::token::Token> = Tokenizer::new(&asm_text)
        .tokenize()
        .context("Failed to tokenize")?;

    let parser = Parser::new(&tokens);
    let (instructions, main_addr) = parser.parse().context("Failed to parse")?;

    let mut vm = VM::new();
    vm.load_program(instructions, main_addr);
    let r = vm.run().context("Failed to run")?;
    println!("Result: {:?}", r);

    Ok(())
}
