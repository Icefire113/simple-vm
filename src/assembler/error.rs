use crate::{
    assembler::{parser::error::ParseError, tokenizer::errors::SASMTokenizeError},
    code::VMInstructionParseError,
};

#[derive(Debug, thiserror::Error)]
pub enum AssemblerError {
    #[error("Failed to tokenize")]
    TokenizeError(#[from] SASMTokenizeError),

    #[error("Failed to parse")]
    ParseError(#[from] ParseError),
}

#[derive(Debug, thiserror::Error)]
pub enum AssembledParseError {
    #[error("Bad magic")]
    BadMagic,
    #[error("Bad version")]
    BadVersion,
    #[error("Too short")]
    TooShort,

    #[error("Constructing instruction")]
    ConstructingInstruction(#[from] VMInstructionParseError),
}
