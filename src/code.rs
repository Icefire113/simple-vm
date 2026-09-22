use crate::vm::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VMInstruction {
    // Arithmetic stuffs
    /// Pops 2 values from the stack and pushes their sum, wrapping on overflow and setting ErrorFlags.overflow if it occurs
    AddUnchecked,
    /// Pops 2 values from the stack and pushes their sum, acting as a runtime assert; raises an error and sets ErrorFlags.overflow if it overflows
    AddChecked,
    /// Pops 2 values from the stack and pushes their difference, wrapping on overflow and setting ErrorFlags.overflow if it occurs
    SubUnchecked,
    /// Pops 2 values from the stack and pushes their difference, acting as a runtime assert; raises an error and sets ErrorFlags.overflow if it overflows
    SubChecked,
    /// Pops 2 values from the stack and pushes their product, acting as a runtime assert; raises an error and sets ErrorFlags.overflow if it overflows
    MulChecked,
    /// Pops 2 values from the stack and pushes their product, wrapping on overflow and setting ErrorFlags.overflow if it occurs
    MulUnchecked,
    /// Pops 2 values from the stack and pushes their quotient, checking that the divisor is not zero setting ErrorFlags.division_by_zero if it is
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
    Lt,
    /// Pops 2 values from the stack and pushes their greater than (true or false)
    ///
    /// Note that types are checked here, so a comparison between a `Value::Int` and a `Value::Bool` can raise an error
    Gt,
    /// Pops 2 values from the stack and pushes their greater than or equal (true or false)
    ///
    /// Note that types are checked here, so a comparison between a `Value::Int` and a `Value::Bool` can raise an error
    GtEq,
    /// Pops 2 values from the stack and pushes their less than or equal (true or false)
    ///
    /// Note that types are checked here, so a comparison between a `Value::Int` and a `Value::Bool` can raise an error
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

    /// Pushes the overflow flag as a boolean
    PushDivisionByZeroFlag,
    /// Pushes the division by zero flag as a boolean
    PushOverflowFlag,
}
