use std::{collections::HashMap, ops::Range};

use anyhow::Context;
use simple_vm::{assembler::Assembler, code::VMInstruction};

#[derive(Debug, Clone, PartialEq, Eq)]
enum BranchCond {
    Jf,
    Jt,
    Jz,
    Jnz,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CodeBlockTerminator {
    /// A jump to an address, address may be `None` in the case of a `popjump`
    Jump(Option<usize>),
    Branch {
        taken: usize,
        not_taken: usize,
        kind: BranchCond,
    },
    Return,
    Halt,
    Exit,
    EOF,
}

impl CodeBlockTerminator {
    /// Whether this terminator is the instruction at `range.end - 1`
    ///
    /// `EOF` and `None` (fallthrough) are synthetic, so the block's last
    /// instruction is still part of its body in those cases
    fn occupies_last_slot(&self) -> bool {
        matches!(
            self,
            Self::Jump(_) | Self::Branch { .. } | Self::Return | Self::Halt | Self::Exit
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodeBlock {
    /// The block's full span, including the terminator instruction when there is one
    range: Range<usize>,
    /// How control leaves the block; `None` means it falls through to `range.end`
    terminator: Option<CodeBlockTerminator>,
}

impl CodeBlock {
    /// The block's instructions, excluding the terminator if it is an instruction
    ///
    /// Not used by the DOT renderer (which shows the full range), but needed by the
    /// stack-effect pass: the terminator is described by `terminator`, not by `body`
    #[allow(dead_code)]
    pub fn body<'a>(&self, program: &'a [VMInstruction]) -> &'a [VMInstruction] {
        let end = match &self.terminator {
            Some(t) if t.occupies_last_slot() => self.range.end - 1,
            _ => self.range.end,
        };
        &program[self.range.start..end]
    }
}

/// Whether an instruction ends a block, i.e. control may leave it somewhere other than `pc + 1`
fn is_terminator(instr: &VMInstruction) -> bool {
    matches!(
        instr,
        VMInstruction::Jump(_)
            | VMInstruction::JumpIfFalse(_)
            | VMInstruction::JumpIfTrue(_)
            | VMInstruction::JumpIfZero(_)
            | VMInstruction::JumpIfNotZero(_)
            | VMInstruction::PopJump
            | VMInstruction::Halt
            | VMInstruction::Exit
            | VMInstruction::Return
    )
}

/// Block id for a jump/call target, falling back to the raw address when the
/// target is not a block start (out of range or malformed)
fn target_name(starts: &HashMap<usize, usize>, addr: u32) -> String {
    match starts.get(&(addr as usize)) {
        Some(&id) => format!("B{id}"),
        None => addr.to_string(),
    }
}

fn instr_text(instr: &VMInstruction, starts: &HashMap<usize, usize>) -> String {
    match instr {
        VMInstruction::AddUnchecked => "addu".into(),
        VMInstruction::AddChecked => "addc".into(),
        VMInstruction::SubUnchecked => "subu".into(),
        VMInstruction::SubChecked => "subc".into(),
        VMInstruction::MulChecked => "mulc".into(),
        VMInstruction::MulUnchecked => "mulu".into(),
        VMInstruction::DivChecked => "divc".into(),
        VMInstruction::NegUnchecked => "negu".into(),
        VMInstruction::NegChecked => "negc".into(),
        VMInstruction::FmaChecked => "fmac".into(),
        VMInstruction::FmaUnchecked => "fmau".into(),
        VMInstruction::ShiftL(n) => format!("shl {}", n),
        VMInstruction::ShiftR(n) => format!("shr {}", n),
        VMInstruction::RotL(n) => format!("rotl {}", n),
        VMInstruction::RotR(n) => format!("rotr {}", n),
        VMInstruction::BitAnd => "and".into(),
        VMInstruction::BitOr => "or".into(),
        VMInstruction::BitXor => "xor".into(),
        VMInstruction::BitNot => "not".into(),
        VMInstruction::Eq => "eq".into(),
        VMInstruction::Neq => "neq".into(),
        VMInstruction::Lt => "lt".into(),
        VMInstruction::Gt => "gt".into(),
        VMInstruction::GtEq => "gteq".into(),
        VMInstruction::LtEq => "lteq".into(),
        VMInstruction::Swap => "swap".into(),
        VMInstruction::Pop => "pop".into(),
        VMInstruction::Dup => "dup".into(),
        VMInstruction::PushImm(value) => format!("push {}", value),
        VMInstruction::Pick(n) => format!("pick {}", n),
        VMInstruction::Move(n) => format!("move {}", n),
        VMInstruction::PopJump => "popjump".into(),
        VMInstruction::Jump(n) => format!("jmp {}", target_name(starts, *n)),
        VMInstruction::JumpIfNotZero(n) => format!("jnz {}", target_name(starts, *n)),
        VMInstruction::JumpIfZero(n) => format!("jz {}", target_name(starts, *n)),
        VMInstruction::JumpIfTrue(n) => format!("jt {}", target_name(starts, *n)),
        VMInstruction::JumpIfFalse(n) => format!("jf {}", target_name(starts, *n)),
        VMInstruction::Call(n) => format!("call {}", target_name(starts, *n)),
        VMInstruction::Return => "ret".into(),
        VMInstruction::Syscall(n) => format!("syscall {}", n),
        VMInstruction::DebugStack => "dbg_stack".into(),
        VMInstruction::Exit => "exit".into(),
        VMInstruction::Halt => "halt".into(),
        VMInstruction::ClearErrorFlags => "clear_err".into(),
        VMInstruction::ClearDivisionByZero => "clear_div_zero".into(),
        VMInstruction::ClearOverflow => "clear_overflow".into(),
        VMInstruction::PushDivisionByZeroFlag => "push_div_zero".into(),
        VMInstruction::PushOverflowFlag => "push_overflow".into(),
    }
}

/// Where an edge points; the non-block variants are synthetic sink nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EdgeTarget {
    Block(usize),
    /// `Return`, `Halt`, `Exit` or falling off the end of the program
    Exit,
    /// A `popjump` in the given block; the target only exists at runtime
    Dispatch(usize),
    /// A static target that is not a block start (out of range or malformed)
    Invalid,
}

/// Edge colors: the affirmative outcome of a test, the negative outcome, and
/// unconditional jumps
const EDGE_TRUE: &str = "#1a7f37";
const EDGE_FALSE: &str = "#cf222e";
const EDGE_JUMP: &str = "#0969da";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EdgeStyle {
    Solid,
    /// Interprocedural: dashed, and excluded from ranking so recursion does not
    /// distort the layout of the enclosing function
    Call,
    /// Computed jump: dashed, but still ranked below its source block
    Indirect,
}

#[derive(Debug)]
struct Edge {
    from: usize,
    to: EdgeTarget,
    style: EdgeStyle,
    color: Option<&'static str>,
}

fn resolve(starts: &HashMap<usize, usize>, addr: usize) -> EdgeTarget {
    match starts.get(&addr) {
        Some(&id) => EdgeTarget::Block(id),
        None => EdgeTarget::Invalid,
    }
}

fn build_edges(
    program: &[VMInstruction],
    blocks: &[CodeBlock],
    starts: &HashMap<usize, usize>,
) -> Vec<Edge> {
    let mut edges = Vec::new();

    for (id, block) in blocks.iter().enumerate() {
        let edge = |to, style, color| Edge {
            from: id,
            to,
            style,
            color,
        };

        match &block.terminator {
            Some(CodeBlockTerminator::Jump(Some(addr))) => {
                edges.push(edge(resolve(starts, *addr), EdgeStyle::Solid, Some(EDGE_JUMP)))
            }
            Some(CodeBlockTerminator::Jump(None)) => edges.push(edge(
                EdgeTarget::Dispatch(id),
                EdgeStyle::Indirect,
                Some(EDGE_JUMP),
            )),
            Some(CodeBlockTerminator::Branch {
                taken,
                not_taken,
                kind,
            }) => {
                // `taken` is the jump target, `not_taken` is the fallthrough.
                // Green is the affirmative test outcome (true / `== 0`), red the negative
                let (taken_color, not_taken_color) = match kind {
                    BranchCond::Jz | BranchCond::Jt => (EDGE_TRUE, EDGE_FALSE),
                    BranchCond::Jnz | BranchCond::Jf => (EDGE_FALSE, EDGE_TRUE),
                };
                edges.push(edge(
                    resolve(starts, *taken),
                    EdgeStyle::Solid,
                    Some(taken_color),
                ));
                edges.push(edge(
                    resolve(starts, *not_taken),
                    EdgeStyle::Solid,
                    Some(not_taken_color),
                ));
            }
            Some(
                CodeBlockTerminator::Return
                | CodeBlockTerminator::Halt
                | CodeBlockTerminator::Exit
                | CodeBlockTerminator::EOF,
            ) => edges.push(edge(EdgeTarget::Exit, EdgeStyle::Solid, None)),
            // Fell through to the next block's start
            None => edges.push(edge(resolve(starts, block.range.end), EdgeStyle::Solid, None)),
        }

        // `Call` is not a terminator, so a block may contain several
        let mut calls: Vec<usize> = block
            .range
            .clone()
            .filter_map(|pc| match program[pc] {
                VMInstruction::Call(addr) => Some(addr as usize),
                _ => None,
            })
            .collect();
        calls.sort_unstable();
        calls.dedup();
        for addr in calls {
            edges.push(edge(resolve(starts, addr), EdgeStyle::Call, None));
        }
    }

    edges
}

/// Escapes a string for use inside a DOT quoted label
fn escape_dot(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn render_dot(
    program: &[VMInstruction],
    blocks: &[CodeBlock],
    edges: &[Edge],
    starts: &HashMap<usize, usize>,
    entry: Option<usize>,
) -> String {
    let mut out = String::new();
    out.push_str("digraph cfg {\n");
    out.push_str("  rankdir=TB;\n");
    out.push_str("  splines=polyline;\n");
    out.push_str("  node [shape=box, fontname=\"Consolas\", fontsize=9];\n");
    out.push_str("  edge [fontname=\"Consolas\", fontsize=8, tailport=s, headport=n];\n");

    for (id, block) in blocks.iter().enumerate() {
        // The full range is the block's text: it contains the terminator when the
        // terminator is an instruction, and is pure body for fallthrough/EOF blocks
        let mut label = String::new();
        if entry == Some(id) {
            label.push_str(&format!("B{id} (entry):\\l"));
        } else {
            label.push_str(&format!("B{id}:\\l"));
        }
        for pc in block.range.clone() {
            label.push_str(&format!(
                "    {pc}: {}\\l",
                escape_dot(&instr_text(&program[pc], starts))
            ));
        }
        let entry_attr = if entry == Some(id) {
            ", peripheries=2"
        } else {
            ""
        };
        out.push_str(&format!("  B{id} [label=\"{label}\"{entry_attr}];\n"));
    }

    // Keep the entry block on the top rank instead of wherever the edge ranks put it
    if let Some(id) = entry {
        out.push_str(&format!("  {{ rank=source; B{id}; }}\n"));
    }

    if edges.iter().any(|e| e.to == EdgeTarget::Exit) {
        out.push_str("  EXIT [shape=oval, style=dashed, label=\"exit\"];\n");
    }
    // One `?` sink per popjump block, so it is ranked directly beneath the block it
    // jumped out of instead of floating wherever a shared node lands
    let mut dispatch_sources: Vec<usize> = Vec::new();
    for edge in edges {
        if let EdgeTarget::Dispatch(id) = edge.to {
            if !dispatch_sources.contains(&id) {
                dispatch_sources.push(id);
            }
        }
    }
    dispatch_sources.sort_unstable();
    for id in dispatch_sources {
        out.push_str(&format!(
            "  DISPATCH_{id} [shape=diamond, label=\"?\", tooltip=\"popjump in B{id}\"];\n"
        ));
    }
    if edges.iter().any(|e| e.to == EdgeTarget::Invalid) {
        out.push_str("  INVALID [shape=octagon, color=red, label=\"invalid target\"];\n");
    }

    for edge in edges {
        let to = match edge.to {
            EdgeTarget::Block(id) => format!("B{id}"),
            EdgeTarget::Exit => "EXIT".into(),
            EdgeTarget::Dispatch(id) => format!("DISPATCH_{id}"),
            EdgeTarget::Invalid => "INVALID".into(),
        };

        let mut attrs = Vec::new();
        if let Some(color) = edge.color {
            attrs.push(format!("color=\"{color}\""));
            attrs.push(format!("fontcolor=\"{color}\""));
        }
        match edge.style {
            EdgeStyle::Solid => {}
            EdgeStyle::Call => {
                attrs.push("style=dashed".into());
                // Interprocedural edges can form cycles; keep them out of ranking
                attrs.push("constraint=false".into());
            }
            EdgeStyle::Indirect => attrs.push("style=dashed".into()),
        }
        let attrs = if attrs.is_empty() {
            String::new()
        } else {
            format!(" [{}]", attrs.join(", "))
        };

        out.push_str(&format!("  B{} -> {to}{attrs};\n", edge.from));
    }

    out.push_str("}\n");
    out
}

fn main() -> anyhow::Result<()> {
    let asm_path = std::env::args().nth(1).context("No sasm file given")?;
    let asm_text = std::fs::read_to_string(&asm_path).context("Failed to read file")?;
    let assembled = Assembler::new(&asm_text)
        .assemble()
        .context("Failed to assemble")?;

    let mut block_starts: Vec<usize> = vec![0, assembled.entry];
    for (i, instr) in assembled.program.iter().enumerate() {
        match instr {
            VMInstruction::Call(addr) => block_starts.push(*addr as usize),
            VMInstruction::Jump(addr)
            | VMInstruction::JumpIfFalse(addr)
            | VMInstruction::JumpIfTrue(addr)
            | VMInstruction::JumpIfZero(addr)
            | VMInstruction::JumpIfNotZero(addr) => {
                block_starts.push(*addr as usize);
                block_starts.push(i + 1);
            }
            _ if is_terminator(instr) => block_starts.push(i + 1),
            _ => (),
        }
    }
    // Sort the block starts, remove duplicates, and remove any that are out of range
    block_starts.sort();
    block_starts.dedup();
    block_starts.retain(|e| *e < assembled.program.len());

    // addr -> block id map
    let starts: HashMap<usize, usize> = block_starts
        .iter()
        .copied()
        .enumerate()
        .map(|(id, addr)| (addr, id))
        .collect();

    let blocks: Vec<CodeBlock> = block_starts
        .iter()
        .enumerate()
        .map(|(i, &start)| {
            let end = *block_starts.get(i + 1).unwrap_or(&assembled.program.len());
            // Block starts are always < program.len(), so `end >= 1` and `last` is in bounds
            let last = end - 1;
            let terminator = match &assembled.program[last] {
                VMInstruction::PopJump => Some(CodeBlockTerminator::Jump(None)),
                VMInstruction::Jump(addr) => Some(CodeBlockTerminator::Jump(Some(*addr as usize))),
                VMInstruction::JumpIfNotZero(addr) => Some(CodeBlockTerminator::Branch {
                    taken: *addr as usize,
                    not_taken: end,
                    kind: BranchCond::Jnz,
                }),
                VMInstruction::JumpIfZero(addr) => Some(CodeBlockTerminator::Branch {
                    taken: *addr as usize,
                    not_taken: end,
                    kind: BranchCond::Jz,
                }),
                VMInstruction::JumpIfTrue(addr) => Some(CodeBlockTerminator::Branch {
                    taken: *addr as usize,
                    not_taken: end,
                    kind: BranchCond::Jt,
                }),
                VMInstruction::JumpIfFalse(addr) => Some(CodeBlockTerminator::Branch {
                    taken: *addr as usize,
                    not_taken: end,
                    kind: BranchCond::Jf,
                }),
                VMInstruction::Return => Some(CodeBlockTerminator::Return),
                VMInstruction::Exit => Some(CodeBlockTerminator::Exit),
                VMInstruction::Halt => Some(CodeBlockTerminator::Halt),
                // Ran off the end of the program without a terminator
                _ if end == assembled.program.len() => Some(CodeBlockTerminator::EOF),
                // Fell through to the next block's start
                _ => None,
            };

            // The terminator occupies the last slot if and only if it is a real instruction, and no
            // instruction before it may be one of its own
            debug_assert_eq!(
                terminator.as_ref().is_some_and(|t| t.occupies_last_slot()),
                is_terminator(&assembled.program[last])
            );
            debug_assert!(!assembled.program[start..last].iter().any(is_terminator));

            CodeBlock {
                range: start..end,
                terminator,
            }
        })
        .collect();

    let edges = build_edges(&assembled.program, &blocks, &starts);
    let dot = render_dot(
        &assembled.program,
        &blocks,
        &edges,
        &starts,
        starts.get(&assembled.entry).copied(),
    );
    print!("{dot}");

    Ok(())
}
