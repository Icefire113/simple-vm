use thiserror::Error;

use crate::assembler::tokenizer::token::Token;

/// Represents an error that occurs while tokenizing
#[derive(Debug, Error)]
pub enum SASMTokenizeError {
    #[error("Illegal token `{:?}` at: {}:{}", _0, _1, _2)]
    IllegalToken(Token, usize, usize),

    #[error("Unknown token `{:?}` at position: {}:{}", _0, _1, _2)]
    UnknownToken(Token, usize, usize),
}
