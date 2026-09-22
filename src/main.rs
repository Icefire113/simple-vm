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
    use anyhow::Context;

    use crate::vm::Value;

    fn run_code(asm: &str) -> anyhow::Result<Value> {
        use crate::assembler::{
            parser::Parser,
            tokenizer::{token::Token, tokenizer::Tokenizer},
        };
        let tokens: Vec<Token> = Tokenizer::new(asm).tokenize().context("Tokenizing")?;
        let parser = Parser::new(&tokens);
        let (instructions, main_addr) = parser.parse().context("parsing")?;
        let mut vm = crate::vm::VM::new();
        vm.load_program(instructions, main_addr);
        let r = vm.run().context("VM Run")?;
        Ok(r)
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

        let r = run_code(asm_test).unwrap();
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

        let r = run_code(asm_test).unwrap();
        assert_eq!(r, crate::vm::Value::Int(138));
    }

    #[test]
    fn call_stack_test() {
        let test_asm = r"
        # Same example from main

:swap_dbg
    dbg_stack
    swap
    dbg_stack
    ret

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
    call swap_dbg
    jmp ret_2

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
# Should exit with Value::Int(-25)
";
        let r = run_code(test_asm).unwrap();
        assert_eq!(r, crate::vm::Value::Int(-25));
    }

    #[test]
    fn test_and() {
        let test_asm = r"
        :main
            push 0xFF
            push 0
            and
            exit
        ";
        let r = run_code(test_asm).unwrap();
        assert_eq!(r, crate::vm::Value::Int(0));
    }

    #[test]
    fn test_or() {
        let test_asm = r"
        :main
            push 0x0F
            push 0xF0
            or
            exit
        ";
        let r = run_code(test_asm).unwrap();
        assert_eq!(r, crate::vm::Value::Int(0xFF));
    }

    #[test]
    fn test_xor() {
        let test_asm = r"
        :main
            push 0xFF
            push 0xF0
            xor
            exit
        ";
        let r = run_code(test_asm).unwrap();
        assert_eq!(r, crate::vm::Value::Int(0xF));
    }

    #[test]
    fn test_not() {
        let test_asm = r"
        :main
            push true
            not
            exit
        ";
        let r = run_code(test_asm).unwrap();
        assert_eq!(r, crate::vm::Value::Bool(false));
    }

    #[test]
    fn test_pick() {
        let test_asm = r"
        :main
            push 10
            push 20
            push 30

            # after those pushes, the stack will be [10, 20, 30]
            #                              indexes:  2   1   0
            pick 2
            exit
        ";
        let r = run_code(test_asm).unwrap();
        assert_eq!(r, crate::vm::Value::Int(10));
    }

        #[test]
    fn test_move() {
        let test_asm = r"
        :main
            push 10
            push 20
            push 30

            # after those pushes, the stack will be [10, 20, 30]
            #                              indexes:  2   1   0
            move 1
            # should be [10, 30, 20]
            exit
        ";
        let r = run_code(test_asm).unwrap();
        assert_eq!(r, crate::vm::Value::Int(20));
    }
}
