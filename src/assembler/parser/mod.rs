use std::{collections::HashMap, iter::Peekable, slice::Iter};

use crate::{
    assembler::{
        parser::error::ParseError::{self, IllegalToken},
        tokenizer::token::{Keyword, LiteralToken, Operator, Token},
    },
    code::VMInstruction,
    vm::Value,
};

pub mod error;

#[derive(Debug)]
pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
    pos: usize,
    label_map: HashMap<String, usize>,
    /// A list of locations in the instruction list that need to be resolved
    fixup_list: HashMap<String, Vec<usize>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens: tokens.iter().peekable(),
            pos: 0,
            label_map: HashMap::new(),
            fixup_list: HashMap::new(),
        }
    }

    /// Consumes this parser and returns a list of instructions, as well as the address of the main label
    pub fn parse(mut self) -> Result<(Vec<VMInstruction>, usize), ParseError> {
        let mut instructions: Vec<VMInstruction> = Vec::new();
        while let Some(&tok) = self.peek() {
            match tok {
                Token::NewLine => {
                    self.advance();
                }
                Token::Colon => {
                    self.advance();
                    match self.advance() {
                        Some(Token::Identifier(ident)) => {
                            if self
                                .label_map
                                .insert(ident.clone(), instructions.len())
                                .is_some()
                            {
                                return Err(ParseError::LabelAlreadyDefined(ident.clone()));
                            }
                            if let Some(positions) = self.fixup_list.remove(ident) {
                                let addr = instructions.len() as u32;
                                for pos in positions {
                                    match instructions.get_mut(pos) {
                                        Some(instr) => match instr {
                                            VMInstruction::Jump(old_addr) => *old_addr = addr,
                                            VMInstruction::JumpIfNotZero(old_addr) => {
                                                *old_addr = addr
                                            }
                                            VMInstruction::JumpIfZero(old_addr) => *old_addr = addr,
                                            VMInstruction::JumpIfTrue(old_addr) => *old_addr = addr,
                                            VMInstruction::JumpIfFalse(old_addr) => {
                                                *old_addr = addr
                                            }
                                            VMInstruction::Call(old_addr) => *old_addr = addr,
                                            _ => panic!(
                                                "Internal error: stored fixup index is not a jump instruction"
                                            ),
                                        },
                                        None => panic!(""),
                                    };
                                }
                            }
                        }
                        Some(t) => return Err(IllegalToken(t.clone(), self.pos)),
                        None => return Err(ParseError::UnexpectedEOF),
                    }
                }
                Token::Keyword(_) => {
                    let instr = self.parse_single_instruction(instructions.len())?;
                    instructions.push(instr);
                }
                t => return Err(ParseError::IllegalToken(t.clone(), self.pos)),
            }
        }

        if !self.fixup_list.is_empty() {
            return Err(ParseError::UndefinedLabels(
                self.fixup_list.into_keys().collect(),
            ));
        }

        match self.label_map.get("main") {
            Some(addr) => Ok((instructions, *addr)),
            None => Err(ParseError::NoMainLabel),
        }
    }

    /// Parses a single instruction
    fn parse_single_instruction(
        &mut self,
        cur_instruction_idx: usize,
    ) -> Result<VMInstruction, ParseError> {
        match self.advance() {
            Some(tok) => match tok {
                Token::Keyword(keyword) => match keyword {
                    Keyword::Push => match self.advance() {
                        Some(Token::Literal(LiteralToken::Int(i))) => {
                            Ok(VMInstruction::PushImm(Value::Int(*i)))
                        }
                        Some(Token::Operator(Operator::Minus)) => match self.advance() {
                            Some(Token::Literal(LiteralToken::Int(i))) => {
                                Ok(VMInstruction::PushImm(Value::Int(-*i)))
                            }
                            Some(t) => Err(IllegalToken(t.clone(), self.pos)),
                            None => Err(ParseError::UnexpectedEOF),
                        },
                        Some(t) => Err(IllegalToken(t.clone(), self.pos)),
                        None => Err(ParseError::UnexpectedEOF),
                    },
                    Keyword::Addu => Ok(VMInstruction::AddUnchecked),
                    Keyword::Addc => Ok(VMInstruction::AddChecked),
                    Keyword::Subu => Ok(VMInstruction::SubUnchecked),
                    Keyword::Subc => Ok(VMInstruction::SubChecked),
                    Keyword::Mulu => Ok(VMInstruction::MulUnchecked),
                    Keyword::Mulc => Ok(VMInstruction::MulChecked),
                    Keyword::Divc => Ok(VMInstruction::DivChecked),
                    Keyword::Negu => Ok(VMInstruction::NegUnchecked),
                    Keyword::Negc => Ok(VMInstruction::NegChecked),
                    Keyword::Shl => match self.advance() {
                        Some(Token::Literal(LiteralToken::Int(i))) => {
                            Ok(VMInstruction::ShiftL(*i as u8))
                        }
                        Some(t) => Err(IllegalToken(t.clone(), self.pos)),
                        None => Err(ParseError::UnexpectedEOF),
                    },
                    Keyword::Shr => match self.advance() {
                        Some(Token::Literal(LiteralToken::Int(i))) => {
                            Ok(VMInstruction::ShiftR(*i as u8))
                        }
                        Some(t) => Err(IllegalToken(t.clone(), self.pos)),
                        None => Err(ParseError::UnexpectedEOF),
                    },
                    Keyword::Rotl => match self.advance() {
                        Some(Token::Literal(LiteralToken::Int(i))) => {
                            Ok(VMInstruction::RotL(*i as u8))
                        }
                        Some(t) => Err(IllegalToken(t.clone(), self.pos)),
                        None => Err(ParseError::UnexpectedEOF),
                    },
                    Keyword::Rotr => match self.advance() {
                        Some(Token::Literal(LiteralToken::Int(i))) => {
                            Ok(VMInstruction::RotR(*i as u8))
                        }
                        Some(t) => Err(IllegalToken(t.clone(), self.pos)),
                        None => Err(ParseError::UnexpectedEOF),
                    },
                    Keyword::Eq => Ok(VMInstruction::Eq),
                    Keyword::Neq => Ok(VMInstruction::Neq),
                    Keyword::Lt => Ok(VMInstruction::Lt),
                    Keyword::Gt => Ok(VMInstruction::Gt),
                    Keyword::Le => Ok(VMInstruction::LtEq),
                    Keyword::Ge => Ok(VMInstruction::GtEq),
                    Keyword::Swap => Ok(VMInstruction::Swap),
                    Keyword::Pop => Ok(VMInstruction::Pop),
                    Keyword::Dup => Ok(VMInstruction::Dup),
                    Keyword::PopJump => Ok(VMInstruction::PopJump),
                    Keyword::Jump => match self.advance() {
                        Some(Token::Identifier(loc)) => match self.label_map.get(loc) {
                            Some(addr) => Ok(VMInstruction::Jump(*addr as u32)),
                            None => {
                                self.fixup_list
                                    .entry(loc.clone())
                                    .or_insert(Vec::new())
                                    .push(cur_instruction_idx);
                                // temp store an invalid address
                                Ok(VMInstruction::Jump(-1i32 as u32))
                            }
                        },
                        Some(Token::Literal(LiteralToken::Int(i))) => {
                            Ok(VMInstruction::Jump(*i as u32))
                        }
                        None => Err(ParseError::UnexpectedEOF),
                        _ => Err(IllegalToken(tok.clone(), self.pos)),
                    },
                    Keyword::JumpIfNotZero => match self.advance() {
                        Some(Token::Identifier(loc)) => match self.label_map.get(loc) {
                            Some(addr) => Ok(VMInstruction::JumpIfNotZero(*addr as u32)),
                            None => {
                                self.fixup_list
                                    .entry(loc.clone())
                                    .or_insert(Vec::new())
                                    .push(cur_instruction_idx);
                                // temp store an invalid address
                                Ok(VMInstruction::JumpIfNotZero(-1i32 as u32))
                            }
                        },
                        Some(Token::Literal(LiteralToken::Int(i))) => {
                            Ok(VMInstruction::JumpIfNotZero(*i as u32))
                        }
                        None => Err(ParseError::UnexpectedEOF),
                        _ => Err(IllegalToken(tok.clone(), self.pos)),
                    },
                    Keyword::JumpIfZero => match self.advance() {
                        Some(Token::Identifier(loc)) => match self.label_map.get(loc) {
                            Some(addr) => Ok(VMInstruction::JumpIfZero(*addr as u32)),
                            None => {
                                self.fixup_list
                                    .entry(loc.clone())
                                    .or_insert(Vec::new())
                                    .push(cur_instruction_idx);
                                // temp store an invalid address
                                Ok(VMInstruction::JumpIfZero(-1i32 as u32))
                            }
                        },
                        Some(Token::Literal(LiteralToken::Int(i))) => {
                            Ok(VMInstruction::JumpIfZero(*i as u32))
                        }
                        None => Err(ParseError::UnexpectedEOF),
                        _ => Err(IllegalToken(tok.clone(), self.pos)),
                    },
                    Keyword::JumpIfTrue => match self.advance() {
                        Some(Token::Identifier(loc)) => match self.label_map.get(loc) {
                            Some(addr) => Ok(VMInstruction::JumpIfTrue(*addr as u32)),
                            None => {
                                self.fixup_list
                                    .entry(loc.clone())
                                    .or_insert(Vec::new())
                                    .push(cur_instruction_idx);
                                // temp store an invalid address
                                Ok(VMInstruction::JumpIfTrue(-1i32 as u32))
                            }
                        },
                        Some(Token::Literal(LiteralToken::Int(i))) => {
                            Ok(VMInstruction::JumpIfTrue(*i as u32))
                        }
                        None => Err(ParseError::UnexpectedEOF),
                        _ => Err(IllegalToken(tok.clone(), self.pos)),
                    },
                    Keyword::JumpIfFalse => match self.advance() {
                        Some(Token::Identifier(loc)) => match self.label_map.get(loc) {
                            Some(addr) => Ok(VMInstruction::JumpIfFalse(*addr as u32)),
                            None => {
                                self.fixup_list
                                    .entry(loc.clone())
                                    .or_insert(Vec::new())
                                    .push(cur_instruction_idx);
                                // temp store an invalid address
                                Ok(VMInstruction::JumpIfFalse(-1i32 as u32))
                            }
                        },
                        Some(Token::Literal(LiteralToken::Int(i))) => {
                            Ok(VMInstruction::JumpIfFalse(*i as u32))
                        }
                        None => Err(ParseError::UnexpectedEOF),
                        _ => Err(IllegalToken(tok.clone(), self.pos)),
                    },
                    Keyword::DebugStack => Ok(VMInstruction::DebugStack),
                    Keyword::Exit => Ok(VMInstruction::Exit),
                    Keyword::ClearEFlags => Ok(VMInstruction::ClearErrorFlags),
                    Keyword::ClearDivZero => Ok(VMInstruction::ClearDivisionByZero),
                    Keyword::ClearOverflow => Ok(VMInstruction::ClearOverflow),
                    Keyword::PushDivisionByZeroFlag => Ok(VMInstruction::PushDivisionByZeroFlag),
                    Keyword::PushOverflowFlag => Ok(VMInstruction::PushOverflowFlag),
                    Keyword::Call => match self.advance() {
                        Some(Token::Identifier(loc)) => {
                            match self.label_map.get(loc) {
                                Some(addr) => Ok(VMInstruction::Call(*addr as u32)),
                                None => {
                                    self.fixup_list
                                        .entry(loc.clone())
                                        .or_insert(Vec::new())
                                        .push(cur_instruction_idx);
                                    // temp store an invalid address
                                    Ok(VMInstruction::Call(-1i32 as u32))
                                }
                            }
                        }
                        None => Err(ParseError::UnexpectedEOF),
                        _ => Err(IllegalToken(tok.clone(), self.pos)),
                    },
                    Keyword::Ret => Ok(VMInstruction::Return),
                    Keyword::BitAnd => Ok(VMInstruction::BitAnd),
                    Keyword::BitOr => Ok(VMInstruction::BitOr),
                    Keyword::BitXor => Ok(VMInstruction::BitXor),
                    Keyword::BitNot => Ok(VMInstruction::BitNot),
                    _ => Err(ParseError::IllegalToken(tok.clone(), self.pos)),
                },
                _ => unreachable!(),
            },
            None => Err(ParseError::UnexpectedEOF),
        }
    }

    // =======
    // Helpers
    // =======

    fn peek(&mut self) -> Option<&&'a Token> {
        self.tokens.peek()
    }

    /// Consumes the next token in the stream and returns it while incrementing
    /// the position counter
    fn advance(&mut self) -> Option<&'a Token> {
        match self.tokens.next() {
            Some(t) => {
                self.pos += 1;
                Some(t)
            }
            None => None,
        }
    }
}
