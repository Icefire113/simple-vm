use thiserror::Error;

use crate::assembler::tokenizer::token::Token;

/// Represents an error that occurs while tokenizing
#[derive(Debug, Error)]
pub enum SQLTokenizeError {
    #[error("Illegal token `{:?}` at: {}:{}", 0, 1, 2)]
    IllegalToken(Token, usize, usize),

    #[error("Unknown token `{:?}` at position: {}:{}", 0, 1, 2)]
    UnknownToken(Token, usize, usize),
}
