use std::fmt::Display;

/// Represents the type of token
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    /// A literal value
    Literal(LiteralToken),
    Identifier(String),
    /// A directive, like `.data`, without the leading dot
    Directive(String),
    /// Something like a `+` sign
    Operator(Operator),
    /// An important keyword
    Keyword(Keyword),

    // Grammars
    NewLine,
    Colon,

    /// An illegal/ unexpected token at position `pos`, contains the position
    Illegal(usize, String),
    /// A token that we don't recognize, contanins the position
    Unknown(usize, String),
}

/// Represents an operator
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operator {
    Plus,
    Minus,
}

/// Represents a literal value token
#[derive(Debug, PartialEq, Clone)]
pub enum LiteralToken {
    /// An integer literal, supports decimal, hex (`0x`) and binary (`0b`)
    Int(i32),
    /// A string literal, supports backslash escape sequences
    String(String),
    /// A single character literal, supports backslash escape sequences
    Char(char),
}

/// Represents an important keyword that we should recognize
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Keyword {
    Push,
    Addu,
    Addc,
    Subu,
    Subc,
    Mulu,
    Mulc,
    Divc,
    Negu,
    Negc,
    Shl,
    Shr,
    Rotl,
    Rotr,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    Swap,
    Pop,
    Dup,
    PopJump,
    Jump,
    JumpIfNotZero,
    JumpIfZero,
    JumpIfTrue,
    JumpIfFalse,
    DebugStack,
    Exit,
    ClearEFlags,
    ClearDivZero,
    ClearOverflow,
    PushDivisionByZeroFlag,
    PushOverflowFlag,
    Call,
    Ret,
    BitOr,
    BitAnd,
    BitXor,
    BitNot,
    Pick,
    Move,
    Syscall,

    True,
    False,
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Push => "push",
            Self::Addu => "addu",
            Self::Addc => "addc",
            Self::Subu => "subu",
            Self::Subc => "subc",
            Self::Mulu => "mulu",
            Self::Mulc => "mulc",
            Self::True => "true",
            Self::False => "false",
            Self::Divc => "divc",
            Self::Negu => "negu",
            Self::Negc => "negc",
            Self::Shl => "shl",
            Self::Shr => "shr",
            Self::Rotl => "rotl",
            Self::Rotr => "rotr",
            Self::Eq => "eq",
            Self::Neq => "neq",
            Self::Lt => "lt",
            Self::Gt => "gt",
            Self::Le => "lteq",
            Self::Ge => "gteq",
            Self::Swap => "swap",
            Self::Pop => "pop",
            Self::Dup => "dup",
            Self::PopJump => "popjump",
            Self::Jump => "jmp",
            Self::JumpIfNotZero => "jnz",
            Self::JumpIfZero => "jz",
            Self::JumpIfTrue => "jt",
            Self::JumpIfFalse => "jf",
            Self::DebugStack => "dbg_stack",
            Self::Exit => "exit",
            Self::ClearEFlags => "clear_err",
            Self::ClearDivZero => "clear_div_zero",
            Self::ClearOverflow => "clear_overflow",
            Self::PushDivisionByZeroFlag => "push_div_zero",
            Self::PushOverflowFlag => "push_overflow",
            Self::Call => "call",
            Self::Ret => "ret",
            Self::BitOr => "or",
            Self::BitAnd => "and",
            Self::BitXor => "xor",
            Self::BitNot => "not",
            Self::Pick => "pick",
            Self::Move => "move",
            Self::Syscall => "syscall",
        };
        write!(f, "{s}")
    }
}
