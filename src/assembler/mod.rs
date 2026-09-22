use crate::{
    assembler::{
        error::{AssembledParseError, AssemblerError},
        parser::Parser,
        tokenizer::tokenizer::Tokenizer,
    },
    code::VMInstruction,
};

pub mod error;
pub mod parser;
pub mod tokenizer;

const PROGRAM_BIN_MAGIC: &[u8; 4] = b"SVM\0";
const PROGRAM_BIN_VERSION: u16 = 1;

/// The result of a successful assembly
#[derive(Debug)]
pub struct Assembled {
    /// The program, as a list of vm instructions
    pub program: Vec<VMInstruction>,
    /// The address of the `:main` label, to be used as the vm's starting program counter
    pub entry: usize,
}

impl Assembled {
    /// Converts the assembled program into a binary file that can be executed by the VM
    pub fn to_bin(&self) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();
        v.extend(PROGRAM_BIN_MAGIC);
        v.extend(PROGRAM_BIN_VERSION.to_le_bytes());
        v.extend((self.entry as u32).to_le_bytes());
        v.extend((self.program.len() as u64).to_le_bytes());
        for instr in self.program.iter() {
            v.extend(instr.as_bytes());
        }

        v
    }

    pub fn from_bin(bin: &[u8]) -> Result<Self, AssembledParseError> {
        if bin.len() < 18 {
            return Err(AssembledParseError::TooShort);
        }
        if bin[0..4] != *PROGRAM_BIN_MAGIC {
            return Err(AssembledParseError::BadMagic);
        }
        let ver = u16::from_le_bytes([bin[4], bin[5]]);
        if ver != PROGRAM_BIN_VERSION {
            return Err(AssembledParseError::BadVersion);
        }
        let entry = u32::from_le_bytes([bin[6], bin[7], bin[8], bin[9]]);
        let instruction_cnt = u64::from_le_bytes([
            bin[10], bin[11], bin[12], bin[13], bin[14], bin[15], bin[16], bin[17],
        ]);
        let mut i = 18;
        let mut instrs: Vec<VMInstruction> = Vec::new();
        for _ in 0..instruction_cnt {
            let (instr, size) = VMInstruction::try_from_bytes(&bin[i..])?;
            i += size as usize;
            instrs.push(instr);
        }

        Ok(Self {
            program: instrs,
            entry: entry as usize,
        })
    }
}

/// Combines tokenizing and parsing
#[derive(Debug)]
pub struct Assembler<'a> {
    text: &'a str,
}

impl<'a> Assembler<'a> {
    /// Creates a new assembler with a given source text
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }

    /// Tokenizes the source text and parses the tokens into a list of vm instructions,
    /// along with the entry point address (the `:main` label)
    pub fn assemble(&self) -> Result<Assembled, AssemblerError> {
        let tokens = Tokenizer::new(self.text).tokenize()?;
        let (program, entry) = Parser::new(&tokens).parse()?;
        Ok(Assembled { program, entry })
    }
}
