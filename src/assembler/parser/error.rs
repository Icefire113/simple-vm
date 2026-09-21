use crate::assembler::tokenizer::token::Token;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("No main label found")]
    NoMainLabel,

    #[error("Unexpected end of file")]
    UnexpectedEOF,

    #[error("Label already defined: {}", _0)]
    LabelAlreadyDefined(String),

    #[error("Undefined label(s): {:?}", _0)]
    UndefinedLabels(Vec<String>),

    #[error("Illegal token {:?} at token position: {}", _0, _1)]
    IllegalToken(Token, usize),
}
