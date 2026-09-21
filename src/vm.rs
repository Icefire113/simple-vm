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
}

#[derive(Debug, Default)]
pub struct ErrorFlags {
    division_by_zero: bool,
    overflow: bool,
}

#[derive(Debug)]
pub struct VM {
    stack: Vec<i32>,
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

    /// Loads a program into the virtual machine, overwriting any existing program
    pub fn load_program(&mut self, program: Vec<VMInstruction>) {
        self.program = program;
        self.pc = 0;
    }

    /// Runs a single instruction, an `Err` return means that an error occurred when executing the instruction,
    /// and the `Ok(None)` return means that the program has not yet exited, and a `Ok(Some(x))` return means
    /// that the program has exited with exit code `x`
    pub fn single_step(&mut self) -> Result<Option<i32>, VMError> {
        match self.program.get(self.pc) {
            Some(&value) => {
                match value {
                    VMInstruction::PushImm(i) => self.stack.push(i),
                    VMInstruction::AddUnchecked => {
                        let a: i32 = self.pop_checked()?;
                        let b: i32 = self.pop_checked()?;
                        self.stack.push(a.wrapping_add(b));
                    }
                    VMInstruction::AddChecked => {
                        let a: i32 = self.pop_checked()?;
                        let b: i32 = self.pop_checked()?;
                        match a.checked_add(b) {
                            Some(r) => self.stack.push(r),
                            None => {
                                self.error_flags.overflow = true;
                                return Err(VMError::Overflow);
                            }
                        }
                    }
                    VMInstruction::Sub => {
                        let a: i32 = self.pop_checked()?;
                        let b: i32 = self.pop_checked()?;
                        self.stack.push(a - b);
                    }
                    VMInstruction::MulUnchecked => {
                        let a: i32 = self.pop_checked()?;
                        let b: i32 = self.pop_checked()?;
                        self.stack.push(a.wrapping_mul(b));
                    }
                    VMInstruction::MulChecked => {
                        let a: i32 = self.pop_checked()?;
                        let b: i32 = self.pop_checked()?;
                        match a.checked_mul(b) {
                            Some(r) => self.stack.push(r),
                            None => {
                                self.error_flags.overflow = true;
                                return Err(VMError::Overflow);
                            }
                        }
                    }
                    VMInstruction::DivChecked => {
                        let a: i32 = self.pop_checked()?;
                        let b: i32 = self.pop_checked()?;
                        if b == 0 {
                            self.error_flags.division_by_zero = true;
                            return Err(VMError::DivisionByZero);
                        }

                        self.stack.push(a / b);
                    }
                    VMInstruction::NegUnchecked => {
                        let a: i32 = self.pop_checked()?;
                        self.stack.push(a.overflowing_neg().0);
                    }
                    VMInstruction::NegChecked => {
                        let a: i32 = self.pop_checked()?;
                        match a.overflowing_neg() {
                            (r, false) => self.stack.push(r),
                            (r, true) => {
                                self.error_flags.overflow = true;
                                self.stack.push(r);
                            }
                        }
                    }
                    VMInstruction::Exit => {
                        let a: i32 = self.pop_checked()?;
                        return Ok(Some(a));
                    }
                    VMInstruction::DebugStack => {
                        eprintln!("Stack Debug: {:?}", self.stack);
                    }
                    VMInstruction::Swap => {
                        let a: i32 = self.pop_checked()?;
                        let b: i32 = self.pop_checked()?;
                        self.stack.push(a);
                        self.stack.push(b);
                    }
                    VMInstruction::Pop => {
                        self.stack.pop().ok_or(VMError::StackExhausted)?;
                    }
                    VMInstruction::PopJump => {
                        let a = self.pop_checked()?;
                        self.pc = a as usize;
                        // skip pc increment
                        return Ok(None);
                    }
                    VMInstruction::Jump(addr) => self.pc = addr as usize,
                    VMInstruction::JumpIfNotZero(addr) => {
                        let a = self.pop_checked()?;
                        if a != 0 {
                            self.pc = addr as usize;
                        }
                        // skip pc increment
                        return Ok(None);
                    }
                    VMInstruction::JumpIfZero(addr) => {
                        let a = self.pop_checked()?;
                        if a == 0 {
                            self.pc = addr as usize;
                        }
                        // skip pc increment
                        return Ok(None);
                    }
                    VMInstruction::Eq => {
                        let a = self.pop_checked()?;
                        let b = self.pop_checked()?;
                        self.stack.push(if a == b { 1 } else { 0 });
                    }
                    VMInstruction::Lt => {
                        let a = self.pop_checked()?;
                        let b = self.pop_checked()?;
                        self.stack.push(if a < b { 1 } else { 0 });
                    }
                    VMInstruction::Gt => {
                        let a = self.pop_checked()?;
                        let b = self.pop_checked()?;
                        self.stack.push(if a > b { 1 } else { 0 });
                    }
                    VMInstruction::GtEq => {
                        let a = self.pop_checked()?;
                        let b = self.pop_checked()?;
                        self.stack.push(if a >= b { 1 } else { 0 });
                    }
                    VMInstruction::LtEq => {
                        let a = self.pop_checked()?;
                        let b = self.pop_checked()?;
                        self.stack.push(if a <= b { 1 } else { 0 });
                    }
                    VMInstruction::Neq => {
                        let a = self.pop_checked()?;
                        let b = self.pop_checked()?;
                        self.stack.push(if a != b { 1 } else { 0 });
                    }
                };
                self.pc += 1;
                Ok(None)
            }
            None => Err(VMError::InvalidPCAddress),
        }
    }

    fn pop_checked(&mut self) -> Result<i32, VMError> {
        self.stack.pop().ok_or(VMError::StackExhausted)
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
