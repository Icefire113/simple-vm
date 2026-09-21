use crate::code::VMInstruction;

#[derive(Debug, thiserror::Error)]
pub enum VMError {
    #[error("Invalid instruction pointer address")]
    InvalidPCAddress,

    #[error("Stack exhausted")]
    StackExhausted,

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Overflow")]
    Overflow,

    #[error("Expected stack value to be an integer")]
    ExpectedInt,

    #[error("Invalid shift amount")]
    InvalidShiftAmount,

    #[error("Expected stack value to be a boolean")]
    ExpectedBool,

    #[error("Mismatched operands")]
    MismatchedOperands,
}

#[derive(Debug, Default)]
pub struct ErrorFlags {
    division_by_zero: bool,
    overflow: bool,
}

impl ErrorFlags {
    /// Returns whether an arithmetic overflow has occurred since the flags were last cleared
    pub fn overflow(&self) -> bool {
        self.overflow
    }

    /// Returns whether a division by zero has been attempted since the flags were last cleared
    pub fn division_by_zero(&self) -> bool {
        self.division_by_zero
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    Int(i32),
    Bool(bool),
}

impl PartialOrd for Value {
    /// Compares two values, only if both are integers
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

#[derive(Debug)]
pub struct VM {
    stack: Vec<Value>,
    program: Vec<VMInstruction>,
    pc: usize,
    error_flags: ErrorFlags,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            program: Vec::new(),
            pc: 0,
            error_flags: Default::default(),
        }
    }

    /// Loads a program into the virtual machine, overwriting any existing program and state
    pub fn load_program(&mut self, program: Vec<VMInstruction>) {
        self.stack.clear();
        self.error_flags = Default::default();
        self.program = program;
        self.pc = 0;
    }

    /// Returns the VM's error flags
    pub fn error_flags(&self) -> &ErrorFlags {
        &self.error_flags
    }

