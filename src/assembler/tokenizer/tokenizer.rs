use std::{iter::Peekable, str::Chars};

use crate::assembler::{
    tokenizer::errors::SASMTokenizeError,
    tokenizer::token::{Keyword, LiteralToken, Operator, Token},
};

/// The tokenizer itself
#[derive(Debug)]
pub struct Tokenizer<'a> {
    input: &'a str,
    /// An iterator over the characters of the input
    chars: Peekable<Chars<'a>>,
    /// The current position, as a byte offset into the input
    pos: usize,
    /// A cache of newline indexes for calculating line and column numbers from byte positions
    newlines: Option<Vec<usize>>,
}

impl<'a> Tokenizer<'a> {
    /// Construct a new tokenizer for a given input
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.chars().peekable(),
            pos: 0,
            newlines: None,
        }
    }

    /// Parse the input to that was given to us into a list of raw tokens
    pub fn tokenize(&mut self) -> Result<Vec<Token>, SASMTokenizeError> {
        let mut tokens = vec![];

        while let Some(tok) = self.get_next_token() {
            match tok {
                Token::Illegal(pos, _) => {
                    let (line, col) = self
                        .pos_to_line_col(pos)
                        .expect("illegal token position should be in bounds");
                    return Err(SASMTokenizeError::IllegalToken(tok, line, col));
                }
                Token::Unknown(pos, _) => {
                    let (line, col) = self
                        .pos_to_line_col(pos)
                        .expect("unknown token position should be in bounds");
                    return Err(SASMTokenizeError::UnknownToken(tok, line, col));
                }
                _ => {}
            }
            tokens.push(tok);
        }

        Ok(tokens)
    }

    /// Gets the next token if one exists
    pub fn get_next_token(&mut self) -> Option<Token> {
        loop {
            let c = *self.chars.peek()?;

            match c {
                // whitespace
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                // newline, terminates the current line
                '\n' => {
                    self.advance();
                    return Some(Token::NewLine);
                }
                // numeric literal, supports decimal, hex and binary
                '0'..='9' => return Some(self.lex_number()),
                // a keyword, an identifier or a directive
                'a'..='z' | 'A'..='Z' | '_' => return Some(self.lex_identifier_or_keyword()),
                // a directive
                '.' => return Some(self.lex_directive()),
                // a char literal
                '\'' => return Some(self.lex_char()),
                // a string literal
                '"' => return Some(self.lex_string()),
                // operators
                '+' => {
                    self.advance();
                    return Some(Token::Operator(Operator::Plus));
                }
                '-' => {
                    self.advance();
                    return Some(Token::Operator(Operator::Minus));
                }
                // line comment
                '#' => self.skip_line_comment(),
                // label definition separator
                ':' => {
                    self.advance();
                    return Some(Token::Colon);
                }
                // anything else
                _ => {
                    let pos = self.pos;
                    self.advance();
                    return Some(Token::Unknown(pos, c.into()));
                }
            }
        }
    }

    /// Lex a numeric literal, supports decimal, hex (`0x`) and binary (`0b`) integers
    fn lex_number(&mut self) -> Token {
        let first = self.advance().expect("peeked char should exist");
        debug_assert!(first.is_ascii_digit());

        // check for a radix prefix
        if first == '0' {
            match self.chars.peek() {
                Some(&'x') | Some(&'X') => return self.lex_radix(16),
                Some(&'b') | Some(&'B') => return self.lex_radix(2),
                _ => {}
            }
        }

        let mut value = String::from(first);
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_digit() {
                value.push(c);
                self.advance();
            } else {
                break;
            }
        }

        match value.parse::<i32>() {
            Ok(n) => Token::Literal(LiteralToken::Int(n)),
            Err(_) => Token::Illegal(self.pos, value),
        }
    }

    /// Lex an integer in the given radix, the radix prefix (`0x`/`0X`/`0b`/`0B`) is expected to have been peeked
    fn lex_radix(&mut self, radix: u32) -> Token {
        // consume the prefix
        self.advance();

        let mut value = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                value.push(c);
                self.advance();
            } else {
                break;
            }
        }

        match u32::from_str_radix(&value, radix) {
            // the full unsigned 32 bit range is allowed so that immediates like `0xFFFFFFFF` work as expected
            Ok(n) => Token::Literal(LiteralToken::Int(n as i32)),
            Err(_) => Token::Illegal(self.pos, value),
        }
    }

    /// Lex an identifier or a keyword, keywords are matched case-insensitively while
    /// identifiers keep their original casing
    fn lex_identifier_or_keyword(&mut self) -> Token {
        let mut value = String::new();
        while let Some(&c) = self.chars.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                value.push(c);
                self.advance();
            } else {
                break;
            }
        }

        match value.to_ascii_lowercase().as_str() {
            "push" => Token::Keyword(Keyword::Push),
            "addu" => Token::Keyword(Keyword::Addu),
            "addc" => Token::Keyword(Keyword::Addc),
            "subu" => Token::Keyword(Keyword::Subu),
            "subc" => Token::Keyword(Keyword::Subc),
            "mulu" => Token::Keyword(Keyword::Mulu),
            "mulc" => Token::Keyword(Keyword::Mulc),
            "divc" => Token::Keyword(Keyword::Divc),
            "negu" => Token::Keyword(Keyword::Negu),
            "negc" => Token::Keyword(Keyword::Negc),
            "shl" => Token::Keyword(Keyword::Shl),
            "shr" => Token::Keyword(Keyword::Shr),
            "rotl" => Token::Keyword(Keyword::Rotl),
            "rotr" => Token::Keyword(Keyword::Rotr),
            "eq" => Token::Keyword(Keyword::Eq),
            "neq" => Token::Keyword(Keyword::Neq),
            "lt" => Token::Keyword(Keyword::Lt),
            "gt" => Token::Keyword(Keyword::Gt),
            "lteq" => Token::Keyword(Keyword::Le),
            "gteq" => Token::Keyword(Keyword::Ge),
            "swap" => Token::Keyword(Keyword::Swap),
            "pop" => Token::Keyword(Keyword::Pop),
            "dup" => Token::Keyword(Keyword::Dup),
            "popjump" => Token::Keyword(Keyword::PopJump),
            "jmp" => Token::Keyword(Keyword::Jump),
            "jnz" => Token::Keyword(Keyword::JumpIfNotZero),
            "jz" => Token::Keyword(Keyword::JumpIfZero),
            "jt" => Token::Keyword(Keyword::JumpIfTrue),
            "jf" => Token::Keyword(Keyword::JumpIfFalse),
            "dbg_stack" => Token::Keyword(Keyword::DebugStack),
            "clear_err" => Token::Keyword(Keyword::ClearEFlags),
            "clear_div_zero" => Token::Keyword(Keyword::ClearDivZero),
            "push_div_zero" => Token::Keyword(Keyword::PushDivisionByZeroFlag),
            "push_overflow" => Token::Keyword(Keyword::PushOverflowFlag),
            "clear_overflow" => Token::Keyword(Keyword::ClearOverflow),
            "exit" => Token::Keyword(Keyword::Exit),
            "call" => Token::Keyword(Keyword::Call),
            "ret" => Token::Keyword(Keyword::Ret),
            "or" => Token::Keyword(Keyword::BitOr),
            "and" => Token::Keyword(Keyword::BitAnd),
            "xor" => Token::Keyword(Keyword::BitXor),
            "not" => Token::Keyword(Keyword::BitNot),
            "pick" => Token::Keyword(Keyword::Pick),
            "move" => Token::Keyword(Keyword::Move),
            "syscall" => Token::Keyword(Keyword::Syscall),
            "fmac" => Token::Keyword(Keyword::Fmac),
            "fmau" => Token::Keyword(Keyword::Fmau),

            "true" => Token::Keyword(Keyword::True),
            "false" => Token::Keyword(Keyword::False),
            _ => Token::Identifier(value),
        }
    }

    /// Lex a directive, expects the next char to be a `.` followed by a name
    fn lex_directive(&mut self) -> Token {
        let dot_pos = self.pos;
        // consume the `.`
        self.advance();

        match self.chars.peek() {
            Some(&c) if c.is_ascii_alphabetic() || c == '_' => {
                let mut name = String::new();
                while let Some(&c) = self.chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        name.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                Token::Directive(name)
            }
            _ => Token::Unknown(dot_pos, '.'.into()),
        }
    }

    /// Lex a char literal, expects the next char to be a `'`.
    /// The contents must be exactly one character after escape processing
    fn lex_char(&mut self) -> Token {
        // consume the opening quote
        self.advance();

        match self.read_quoted('\'') {
            Some(value) => {
                let mut chars = value.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => Token::Literal(LiteralToken::Char(c)),
                    _ => Token::Illegal(self.pos, format!("'{value}'")),
                }
            }
            None => Token::Illegal(self.pos, "'".into()),
        }
    }

    /// Lex a string literal, expects the next char to be a `"`
    fn lex_string(&mut self) -> Token {
        // consume the opening quote
        self.advance();

        match self.read_quoted('"') {
            Some(value) => Token::Literal(LiteralToken::String(value)),
            None => Token::Illegal(self.pos, "\"".into()),
        }
    }

    /// Reads characters until the closing `quote`, processing backslash escape sequences
    /// (`\n`, `\t`, `\r`, `\0`, `\\`, `\'` and `\"`).
    ///
    /// Unknown escapes resolve to the escaped character itself.
    /// Returns `None` if the input ends before the closing quote is found
    fn read_quoted(&mut self, quote: char) -> Option<String> {
        let mut value = String::new();
        loop {
            match self.advance()? {
                '\\' => {
                    let escaped = self.advance()?;
                    value.push(match escaped {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '0' => '\0',
                        '\\' => '\\',
                        '\'' => '\'',
                        '"' => '"',
                        other => other,
                    });
                }
                c if c == quote => return Some(value),
                c => value.push(c),
            }
        }
    }

    /// Skips to the end of the current line comment, leaving the terminating newline
    /// in place so that it is emitted as a `NewLine` token
    fn skip_line_comment(&mut self) {
        // consume the `#`
        self.advance();

        while let Some(&c) = self.chars.peek() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
    }

    /// Turns a byte position into a line and column number of the underlying input string
    ///
    /// If the position is out of bounds, returns None, otherwise returns the line and column number
    fn pos_to_line_col(&mut self, pos: usize) -> Option<(usize, usize)> {
        if pos > self.input.len() {
            return None;
        }

        let newlines: &Vec<usize> = self.newlines.get_or_insert_with(|| {
            self.input
                .bytes()
                .enumerate()
                .filter_map(|(i, b)| (b == b'\n').then_some(i))
                .collect()
        });

        let line = newlines.partition_point(|&i| i < pos);

        let line_start = if line == 0 { 0 } else { newlines[line - 1] + 1 };

        Some((line + 1, pos - line_start))
    }

    /// Advance the tokenizer to the next character and increase our position
    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.pos += c.len_utf8();
        Some(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Result<Vec<Token>, SASMTokenizeError> {
        Tokenizer::new(input).tokenize()
    }

    #[test]
    fn lexes_int_literals() {
        let tokens = tokenize("42 0x1F 0X2a 0xFFFFFFFF 0b1010 0B11").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Literal(LiteralToken::Int(42)),
                Token::Literal(LiteralToken::Int(0x1F)),
                Token::Literal(LiteralToken::Int(0x2a)),
                Token::Literal(LiteralToken::Int(-1)),
                Token::Literal(LiteralToken::Int(0b1010)),
                Token::Literal(LiteralToken::Int(0b11)),
            ]
        );
    }

    #[test]
    fn invalid_radix_literals_are_illegal() {
        assert!(matches!(
            tokenize("0xZZ"),
            Err(SASMTokenizeError::IllegalToken(..))
        ));
        assert!(matches!(
            tokenize("0b2"),
            Err(SASMTokenizeError::IllegalToken(..))
        ));
        assert!(matches!(
            tokenize("0x"),
            Err(SASMTokenizeError::IllegalToken(..))
        ));
    }

    #[test]
    fn lexes_keywords_case_insensitively() {
        let tokens = tokenize("PUSh push True").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Keyword(Keyword::Push),
                Token::Keyword(Keyword::Push),
                Token::Keyword(Keyword::True),
            ]
        );
    }

    #[test]
    fn identifiers_keep_their_casing() {
        let tokens = tokenize("FooBar foo_bar").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("FooBar".into()),
                Token::Identifier("foo_bar".into()),
            ]
        );
    }

    #[test]
    fn lexes_directives_and_labels() {
        let tokens = tokenize(".data\nstart:").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Directive("data".into()),
                Token::NewLine,
                Token::Identifier("start".into()),
                Token::Colon,
            ]
        );
    }

    #[test]
    fn lexes_char_literals() {
        let tokens = tokenize("'a' '\\n' '\\'' ' ' 'é'").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Literal(LiteralToken::Char('a')),
                Token::Literal(LiteralToken::Char('\n')),
                Token::Literal(LiteralToken::Char('\'')),
                Token::Literal(LiteralToken::Char(' ')),
                Token::Literal(LiteralToken::Char('é')),
            ]
        );
    }

    #[test]
    fn invalid_char_literals_are_illegal() {
        // empty char
        assert!(matches!(
            tokenize("''"),
            Err(SASMTokenizeError::IllegalToken(..))
        ));
        // more than one char
        assert!(matches!(
            tokenize("'ab'"),
            Err(SASMTokenizeError::IllegalToken(..))
        ));
        // unterminated
        assert!(matches!(
            tokenize("'a"),
            Err(SASMTokenizeError::IllegalToken(..))
        ));
    }

    #[test]
    fn lexes_string_literals() {
        let tokens = tokenize("\"hi\\nthere\" \"\" \"say \\\"hi\\\"\"").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Literal(LiteralToken::String("hi\nthere".into())),
                Token::Literal(LiteralToken::String("".into())),
                Token::Literal(LiteralToken::String("say \"hi\"".into())),
            ]
        );
    }

    #[test]
    fn unterminated_string_is_illegal() {
        assert!(matches!(
            tokenize("\"abc"),
            Err(SASMTokenizeError::IllegalToken(..))
        ));
    }

    #[test]
    fn line_comments_preserve_the_terminating_newline() {
        let tokens = tokenize("push 1 # comment\npush 2").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Keyword(Keyword::Push),
                Token::Literal(LiteralToken::Int(1)),
                Token::NewLine,
                Token::Keyword(Keyword::Push),
                Token::Literal(LiteralToken::Int(2)),
            ]
        );
    }

    #[test]
    fn negative_immediates_lex_as_operator_then_int() {
        let tokens = tokenize("push -4").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Keyword(Keyword::Push),
                Token::Operator(Operator::Minus),
                Token::Literal(LiteralToken::Int(4)),
            ]
        );
    }

    #[test]
    fn unknown_chars_report_their_position() {
        let err = tokenize("push 1\n?").unwrap_err();
        let SASMTokenizeError::UnknownToken(_, line, col) = err else {
            panic!("expected an unknown token error");
        };
        assert_eq!((line, col), (2, 0));
    }

    #[test]
    fn handles_multibyte_characters_in_errors() {
        // 'é' is 2 bytes wide, so the line/col calculation must be byte based
        let err = tokenize("'é'\n\"unterminated").unwrap_err();
        let SASMTokenizeError::IllegalToken(_, line, col) = err else {
            panic!("expected an illegal token error");
        };
        assert_eq!((line, col), (2, 13));
    }
}
