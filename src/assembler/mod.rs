use crate::{
    assembler::{error::AssemblerError, tokenizer::tokenizer::Tokenizer},
    code::VMInstruction,
};

pub mod error;
pub mod parser;
pub mod tokenizer;

#[derive(Debug)]
pub struct Assembler<'a> {
    tokenizer: Tokenizer<'a>,
}

impl<'a> Assembler<'a> {
    /// Creates a new assembler with a given source text
    pub fn new(text: &'a str) -> Self {
        Self {
            tokenizer: Tokenizer::new(text),
        }
    }

    /// Tokenizes the source text and converts it to a list of vm instructions
    pub fn to_instructions(&mut self) -> Result<Vec<VMInstruction>, AssemblerError> {
        todo!()
    }
}
