//! SN8F2288 assembler / disassembler.
//!
//! Output is intended to be byte-identical to the Python implementation.

mod asm;
mod config;
mod disasm;
mod opcode;

pub use asm::assemble;
pub use config::Chip;
pub use disasm::disassemble;