    /// Runs a single instruction, an `Err` return means that an error occurred when executing the instruction,
    /// and the `Ok(None)` return means that the program has not yet exited, and a `Ok(Some(x))` return means
    /// that the program has exited with exit code `x`
    pub fn single_step(&mut self) -> Result<Option<Value>, VMError> {
        // eprintln!("[VM] PC: {} | Instr: {:?}", self.pc, self.program[self.pc]);
        match self.program.get(self.pc) {
            Some(&value) => {
                match value {
                    VMInstruction::PushImm(i) => self.stack.push(i),
                    VMInstruction::AddUnchecked => {
                        let a: i32 = self.pop_checked_int()?;
                        let b: i32 = self.pop_checked_int()?;
                        let (r, overflowed) = a.overflowing_add(b);
                        if overflowed {
                            self.error_flags.overflow = true;
                        }
                        self.stack.push(r.into());
                    }
                    VMInstruction::AddChecked => {
                        let a: i32 = self.pop_checked_int()?;
                        let b: i32 = self.pop_checked_int()?;
                        match a.checked_add(b) {
                            Some(r) => self.stack.push(r.into()),
                            None => {
                                self.error_flags.overflow = true;
                                return Err(VMError::Overflow);
                            }
                        }
                    }
                    VMInstruction::SubUnchecked => {
                        let a: i32 = self.pop_checked_int()?;
                        let b: i32 = self.pop_checked_int()?;
                        let (r, overflowed) = a.overflowing_sub(b);
                        if overflowed {
                            self.error_flags.overflow = true;
                        }
                        self.stack.push(r.into());
                    }
                    VMInstruction::SubChecked => {
                        let a: i32 = self.pop_checked_int()?;
                        let b: i32 = self.pop_checked_int()?;
                        match a.checked_sub(b) {
                            Some(r) => self.stack.push(r.into()),
                            None => {
                                self.error_flags.overflow = true;
                                return Err(VMError::Overflow);
                            }
                        }
                    }
                    VMInstruction::MulUnchecked => {
                        let a: i32 = self.pop_checked_int()?;
                        let b: i32 = self.pop_checked_int()?;
                        let (r, overflowed) = a.overflowing_mul(b);
                        if overflowed {
                            self.error_flags.overflow = true;
                        }
                        self.stack.push(r.into());
                    }
                    VMInstruction::MulChecked => {
                        let a: i32 = self.pop_checked_int()?;
                        let b: i32 = self.pop_checked_int()?;
                        match a.checked_mul(b) {
                            Some(r) => self.stack.push(r.into()),
                            None => {
                                self.error_flags.overflow = true;
                                return Err(VMError::Overflow);
                            }
                        }
                    }
                    VMInstruction::DivChecked => {
                        let a: i32 = self.pop_checked_int()?;
                        let b: i32 = self.pop_checked_int()?;
                        if b == 0 {
                            self.error_flags.division_by_zero = true;
                            return Err(VMError::DivisionByZero);
                        }

                        self.stack.push((a / b).into());
                    }
                    VMInstruction::NegUnchecked => {
                        let a: i32 = self.pop_checked_int()?;
                        let (r, overflowed) = a.overflowing_neg();
                        if overflowed {
                            self.error_flags.overflow = true;
                        }
                        self.stack.push(r.into());
                    }
                    VMInstruction::NegChecked => {
                        let a: i32 = self.pop_checked_int()?;
                        match a.overflowing_neg() {
                            (r, false) => self.stack.push(r.into()),
                            (_, true) => {
                                self.error_flags.overflow = true;
                                return Err(VMError::Overflow);
                            }
                        }
                    }
                    VMInstruction::Exit => {
                        let a = self.pop_checked()?;
                        return Ok(Some(a));
                    }
                    VMInstruction::DebugStack => {
                        eprintln!("[VM] PC: {} | Stack Debug: {:?}", self.pc, self.stack);
                    }
                    VMInstruction::Swap => {
                        let a = self.pop_checked()?;
                        let b = self.pop_checked()?;
                        self.stack.push(a);
                        self.stack.push(b);
                    }
                    VMInstruction::Pop => {
                        self.stack.pop().ok_or(VMError::StackExhausted)?;
                    }
                    VMInstruction::Dup => {
                        let a = self.pop_checked()?;
                        self.stack.push(a);
                        self.stack.push(a);
                    }
                    VMInstruction::PopJump => {
                        let a = self.pop_checked_int()?;
                        self.pc = a as usize;
                        // skip pc increment
                        return Ok(None);
                    }
                    VMInstruction::Jump(addr) => {
                        self.pc = addr as usize;
                        // skip pc increment
                        return Ok(None);
                    }
                    VMInstruction::JumpIfNotZero(addr) => {
                        let a = self.pop_checked_int()?;
                        if a != 0 {
                            self.pc = addr as usize;
                            // skip pc increment
                            return Ok(None);
                        }
                    }
                    VMInstruction::JumpIfZero(addr) => {
                        let a = self.pop_checked_int()?;
                        if a == 0 {
                            self.pc = addr as usize;
                            // skip pc increment
                            return Ok(None);
                        }
                    }
                    VMInstruction::JumpIfTrue(addr) => {
                        let a = self.pop_checked_bool()?;
                        if a {
                            self.pc = addr as usize;
                            // skip pc increment
                            return Ok(None);
                        }
                    }
                    VMInstruction::JumpIfFalse(addr) => {
                        let a = self.pop_checked_bool()?;
                        if !a {
                            self.pc = addr as usize;
                            // skip pc increment
                            return Ok(None);
                        }
                    }
                    VMInstruction::Eq => {
                        let a = self.pop_checked()?;
                        let b = self.pop_checked()?;
                        self.stack.push((a == b).into());
                    }
                    VMInstruction::Lt => {
                        let a = self.pop_checked()?;
                        let b = self.pop_checked()?;
                        match a.partial_cmp(&b) {
                            Some(ord) => {
                                self.stack.push(ord.is_lt().into());
                            }
                            None => return Err(VMError::MismatchedOperands),
                        }
                    }
                    VMInstruction::Gt => {
                        let a = self.pop_checked()?;
                        let b = self.pop_checked()?;
                        match a.partial_cmp(&b) {
                            Some(ord) => {
                                self.stack.push(ord.is_gt().into());
                            }
                            None => return Err(VMError::MismatchedOperands),
                        }
                    }
                    VMInstruction::GtEq => {
                        let a = self.pop_checked()?;
                        let b = self.pop_checked()?;
                        match a.partial_cmp(&b) {
                            Some(ord) => {
                                self.stack.push(ord.is_ge().into());
                            }
                            None => return Err(VMError::MismatchedOperands),
                        }
                    }
                    VMInstruction::LtEq => {
                        let a = self.pop_checked()?;
                        let b = self.pop_checked()?;
                        match a.partial_cmp(&b) {
                            Some(ord) => {
                                self.stack.push(ord.is_le().into());
                            }
                            None => return Err(VMError::MismatchedOperands),
                        }
                    }
                    VMInstruction::Neq => {
                        let a = self.pop_checked()?;
                        let b = self.pop_checked()?;
                        self.stack.push((a != b).into());
                    }
                    VMInstruction::ShiftL(n) => {
                        let a = self.pop_checked_int()?;
                        let r = a.checked_shl(n as u32).ok_or(VMError::InvalidShiftAmount)?;
                        self.stack.push(r.into());
                    }
                    VMInstruction::ShiftR(n) => {
                        let a = self.pop_checked_int()?;
                        let r = a.checked_shr(n as u32).ok_or(VMError::InvalidShiftAmount)?;
                        self.stack.push(r.into());
                    }
                    VMInstruction::RotL(n) => {
                        let a = self.pop_checked_int()?;
                        self.stack.push(a.rotate_left(n as u32).into());
                    }
                    VMInstruction::RotR(n) => {
                        let a = self.pop_checked_int()?;
                        self.stack.push(a.rotate_right(n as u32).into());
                    }
                    VMInstruction::ClearErrorFlags => self.error_flags = Default::default(),
                    VMInstruction::ClearDivisionByZero => self.error_flags.division_by_zero = false,
                    VMInstruction::ClearOverflow => self.error_flags.overflow = false,
                    VMInstruction::PushDivisionByZeroFlag => {
                        self.stack.push(self.error_flags.division_by_zero.into())
                    }
                    VMInstruction::PushOverflowFlag => {
                        self.stack.push(self.error_flags.overflow.into())
                    }
                };
                self.pc += 1;
                Ok(None)
            }
            None => Err(VMError::InvalidPCAddress),
        }
    }

    fn pop_checked(&mut self) -> Result<Value, VMError> {
        self.stack.pop().ok_or(VMError::StackExhausted)
    }

    fn pop_checked_int(&mut self) -> Result<i32, VMError> {
        match self.stack.pop().ok_or(VMError::StackExhausted)? {
            Value::Int(n) => Ok(n),
            Value::Bool(_) => Err(VMError::ExpectedInt),
        }
    }

    fn pop_checked_bool(&mut self) -> Result<bool, VMError> {
        match self.stack.pop().ok_or(VMError::StackExhausted)? {
            Value::Bool(n) => Ok(n),
            Value::Int(_) => Err(VMError::ExpectedBool),
        }
    }

    /// Runs the virtual machine until the program exits or an error occurs
    pub fn run(&mut self) -> Result<Value, VMError> {
        let mut result: Option<Value> = self.single_step()?;
        while result.is_none() {
            result = self.single_step()?;
        }

        Ok(result.unwrap())
    }
}
