use std::collections::VecDeque;

use crate::code::VMInstruction;

#[derive(Debug, thiserror::Error)]
pub enum VMError {
    #[error("Program code is out of bounds")]
    ProgramCodeOutOfBounds,

    #[error("Stack exhausted")]
    StackExhausted,

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Overflow")]
    Overflow,
}

#[derive(Debug, Default)]
pub struct ErrorFlags {
    division_by_zero: bool,
    overflow: bool,
}

#[derive(Debug)]
pub struct VM {
    stack: Vec<i32>,
    program: VecDeque<VMInstruction>,
    error_flags: ErrorFlags,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            program: VecDeque::new(),
            error_flags: Default::default(),
        }
    }

    /// Loads a program into the virtual machine, overwriting any existing program
    pub fn load_program(&mut self, program: Vec<VMInstruction>) {
        self.program = VecDeque::from(program);
    }

    /// Runs a single instruction, an `Err` return means that an error occurred when executing the instruction,
    /// and the `Ok(None)` return means that the program has not yet exited, and a `Ok(Some(x))` return means
    /// that the program has exited with exit code `x`
    pub fn single_step(&mut self) -> Result<Option<i32>, VMError> {
        match self.program.pop_front() {
            Some(value) => {
                match value {
                    VMInstruction::PushImm(i) => self.stack.push(i),
                    VMInstruction::AddUnchecked => {
                        let a: i32 = self.stack.pop().ok_or(VMError::StackExhausted)?;
                        let b: i32 = self.stack.pop().ok_or(VMError::StackExhausted)?;
                        self.stack.push(a.wrapping_add(b));
                    }
                    VMInstruction::AddChecked => {
                        let a: i32 = self.stack.pop().ok_or(VMError::StackExhausted)?;
                        let b: i32 = self.stack.pop().ok_or(VMError::StackExhausted)?;
                        match a.checked_add(b) {
                            Some(r) => self.stack.push(r),
                            None => {
                                self.error_flags.overflow = true;
                                return Err(VMError::Overflow);
                            }
                        }
                    }
                    VMInstruction::Sub => {
                        let a: i32 = self.stack.pop().ok_or(VMError::StackExhausted)?;
                        let b: i32 = self.stack.pop().ok_or(VMError::StackExhausted)?;
                        self.stack.push(a - b);
                    }
                    VMInstruction::MulUnchecked => {
                        let a: i32 = self.stack.pop().ok_or(VMError::StackExhausted)?;
                        let b: i32 = self.stack.pop().ok_or(VMError::StackExhausted)?;
                        self.stack.push(a.wrapping_mul(b));
                    }
                    VMInstruction::MulChecked => {
                        let a: i32 = self.stack.pop().ok_or(VMError::StackExhausted)?;
                        let b: i32 = self.stack.pop().ok_or(VMError::StackExhausted)?;
                        match a.checked_mul(b) {
                            Some(r) => self.stack.push(r),
                            None => {
                                self.error_flags.overflow = true;
                                return Err(VMError::Overflow);
                            }
                        }
                    }
                    VMInstruction::DivChecked => {
                        let a: i32 = self.stack.pop().ok_or(VMError::StackExhausted)?;
                        let b: i32 = self.stack.pop().ok_or(VMError::StackExhausted)?;
                        if b == 0 {
                            self.error_flags.division_by_zero = true;
                            return Err(VMError::DivisionByZero);
                        }

                        self.stack.push(a / b);
                    }
                    VMInstruction::Neg => {
                        let a: i32 = self.stack.pop().ok_or(VMError::StackExhausted)?;
                        self.stack.push(a * -1);
                    }
                    VMInstruction::Exit => {
                        let a: i32 = self.stack.pop().ok_or(VMError::StackExhausted)?;
                        return Ok(Some(a));
                    }
                    VMInstruction::DebugStack => {
                        eprintln!("Stack Debug: {:?}", self.stack);
                    }
                    VMInstruction::Swap => {
                        let a: i32 = self.stack.pop().ok_or(VMError::StackExhausted)?;
                        let b: i32 = self.stack.pop().ok_or(VMError::StackExhausted)?;
                        self.stack.push(a);
                        self.stack.push(b);
                    }
                };
                Ok(None)
            }
            None => Err(VMError::ProgramCodeOutOfBounds),
        }
    }

    /// Runs the virtual machine until the program exits or an error occurs
    pub fn run(&mut self) -> Result<i32, VMError> {
        let mut result: Option<i32> = self.single_step()?;
        while result.is_none() {
            result = self.single_step()?;
        }

        Ok(result.unwrap())
    }
}
