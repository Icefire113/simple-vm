use crate::vm::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VMInstruction {
    // Arithmetic stuffs
    /// Pops 2 values from the stack and pushes their sum, wrapping on overflow and setting ErrorFlags.overflow if it occurs
    ///
    /// Given stack: [a, b], pushes [b + a]
    AddUnchecked,
    /// Pops 2 values from the stack and pushes their sum, acting as a runtime assert; raises an error and sets ErrorFlags.overflow if it overflows
    ///
    /// Given stack: [a, b], pushes [b + a]
    AddChecked,
    /// Pops 2 values from the stack and pushes their difference, wrapping on overflow and setting ErrorFlags.overflow if it occurs
    ///
    /// Given stack: [a, b], pushes [b - a]
    SubUnchecked,
    /// Pops 2 values from the stack and pushes their difference, acting as a runtime assert; raises an error and sets ErrorFlags.overflow if it overflows
    ///
    /// Given stack: [a, b], pushes [b - a]
    SubChecked,
    /// Pops 2 values from the stack and pushes their product, acting as a runtime assert; raises an error and sets ErrorFlags.overflow if it overflows
    MulChecked,
    /// Pops 2 values from the stack and pushes their product, wrapping on overflow and setting ErrorFlags.overflow if it occurs
    MulUnchecked,
    /// Pops 2 values from the stack and pushes their quotient, checking that the divisor is not zero setting ErrorFlags.division_by_zero if it is
    ///
    /// Given stack: [a, b], pushes [b / a]
    DivChecked,
    /// Pops a value from the stack and pushes its negation, wrapping on overflow and setting ErrorFlags.overflow if it occurs
    NegUnchecked,
    /// Pops a value from the stack and pushes its negation, acting as a runtime assert; raises an error and sets ErrorFlags.overflow if it overflows
    ///
    /// Note that this only applies to integer values, booleans will raise an error
    NegChecked,
    /// Pops a value from the stack and shifts it left by the given amount, then pushes the result
    ///
    /// Note that this only applies to integer values, booleans will raise an error.
    /// Raises an error if the shift amount is >= the number of bits in an `i32` (32).
    /// Note that this does *not* raise an error if bits are shifted out, only if the shift amount itself is invalid
    ShiftL(u8),
    /// Pops a value from the stack and shifts it right by the given amount, then pushes the result
    ///
    /// Note that this only applies to integer values, booleans will raise an error.
    /// Raises an error if the shift amount is >= the number of bits in an `i32` (32).
    /// This is an arithmetic shift, so the sign bit is copied in for negative values
    ShiftR(u8),
    /// Pops a value from the stack and rotates its bits left by the given amount, then pushes the result
    ///
    /// Note that this only applies to integer values, booleans will raise an error.
    /// The shift amount is taken modulo 32, e.g. a rotation of 33 behaves as a rotation of 1
    RotL(u8),
    /// Pops a value from the stack and rotates its bits right by the given amount, then pushes the result
    ///
    /// Note that this only applies to integer values, booleans will raise an error.
    /// The shift amount is taken modulo 32, e.g. a rotation of 33 behaves as a rotation of 1
    RotR(u8),

    /// Pops values `a, b` from the stack and pushes their bitwise `and`
    ///
    /// Both operands must be the same type; ints are combined bitwise and bools logically,
    /// mixed types raise an error
    BitAnd,
    /// Pops values `a, b` from the stack and pushes their bitwise `or`
    ///
    /// Both operands must be the same type; ints are combined bitwise and bools logically,
    /// mixed types raise an error
    BitOr,
    /// Pops values `a, b` from the stack and pushes their bitwise `xor`
    ///
    /// Both operands must be the same type; ints are combined bitwise and bools logically,
    /// mixed types raise an error
    BitXor,
    /// Pops a value `a` from the stack and pushes `!a`
    ///
    /// Ints are complemented bitwise and bools are negated logically
    BitNot,

    // Comparison stuffs
    /// Pops 2 values from the stack and pushes their equality (true or false)
    ///
    /// Note that equality does also check that the types are the same, so `Value::Bool(true) == Value::Int(1)` is false
    Eq,
    /// Pops 2 values from the stack and pushes their not equal (true or false)
    ///
    /// Note that equality does also check that the types are the same, so `Value::Bool(true) == Value::Int(1)` is false
    Neq,
    /// Pops 2 values from the stack and pushes their less than (true or false)
    ///
    /// Note that types are checked here, so a comparison between a `Value::Int` and a `Value::Bool` can raise an error
    ///
    /// Given stack: [a, b], pushes [b < a]
    Lt,
    /// Pops 2 values from the stack and pushes their greater than (true or false)
    ///
    /// Note that types are checked here, so a comparison between a `Value::Int` and a `Value::Bool` can raise an error
    ///
    /// Given stack: [a, b], pushes [b > a]
    Gt,
    /// Pops 2 values from the stack and pushes their greater than or equal (true or false)
    ///
    /// Note that types are checked here, so a comparison between a `Value::Int` and a `Value::Bool` can raise an error
    ///
    /// Given stack: [a, b], pushes [b >= a]
    GtEq,
    /// Pops 2 values from the stack and pushes their less than or equal (true or false)
    ///
    /// Note that types are checked here, so a comparison between a `Value::Int` and a `Value::Bool` can raise an error
    ///
    /// Given stack: [a, b], pushes [b >= a]
    LtEq,

    // stack operations
    /// Pops the top two values from the stack, swaps them, then pushes them back
    Swap,
    /// Pops a value from the stack and discards it
    Pop,
    /// Pops a value from the stack, then pushes 2 copies of that value back onto the stack
    Dup,
    /// Push a 32bit signed integer immediate value onto the stack
    PushImm(Value),
    /// Copies the nth item in the stack to the top, so if our stack is `[1,2,3]` and we `pick 1` we should end up with `[1,2,3,2]`
    /// `pick 0` is the same as `dup`
    /// `pick 2` would result in `[1,2,3,1]`
    Pick(u32),
    /// Same as `pick`, but moves the value instead of copying
    Move(u32),

    // control flow stuffs
    /// Pops a value from the stack and sets the program counter to that value
    PopJump,
    /// Sets the program counter to the given value
    Jump(u32),

    /// Pops a value from the stack and jumps to the given value if the value is not zero
    JumpIfNotZero(u32),
    /// Pops a value from the stack and jumps to the given value if the value is zero
    JumpIfZero(u32),

    /// Pops a value from the stack and jumps to the given value if the value is true
    JumpIfTrue(u32),
    /// Pops a value from the stack and jumps to the given value if the value is false
    JumpIfFalse(u32),

    /// Pushes PC + 1 onto the return stack, then jumps to the given address
    Call(u32),
    /// Pops a value from the return stack and jumps to that value
    Return,

    // VM control stuffs
    /// Debug instruction, prints the current state of the stack to stderr
    DebugStack,
    /// Pops a value from the stack and treats that value as the VM's "exit code"
    Exit,
    /// Clears all the error flags
    ClearErrorFlags,
    /// Clears the division by zero error flag
    ClearDivisionByZero,
    /// Clears the overflow error flag
    ClearOverflow,

    /// Pushes the division by zero flag as a boolean
    PushDivisionByZeroFlag,
    /// Pushes the overflow flag as a boolean
    PushOverflowFlag,
}

