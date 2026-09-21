use crate::assembler::tokenizer::errors::SASMTokenizeError;

#[derive(Debug, thiserror::Error)]
pub enum AssemblerError {
    #[error("Failed to tokenize")]
    TokenizeError(#[from] SASMTokenizeError),

    #[error("Illegal token at token pos {}", _0)]
    IllegalToken(usize),
}
