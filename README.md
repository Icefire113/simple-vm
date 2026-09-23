# simple-vm

A stack-based virtual machine over `i32` and `bool` values, with its own
assembly language (**SASM**) and a simple binary program format (`.svm`).
Single Rust crate with a library core and two thin CLI binaries.

## Usage

Assemble a source file and run it:

```sh
cargo run --bin sasm -- samples/fib.sasm out.svm
cargo run --bin svm  -- out.svm
```

Note: `sasm` uses `create_new(true)`, so the output file must not already
exist — delete the `.svm` between runs.

## SASM in a nutshell

- Stack machine: instructions push/pop 32-bit integers and booleans
- Programs are flat instruction lists organized under `:label` blocks; `:main` is required
- Comments start with `#`
- Arithmetic/compare ops (unchecked variants wrap and set sticky error flags; checked variants halt on error)
- Jumps: unconditional, plus `jz`/`jnz` (int test) and `jt`/`jf` (bool test), which pop their operand

Example:

```sasm
:main
    push 10
    push 32
    shl 2        # 32 << 2 = 128
    addu
    exit         # exits with 138
```

See `docs/asm.md` for the full instruction table and language details.
