#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VMInstruction {
    /// Push a 32bit signed integer immediate value onto the stack
    PushImm(i32),

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
    /// Pops a value from the stack and pushes its negation
    Neg,

    /// Pops the top two values from the stack, swaps them, then pushes them back
    Swap,
    /// Pops a value from the stack and treats that value as the VM's "exit code"
    Exit,

    /// Debug instruction, prints the current state of the stack to stderr
    DebugStack,
}
