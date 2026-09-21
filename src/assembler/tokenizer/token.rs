use std::fmt::Display;

use strum::EnumIter;

/// Represents the type of token
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    /// A literal value
    Literal(LiteralToken),
    /// Something like a table or column name
    Identifier(String),
    /// Something like a table or column name (but in quotes)
    QuotedIdentifier(String),
    /// Something like an `=` sign
    Operator(Operator),
    /// An important keyword like `SELECT`, `FROM`, etc
    Keyword(Keyword),

    // Grammars
    LParen,
    RParen,
    Comma,
    SemiColon,

    /// An illegal/ unexpected token at position `pos`, contains the position
    Illegal(usize, String),
    /// A token that we don't recognize, contanins the position
    Unknown(usize, String),
}

/// Represents an operator
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operator {
    Equals,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
    Plus,
    Minus,
    Star,
    Divide,
    Modulus,
    Pow,
}

/// Gets the binding power of a given token, used for pratt parsing
pub fn get_token_bp(op: &Token) -> Option<(u8, u8)> {
    match op {
        Token::Operator(op) => match op {
            Operator::Equals | Operator::NotEq => Some((5, 6)),
            Operator::Lt | Operator::Lte | Operator::Gt | Operator::Gte => Some((7, 8)),
            Operator::Plus | Operator::Minus => Some((9, 10)),
            Operator::Star | Operator::Divide | Operator::Modulus => Some((11, 12)),
            Operator::Pow => Some((13, 14)),
        },
        Token::Keyword(Keyword::Or) => Some((1, 2)),
        Token::Keyword(Keyword::And) => Some((3, 4)),
        _ => None,
    }
}

/// Gets the prefix binding power, used for pratt parsing
pub fn get_prefix_bp(tok: &Token) -> Option<u8> {
    match tok {
        Token::Operator(Operator::Minus) | Token::Keyword(Keyword::Not) => Some(13),
        _ => None,
    }
}

/// Represents a literal value token
#[derive(Debug, PartialEq, Clone)]
pub enum LiteralToken {
    Int(i32),
    BigInt(i64),
    Float(f32),
    BigFloat(f64),
    String(String),
}

/// Represents an important keyword that we should recognize
#[derive(Debug, PartialEq, EnumIter, Clone, Copy)]
pub enum Keyword {
    Create,
    Select,
    From,
    Where,
    Insert,
    Into,
    Values,
    Update,
    Delete,
    Set,
    Alter,
    Add,
    Drop,
    Table,
    Column,
    Database,
    Use,
    // type keywords
    Int,
    BigInt,
    Float,
    BigFloat,
    String,
    Bool,
    // Logical operator keywords
    All,
    And,
    Any,
    Between,
    Exists,
    Not,
    Or,
    Some,
    False,
    True,
    Null,
    Is,
    Left,
    Right,
    Outer,
    Join,
    Inner,
    Full,
    Cross,
    On,
    Load,
    Data,
    InFile,
    Index,
    Modifier,
    // Column modifiers
    Nullable,
    Indexed,
    Unique,
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Create => "CREATE",
            Self::Select => "SELECT",
            Self::From => "FROM",
            Self::Where => "WHERE",
            Self::Insert => "INSERT",
            Self::Into => "INTO",
            Self::Values => "VALUES",
            Self::Update => "UPDATE",
            Self::Delete => "DELETE",
            Self::Set => "SET",
            Self::Alter => "ALTER",
            Self::Add => "ADD",
            Self::Drop => "DROP",
            Self::Table => "TABLE",
            Self::Column => "COLUMN",
            Self::Database => "DATABASE",
            Self::Use => "USE",
            Self::Int => "int",
            Self::BigInt => "bigint",
            Self::Float => "float",
            Self::BigFloat => "bigfloat",
            Self::String => "string",
            Self::Bool => "bool",
            Self::All => "ALL",
            Self::And => "AND",
            Self::Any => "ANY",
            Self::Between => "BETWEEN",
            Self::Exists => "EXISTS",
            Self::Not => "NOT",
            Self::Or => "OR",
            Self::Some => "SOME",
            Self::False => "false",
            Self::True => "true",
            Self::Null => "NULL",
            Self::Is => "IS",
            Self::Left => "LEFT",
            Self::Right => "RIGHT",
            Self::Outer => "OUTER",
            Self::Join => "JOIN",
            Self::Inner => "INNER",
            Self::On => "ON",
            Self::Load => "LOAD",
            Self::Data => "DATA",
            Self::InFile => "INFILE",
            Self::Index => "INDEX",
            Self::Modifier => "MODIFIER",
            Self::Nullable => "nullable",
            Self::Indexed => "indexed",
            Self::Unique => "unique",
            Self::Full => "FULL",
            Self::Cross => "CROSS",
        };
        write!(f, "{s}")
    }
}