impl VMInstruction {
    const fn get_opcode(&self) -> [u8; 4] {
        match self {
            // arithmetic
            Self::AddUnchecked => [0x10, 0x01, 0, 0],
            Self::AddChecked => [0x10, 0x11, 0, 0],
            Self::SubUnchecked => [0x10, 0x02, 0, 0],
            Self::SubChecked => [0x10, 0x12, 0, 0],
            Self::MulChecked => [0x10, 0x13, 0, 0],
            Self::MulUnchecked => [0x10, 0x03, 0, 0],
            Self::DivChecked => [0x10, 0x14, 0, 0],
            Self::NegUnchecked => [0x10, 0x05, 0, 0],
            Self::NegChecked => [0x10, 0x15, 0, 0],
            // bit twiddling
            Self::ShiftL(_) => [0x03, 0x00, 0, 0],
            Self::ShiftR(_) => [0x03, 0x01, 0, 0],
            Self::RotL(_) => [0x03, 0x02, 0, 0],
            Self::RotR(_) => [0x03, 0x03, 0, 0],
            Self::BitAnd => [0x03, 0x04, 0, 0],
            Self::BitOr => [0x03, 0x05, 0, 0],
            Self::BitXor => [0x03, 0x06, 0, 0],
            Self::BitNot => [0x03, 0x07, 0, 0],
            // comparison
            Self::Eq => [0x04, 0x00, 0, 0],
            Self::Neq => [0x04, 0x01, 0, 0],
            Self::Lt => [0x04, 0x02, 0, 0],
            Self::Gt => [0x04, 0x03, 0, 0],
            Self::GtEq => [0x04, 0x04, 0, 0],
            Self::LtEq => [0x04, 0x05, 0, 0],
            // stack twidling
            Self::Swap => [0x06, 0x00, 0, 0],
            Self::Pop => [0x06, 0x01, 0, 0],
            Self::Dup => [0x06, 0x02, 0, 0],
            Self::PushImm(_) => [0x06, 0x13, 0, 0],
            Self::Pick(_) => [0x06, 0x14, 0, 0],
            Self::Move(_) => [0x06, 0x15, 0, 0],
            // control flow
            Self::PopJump => [0x0F, 0x00, 0, 0],
            Self::Jump(_) => [0x0F, 0x01, 0, 0],
            Self::JumpIfNotZero(_) => [0x0F, 0x02, 0, 0],
            Self::JumpIfZero(_) => [0x0F, 0x03, 0, 0],
            Self::JumpIfTrue(_) => [0x0F, 0x04, 0, 0],
            Self::JumpIfFalse(_) => [0x0F, 0x05, 0, 0],
            Self::Call(_) => [0x0F, 0x06, 0, 0],
            Self::Return => [0x0F, 0xFF, 0, 0],
            // error checking
            Self::ClearErrorFlags => [0x20, 0x00, 0, 0],
            Self::ClearDivisionByZero => [0x20, 0x01, 0, 0],
            Self::ClearOverflow => [0x20, 0x02, 0, 0],
            Self::PushDivisionByZeroFlag => [0x20, 0x03, 0, 0],
            Self::PushOverflowFlag => [0x20, 0x04, 0, 0],
            // system
            Self::Exit => [0xFF, 0x00, 0, 0],
            Self::DebugStack => [0xFF, 0x01, 0, 0],
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let opcode: [u8; 4] = self.get_opcode();
        // overallocate a small amount
        let mut v: Vec<u8> = Vec::with_capacity(opcode.len() + 5);
        v.extend(opcode);

        // extend opcode with arguments
        // DO NOT FORGET TO ADD ANY NEW INSUTRCTIONS THAT HAVE ARGS HERE
        match self {
            Self::ShiftL(n) => v.push(*n),
            Self::ShiftR(n) => v.push(*n),
            Self::RotL(n) => v.push(*n),
            Self::RotR(n) => v.push(*n),
            Self::PushImm(value) => v.extend(value.as_bytes()),
            Self::Pick(n) => v.extend(n.to_le_bytes()),
            Self::Move(n) => v.extend(n.to_le_bytes()),
            Self::Jump(a) => v.extend(a.to_le_bytes()),
            Self::JumpIfNotZero(a) => v.extend(a.to_le_bytes()),
            Self::JumpIfZero(a) => v.extend(a.to_le_bytes()),
            Self::JumpIfTrue(a) => v.extend(a.to_le_bytes()),
            Self::JumpIfFalse(a) => v.extend(a.to_le_bytes()),
            Self::Call(a) => v.extend(a.to_le_bytes()),
            // everything else does not have any arguments
            _ => {}
        };
        v
    }

    /// Tries to constrct an instruction from a byte slice, if successful returns the
    /// instruction and the number of bytes that instruction took
    pub fn try_from_bytes(bytes: &[u8]) -> Result<(Self, u8), VMInstructionParseError> {
        if bytes.len() < 4 {
            return Err(VMInstructionParseError::TooShort);
        }
        // inverse of Self::get_opcode
        match bytes[0..4] {
            [0x10, 0x01, 0, 0] => Ok((Self::AddUnchecked, 4)),
            [0x10, 0x11, 0, 0] => Ok((Self::AddChecked, 4)),
            [0x10, 0x02, 0, 0] => Ok((Self::SubUnchecked, 4)),
            [0x10, 0x12, 0, 0] => Ok((Self::SubChecked, 4)),
            [0x10, 0x13, 0, 0] => Ok((Self::MulChecked, 4)),
            [0x10, 0x03, 0, 0] => Ok((Self::MulUnchecked, 4)),
            [0x10, 0x14, 0, 0] => Ok((Self::DivChecked, 4)),
            [0x10, 0x05, 0, 0] => Ok((Self::NegUnchecked, 4)),
            [0x10, 0x15, 0, 0] => Ok((Self::NegChecked, 4)),
            [0x03, 0x00, 0, 0] => {
                if bytes.len() < 5 {
                    Err(VMInstructionParseError::TooShort)
                } else {
                    Ok((Self::ShiftL(bytes[4]), 5))
                }
            }
            [0x03, 0x01, 0, 0] => {
                if bytes.len() < 5 {
                    Err(VMInstructionParseError::TooShort)
                } else {
                    Ok((Self::ShiftR(bytes[4]), 5))
                }
            }
            [0x03, 0x02, 0, 0] => {
                if bytes.len() < 5 {
                    Err(VMInstructionParseError::TooShort)
                } else {
                    Ok((Self::RotL(bytes[4]), 5))
                }
            }
            [0x03, 0x03, 0, 0] => {
                if bytes.len() < 5 {
                    Err(VMInstructionParseError::TooShort)
                } else {
                    Ok((Self::RotR(bytes[4]), 5))
                }
            }
            [0x03, 0x04, 0, 0] => Ok((Self::BitAnd, 4)),
            [0x03, 0x05, 0, 0] => Ok((Self::BitOr, 4)),
            [0x03, 0x06, 0, 0] => Ok((Self::BitXor, 4)),
            [0x03, 0x07, 0, 0] => Ok((Self::BitNot, 4)),
            [0x04, 0x00, 0, 0] => Ok((Self::Eq, 4)),
            [0x04, 0x01, 0, 0] => Ok((Self::Neq, 4)),
            [0x04, 0x02, 0, 0] => Ok((Self::Lt, 4)),
            [0x04, 0x03, 0, 0] => Ok((Self::Gt, 4)),
            [0x04, 0x04, 0, 0] => Ok((Self::GtEq, 4)),
            [0x04, 0x05, 0, 0] => Ok((Self::LtEq, 4)),
            [0x06, 0x00, 0, 0] => Ok((Self::Swap, 4)),
            [0x06, 0x01, 0, 0] => Ok((Self::Pop, 4)),
            [0x06, 0x02, 0, 0] => Ok((Self::Dup, 4)),
            [0x06, 0x13, 0, 0] => {
                if bytes.len() < 9 {
                    Err(VMInstructionParseError::TooShort)
                } else {
                    match Value::try_from_bytes(&bytes[4..9]) {
                        Some(value) => Ok((Self::PushImm(value), 9)),
                        None => Err(VMInstructionParseError::InvalidValue),
                    }
                }
            }
            [0x06, 0x14, 0, 0] => {
                if bytes.len() < 8 {
                    Err(VMInstructionParseError::TooShort)
                } else {
                    Ok((
                        Self::Pick(u32::from_le_bytes(
                            bytes[4..8].try_into().expect("Internal error"),
                        )),
                        8,
                    ))
                }
            }
            [0x06, 0x15, 0, 0] => {
                if bytes.len() < 8 {
                    Err(VMInstructionParseError::TooShort)
                } else {
                    Ok((
                        Self::Move(u32::from_le_bytes(
                            bytes[4..8].try_into().expect("Internal error"),
                        )),
                        8,
                    ))
                }
            }
            [0x0F, 0x00, 0, 0] => Ok((Self::PopJump, 4)),
            [0x0F, 0x01, 0, 0] => {
                if bytes.len() < 8 {
                    Err(VMInstructionParseError::TooShort)
                } else {
                    Ok((
                        Self::Jump(u32::from_le_bytes(
                            bytes[4..8].try_into().expect("Internal error"),
                        )),
                        8,
                    ))
                }
            }
            [0x0F, 0x02, 0, 0] => {
                if bytes.len() < 8 {
                    Err(VMInstructionParseError::TooShort)
                } else {
                    Ok((
                        Self::JumpIfNotZero(u32::from_le_bytes(
                            bytes[4..8].try_into().expect("Internal error"),
                        )),
                        8,
                    ))
                }
            }
            [0x0F, 0x03, 0, 0] => {
                if bytes.len() < 8 {
                    Err(VMInstructionParseError::TooShort)
                } else {
                    Ok((
                        Self::JumpIfZero(u32::from_le_bytes(
                            bytes[4..8].try_into().expect("Internal error"),
                        )),
                        8,
                    ))
                }
            }
            [0x0F, 0x04, 0, 0] => {
                if bytes.len() < 8 {
                    Err(VMInstructionParseError::TooShort)
                } else {
                    Ok((
                        Self::JumpIfTrue(u32::from_le_bytes(
                            bytes[4..8].try_into().expect("Internal error"),
                        )),
                        8,
                    ))
                }
            }
            [0x0F, 0x05, 0, 0] => {
                if bytes.len() < 8 {
                    Err(VMInstructionParseError::TooShort)
                } else {
                    Ok((
                        Self::JumpIfFalse(u32::from_le_bytes(
                            bytes[4..8].try_into().expect("Internal error"),
                        )),
                        8,
                    ))
                }
            }
            [0x0F, 0x06, 0, 0] => {
                if bytes.len() < 8 {
                    Err(VMInstructionParseError::TooShort)
                } else {
                    Ok((
                        Self::Call(u32::from_le_bytes(
                            bytes[4..8].try_into().expect("Internal error"),
                        )),
                        8,
                    ))
                }
            }
            [0x0F, 0xFF, 0, 0] => Ok((Self::Return, 4)),
            [0x20, 0x00, 0, 0] => Ok((Self::ClearErrorFlags, 4)),
            [0x20, 0x01, 0, 0] => Ok((Self::ClearDivisionByZero, 4)),
            [0x20, 0x02, 0, 0] => Ok((Self::ClearOverflow, 4)),
            [0x20, 0x03, 0, 0] => Ok((Self::PushDivisionByZeroFlag, 4)),
            [0x20, 0x04, 0, 0] => Ok((Self::PushOverflowFlag, 4)),
            [0xFF, 0x00, 0, 0] => Ok((Self::Exit, 4)),
            [0xFF, 0x01, 0, 0] => Ok((Self::DebugStack, 4)),
            _ => Err(VMInstructionParseError::InvalidOpcode),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VMInstructionParseError {
    #[error("Not enough bytes")]
    TooShort,

    #[error("Invalid opcode")]
    InvalidOpcode,

    #[error("Invalid value")]
    InvalidValue,
}
