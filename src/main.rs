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

#[cfg(test)]
mod tests {
    use crate::vm::Value;

    fn run_code(asm: &str) -> Value {
        use crate::assembler::{
            parser::Parser,
            tokenizer::{token::Token, tokenizer::Tokenizer},
        };
        let tokens: Vec<Token> = Tokenizer::new(asm).tokenize().unwrap();
        let parser = Parser::new(&tokens);
        let (instructions, main_addr) = parser.parse().unwrap();
        let mut vm = crate::vm::VM::new();
        vm.load_program(instructions, main_addr);
        vm.run().unwrap()
    }

    #[test]
    fn test1() {
        let asm_test = r"
:loc_2
    jmp loc_1


:ret_2
    divc
    push 10
    mulc
    push 1
    addc
    push 26
    subu
    exit

:main
    push 2
    push 5
    addu
    push 3
    mulc
    negu
    push -4
    jmp loc_2
:loc_1
    dbg_stack
    swap
    dbg_stack
    jmp ret_2
";

        let r = run_code(asm_test);
        assert_eq!(r, crate::vm::Value::Int(-25));
    }

    #[test]
    fn test2() {
        let asm_test = r"
        :main
    push 10
    push 32
    shl 2        # 32 << 2 = 128
    addu
    exit         # exits with 138
        ";

        let r = run_code(asm_test);
        assert_eq!(r, crate::vm::Value::Int(138));
    }
}
