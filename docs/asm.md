# SASM — Simple Assembly

> [!NOTE]
> This document was written with AI assistance and may contain errors or drift
> from the implementation. When in doubt, trust the code (and tests) over this
> file — and please fix the docs if you find a mismatch.

SASM is the assembly language for `simple-vm`, a stack-based virtual machine over
32-bit signed integers and booleans. Programs are flat lists of instructions,
organized into **named blocks (labels)**. The assembler resolves label names into
instruction indices at build time, so source code never deals with raw addresses
(though they are allowed).

One source line conventionally contains one instruction; blank lines and comments
are ignored.

## Example

```sasm
:main
    push 10
    push 32
    shl 2        # 32 << 2 = 128
    addu
    exit         # exits with 138
```

## Lexical structure

### Comments

`#` starts a line comment; everything up to the newline is ignored. Comments may
appear on their own line or after an instruction.

```sasm
push 5    # this is a comment
```

### Labels

A label is a `:` followed by an identifier, conventionally at the start of a line:

```sasm
:main
:loop_start
```

A label binds to the index of the *next* instruction emitted after it — not the
next line. Multiple labels may stack on the same instruction; a label with no
following instruction (e.g. at end of file) binds to one past the last
instruction.

- Every program must define `:main` (see [Entry point](#entry-point)).
- Labels must be unique; redefinition is an assembly error.
- Referencing an undefined label is an assembly error.
- Label identifiers are case-sensitive and may contain letters, digits and `_`.
  Mnemonic spellings are matched case-insensitively (`PUSH` == `push`).

### Integer literals

Decimal, hex (`0x`) and binary (`0b`) are supported. Hex literals accept the full
unsigned 32-bit range; `0xFFFFFFFF` is a valid spelling of `-1`.

```sasm
push 42
push 0x1F
push 0b1010
push 0xFFFFFFFF
```

Negative immediates are written with a leading `-`:

```sasm
push -4
```

### Character and string literals

Char literals hold exactly one character; string literals hold any number of
characters. Both support backslash escapes: `\n` `\t` `\r` `\0` `\\` `\'` `\"`.
(An unknown escape like `\q` resolves to `q`.)

```sasm
push 'a'
push '\n'
```

Currently only char literals are valid operands for `push`; strings are reserved
for future data directives.

### Directives

A `.` followed by a name is a directive token (e.g. `.data`). No directives are
implemented yet; they are reserved for future sections (data, string storage, …).

## Grammar

```
program     := line*
line        := (label | instruction | comment)? newline
label       := ':' identifier
instruction := keyword operand?
operand     := int | char | identifier
```

## Entry point

Execution begins at the `:main` label, wherever it appears in the file. `:main`
is required; a program without it fails to assemble. (Instruction order in the
source is preserved — `:main` does not need to be the first block, though it
conventionally comes last so that fall-through into it is harmless.)

## Instructions

Stack notation: `[a, b, c]` with `c` on top. Notation `[a, b] → [x]` means the
operand stack before → after. "Checked" arithmetic acts as a runtime assert: on
overflow it sets the overflow flag **and raises an error**, halting the VM.
"Unchecked" arithmetic wraps and sets the overflow flag but keeps running, so a
program can inspect flags afterward (`push_overflow`, `jnz ...`). Flags are
sticky until cleared (see [Flags](#error-flags)).

### Stack manipulation

| Mnemonic  | Stack                    | Effect                                    |
|-----------|--------------------------|-------------------------------------------|
| `push v`  | `[] → [v]`               | Push a literal (int or char)              |
| `dup`     | `[a] → [a, a]`           | Duplicate top value                       |
| `swap`    | `[a, b] → [b, a]`        | Swap top two values                       |
| `pop`     | `[a, …] → […]`           | Discard top value                         |
| `popjump` | `[addr] → []`            | Set pc to popped value (unchecked target) |

### Arithmetic

Binary ops pop the top two values and push the result. For `op a b` forms the
**top of stack is the right-hand operand**: `push x; push y; subu` computes
`y - x`.

| Mnemonic | Stack            | Effect                                  |
|----------|------------------|-----------------------------------------|
| `addu`   | `[a, b] → [a+b]` | Add, wraps on overflow, sets flag       |
| `addc`   | `[a, b] → [a+b]` | Add, asserts no overflow                |
| `subu`   | `[a, b] → [a-b]` | Subtract (`b - a`), wraps, sets flag    |
| `subc`   | `[a, b] → [a-b]` | Subtract, asserts no overflow           |
| `mulu`   | `[a, b] → [a*b]` | Multiply, wraps, sets flag              |
| `mulc`   | `[a, b] → [a*b]` | Multiply, asserts no overflow           |
| `divc`   | `[a, b] → [a/b]` | Divide (`a / b`, truncating), `b = 0` → error + flag |
| `negu`   | `[a] → [-a]`     | Negate, wraps (`i32::MIN` → itself), sets flag |
| `negc`   | `[a] → [-a]`     | Negate, asserts no overflow             |

### Bitwise

Shift/rotate take the value from the stack; the **amount is an immediate
operand**. Amounts >= 32 are errors for shifts; rotate amounts wrap modulo 32.
Right shifts are arithmetic (sign-preserving).

| Mnemonic  | Stack          | Effect                              |
|-----------|----------------|-------------------------------------|
| `shl n`   | `[a] → [a<<n]` | Shift left by immediate n (0..=31)  |
| `shr n`   | `[a] → [a>>n]` | Arithmetic shift right              |
| `rotl n`  | `[a] → [a rotl n]` | Rotate left, n mod 32           |
| `rotr n`  | `[a] → [a rotr n]` | Rotate right, n mod 32          |

### Comparison

Pop two values, push a boolean. Equality compares by value *and* type
(`true == 1` is `false`); ordered comparisons require both operands to be ints.

| Mnemonic | Result           |
|----------|------------------|
| `eq`     | `a == b`         |
| `neq`    | `a != b`         |
| `lt`     | `a < b`          |
| `gt`     | `a > b`          |
| `lteq`   | `a <= b`         |
| `gteq`   | `a >= b`         |

### Control flow

Targets are label names (assembled to instruction indices). Conditional jumps
**pop their test value**; `jz`/`jnz` test an int against zero, `jt`/`jf` require
a boolean.

| Mnemonic   | Stack          | Effect                                  |
|------------|----------------|-----------------------------------------|
| `jmp lbl`  | —              | Unconditional jump                      |
| `jz lbl`   | `[v] → []`     | Jump if `v == 0` (int)                  |
| `jnz lbl`  | `[v] → []`     | Jump if `v != 0` (int)                  |
| `jt lbl`   | `[b] → []`     | Jump if `b == true` (bool)              |
| `jf lbl`   | `[b] → []`     | Jump if `b == false` (bool)             |
| `exit`     | `[v] → []`     | Halt; popped value is the exit code     |

A label defined at the very end of the program (with no instructions after it)
resolves to one past the last instruction. Jumping there ends execution — this
is currently a runtime error (`InvalidPCAddress`), so end-of-program jump
targets should be avoided for now.

### Error flags and VM control

Flags (`overflow`, `division_by_zero`) are sticky: set when the corresponding
condition occurs (checked *or* unchecked), cleared only by the instructions
below or by loading a new program.

| Mnemonic            | Stack        | Effect                                |
|---------------------|--------------|---------------------------------------|
| `clear_err`         | —            | Clear both flags                      |
| `clear_div_zero`    | —            | Clear the division-by-zero flag       |
| `clear_overflow`    | —            | Clear the overflow flag               |
| `push_div_zero`     | `[] → [b]`   | Push division-by-zero flag as bool    |
| `push_overflow`     | `[] → [b]`   | Push overflow flag as bool            |
| `dbg_stack`         | —            | Print pc + stack to stderr (debug aid) |

## Idioms

```sasm
# test a value without consuming it (jz pops): duplicate first
dup
jz is_zero

# decrement top of stack (note operand order: subu computes b - a)
push -1
addu
```

## Known gaps

- `push true` / `push false`: the `true`/`false` keywords are tokenized but not
  yet accepted by the parser.
- No `call`/`ret`, no memory (`load`/`store`), no bitwise logic ops
  (`and`/`or`/`xor`/`not`), no `mod`.
- Jump targets can also be raw integer literals (`jmp 3`), which skips label
  safety entirely — useful for tests, footguns otherwise.
