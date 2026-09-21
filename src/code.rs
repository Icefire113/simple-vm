use crate::vm::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VMInstruction {
    // Arithmetic stuffs
    /// Pops 2 values from the stack and pushes their sum, allows for overflow
    AddUnchecked,
    /// Pops 2 values from the stack and pushes their sum, checking for overflow setting ErrorFlags.overflow if it does
    AddChecked,
    /// Pops 2 values from the stack and pushes their difference
    Sub,
    /// Pops 2 values from the stack and pushes their product, checking for overflow setting ErrorFlags.overflow if it does
    MulChecked,
    /// Pops 2 values from the stack and pushes their product, allows for overflow
    MulUnchecked,
    /// Pops 2 values from the stack and pushes their quotient, checking that the divisor is not zero setting ErrorFlags.division_by_zero if it is
    DivChecked,
    /// Pops a value from the stack and pushes its negation, allowing for overflow and not setting ErrorFlags.overflow
    NegUnchecked,
    /// Pops a value from the stack and pushes its negation, allowing for overflow and setting ErrorFlags.overflow if it occurs
    NegChecked,

    // Comparison stuffs
    /// Pops 2 values from the stack and pushes their equality (1 or 0)
    Eq,
    /// Pops 2 values from the stack and pushes their less than (1 or 0)
    Lt,
    /// Pops 2 values from the stack and pushes their greater than (1 or 0)
    Gt,
    /// Pops 2 values from the stack and pushes their greater than or equal (1 or 0)
    GtEq,
    /// Pops 2 values from the stack and pushes their less than or equal (1 or 0)
    LtEq,
    /// Pops 2 values from the stack and pushes their not equal (1 or 0)
    Neq,

    // stack operations
    /// Pops the top two values from the stack, swaps them, then pushes them back
    Swap,
    /// Pops a value from the stack and discards it
    Pop,
    /// Push a 32bit signed integer immediate value onto the stack
    PushImm(Value),

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

    // VM control stuffs
    /// Debug instruction, prints the current state of the stack to stderr
    DebugStack,
    /// Pops a value from the stack and treats that value as the VM's "exit code"
    Exit,
}
