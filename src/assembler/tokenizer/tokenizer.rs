use std::{iter::Peekable, str::Chars};

use strum::IntoEnumIterator;

use crate::assembler::{
    tokenizer::errors::SQLTokenizeError,
    tokenizer::token::{Keyword, LiteralToken, Operator, Token},
};

/// The tokenizer itself
#[derive(Debug)]
pub struct Tokenizer<'a> {
    input: &'a str,
    num_chars: usize,
    /// An iterator over the characters of the input
    chars: Peekable<Chars<'a>>,
    /// The current position
    pos: usize,
    /// A cache of newline indexes for calculating line and column numbers from char positions
    newlines: Option<Vec<usize>>,
}

impl<'a> Tokenizer<'a> {
    /// Construct a new tokenizer for a given input
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            num_chars: input.chars().count(),
            chars: input.chars().peekable(),
            pos: 0,
            newlines: None,
        }
    }

    /// Parse the input to that was given to us into a list of raw tokens
    pub fn tokenize(&mut self) -> anyhow::Result<Vec<Token>, SQLTokenizeError> {
        let mut tokens = vec![];

        while let Some(tok) = self.get_next_token() {
            match tok {
                Token::Illegal(pos, _) => {
                    let (line, col) = self.char_pos_to_line_col(pos).unwrap();
                    return Err(SQLTokenizeError::IllegalToken(tok, line, col));
                }
                Token::Unknown(pos, _) => {
                    let (line, col) = self.char_pos_to_line_col(pos).unwrap();
                    return Err(SQLTokenizeError::UnknownToken(tok, line, col));
                }
                _ => {}
            }
            tokens.push(tok);
        }

        Ok(tokens)
    }

    /// Gets the next token if one exists
    pub fn get_next_token(&mut self) -> Option<Token> {
        if self.pos >= self.num_chars {
            return None;
        }

        match self.chars.peek() {
            Some(&c) => match c {
                // whitespace
                ' ' | '\t' | '\n' | '\r' => {
                    self.advance();
                    self.get_next_token()
                }
                // numeric literal
                '0'..='9' => {
                    let mut value = String::new();
                    let mut has_dot = false;

                    while let Some(&c) = self.chars.peek() {
                        if c.is_ascii_digit() {
                            value.push(c);
                            self.advance();
                        } else if c == '.' && !has_dot {
                            has_dot = true;

                            value.push(c);
                            self.advance();
                        } else {
                            break;
                        }
                    }

                    if let Ok(n) = value.parse::<i32>() {
                        Some(Token::Literal(LiteralToken::Int(n)))
                    } else if let Ok(n) = value.parse::<i64>() {
                        Some(Token::Literal(LiteralToken::BigInt(n)))
                    } else if let Ok(n) = value.parse::<f32>() {
                        Some(Token::Literal(LiteralToken::Float(n)))
                    } else if let Ok(n) = value.parse::<f64>() {
                        Some(Token::Literal(LiteralToken::BigFloat(n)))
                    } else {
                        Some(Token::Illegal(self.pos, value))
                    }
                }
                // a keyword, or identifier
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut value = String::new();
                    while let Some(&c) = self.chars.peek() {
                        match c {
                            'a'..='z' | 'A'..='Z' | '_' | '0'..='9' | '.' => {
                                value.push(c);
                                self.advance();
                            }
                            _ => break,
                        }
                    }

                    // check if it's a keyword
                    let tok_type = Keyword::iter()
                        .find(|k| k.to_string().to_uppercase() == value.to_uppercase())
                        .map(Token::Keyword)
                        .unwrap_or(Token::Identifier(value.clone()));

                    Some(tok_type)
                }
                // a string literal
                '\'' => {
                    self.advance();
                    let mut value = String::new();
                    let mut found_end = false;
                    while let Some(&c) = self.chars.peek() {
                        self.advance();
                        if c == '\'' {
                            if self.chars.peek() == Some(&'\'') {
                                value.push(c);
                                self.advance();
                            } else {
                                found_end = true;
                                break;
                            }
                        } else {
                            value.push(c);
                        }
                    }
                    if found_end {
                        Some(Token::Literal(LiteralToken::String(value)))
                    } else {
                        Some(Token::Illegal(self.pos, value))
                    }
                }
                // a quoted identifier
                '"' => {
                    self.advance();
                    let mut value: String = String::new();
                    let mut found_end: bool = false;
                    while let Some(&c) = self.chars.peek() {
                        self.advance();
                        if c == '"' {
                            if self.chars.peek() == Some(&'"') {
                                value.push(c);
                                self.advance();
                            } else {
                                found_end = true;
                                break;
                            }
                        } else {
                            value.push(c);
                        }
                    }
                    if found_end {
                        Some(Token::QuotedIdentifier(value))
                    } else {
                        Some(Token::Illegal(self.pos, value))
                    }
                }
                // an operator that starts with a < sign
                '<' => {
                    self.advance();
                    match self.chars.peek() {
                        Some(&'=') => {
                            self.advance();
                            Some(Token::Operator(Operator::Lte))
                        }
                        Some(&'>') => {
                            self.advance();
                            Some(Token::Operator(Operator::NotEq))
                        }
                        Some(&' ') | None => Some(Token::Operator(Operator::Lt)),
                        Some(&c) => {
                            self.advance();
                            Some(Token::Illegal(self.pos, c.into()))
                        }
                    }
                }
                // an operator that starts with a > sign
                '>' => {
                    self.advance();
                    match self.chars.peek() {
                        Some(&'=') => {
                            self.advance();
                            Some(Token::Operator(Operator::Gte))
                        }
                        Some(&' ') | None => Some(Token::Operator(Operator::Gt)),
                        Some(&c) => {
                            self.advance();
                            Some(Token::Illegal(self.pos, c.into()))
                        }
                    }
                }
                // the = operator
                '=' => {
                    self.advance();
                    Some(Token::Operator(Operator::Equals))
                }
                // an operator that starts with a !
                '!' => {
                    self.advance();
                    match self.chars.peek() {
                        Some(&'=') => {
                            self.advance();
                            Some(Token::Operator(Operator::NotEq))
                        }
                        // TODO: This should be an error, as we do expect a following token
                        None => None,
                        Some(&c) => Some(Token::Illegal(self.pos, c.into())),
                    }
                }
                // an operator that starts with a + sign
                '+' => {
                    self.advance();
                    Some(Token::Operator(Operator::Plus))
                }
                // an operator that starts with a - sign
                '-' => {
                    self.advance();
                    match self.chars.peek() {
                        Some(&'-') => {
                            self.advance();
                            while let Some(&c) = self.chars.peek() {
                                if c == '\n' || c == '\r' {
                                    break;
                                }
                                self.advance();
                            }
                            self.get_next_token()
                        }
                        _ => Some(Token::Operator(Operator::Minus)),
                    }
                }
                // a * operator
                '*' => {
                    self.advance();
                    Some(Token::Operator(Operator::Star))
                }
                // an operator that starts with a / sign, or some kind of comment
                '/' => {
                    self.advance();
                    match self.chars.peek() {
                        // skip block comments
                        Some(&'*') => {
                            self.advance();
                            loop {
                                match self.chars.peek() {
                                    Some(&'*') => {
                                        self.advance();
                                        if self.chars.peek() == Some(&'/') {
                                            self.advance();
                                            break;
                                        }
                                    }
                                    None => {
                                        break;
                                    }
                                    _ => {
                                        self.advance();
                                    }
                                }
                            }
                            self.get_next_token()
                        }
                        _ => Some(Token::Operator(Operator::Divide)),
                    }
                }
                // Modulus
                '%' => {
                    self.advance();
                    Some(Token::Operator(Operator::Modulus))
                }
                // semicolon
                ';' => {
                    self.advance();
                    Some(Token::SemiColon)
                }
                // left parenthesis
                '(' => {
                    self.advance();
                    Some(Token::LParen)
                }
                // right parenthesis
                ')' => {
                    self.advance();
                    Some(Token::RParen)
                }
                // comma
                ',' => {
                    self.advance();
                    Some(Token::Comma)
                }
                // pow op
                '^' => {
                    self.advance();
                    Some(Token::Operator(Operator::Pow))
                }
                // anything else
                c => {
                    self.advance();
                    Some(Token::Unknown(self.pos, c.into()))
                }
            },
            None => None,
        }
    }

    /// Turns a char position into a line and column number of the underlying input string
    ///
    /// If the position is out of bounds, returns None, otherwise returns the line and column number
    fn char_pos_to_line_col(&mut self, pos: usize) -> Option<(usize, usize)> {
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
        if let Some(c) = self.chars.next() {
            self.pos += 1;
            Some(c)
        } else {
            None
        }
    }
}
