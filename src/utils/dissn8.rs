// sn8_disasm.rs
use std::fmt::Write as _;

/// Addressing space for the operand (reduced set vs Python version)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AddrSpace {
    Null, // no operand
    Zero, // zero page RAM (ZRO_SPACE)
    Ram,  // RAM_SPACE
    Rom,  // ROM_SPACE
    Imm,  // IMM_SPACE
}

/// Where each operand string comes from
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperandSlot {
    None,
    Fixed(&'static str), // e.g. "A", "R", "0", "1"
    Dynamic,             // replaced by decoded operand string
}

/// One opcode entry (keyed by the "opcode_key")
#[derive(Clone, Copy, Debug)]
struct OpcodeEntry {
    key: u8,
    mask: u16,
    space: AddrSpace,
    mnemonic: &'static str,
    left: OperandSlot,
    right: OperandSlot,
}

// Direct port of opcode_dict from libsn8.py (but only data needed for disasm).:contentReference[oaicite:1]{index=1}
const OPCODES: &[OpcodeEntry] = &[
    // key,  mask,    space,        mnemonic,  left,               right
    OpcodeEntry { key: 0x00, mask: 0x0000, space: AddrSpace::Null, mnemonic: "NOP",
        left: OperandSlot::None, right: OperandSlot::None },

    OpcodeEntry { key: 0x02, mask: 0x00ff, space: AddrSpace::Zero, mnemonic: "B0XCH",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x03, mask: 0x00ff, space: AddrSpace::Zero, mnemonic: "B0ADD",
        left: OperandSlot::Dynamic, right: OperandSlot::Fixed("A") },

    OpcodeEntry { key: 0x04, mask: 0x0000, space: AddrSpace::Null, mnemonic: "PUSH",
        left: OperandSlot::None, right: OperandSlot::None },

    OpcodeEntry { key: 0x05, mask: 0x0000, space: AddrSpace::Null, mnemonic: "POP",
        left: OperandSlot::None, right: OperandSlot::None },

    OpcodeEntry { key: 0x06, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "CMPRS",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x07, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "CMPRS",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x08, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "RRC",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x09, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "RRCM",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x0a, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "RLC",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x0b, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "RLCM",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x0d, mask: 0x0000, space: AddrSpace::Null, mnemonic: "MOVC",
        left: OperandSlot::None, right: OperandSlot::None },

    OpcodeEntry { key: 0x0e, mask: 0x0000, space: AddrSpace::Null, mnemonic: "RET",
        left: OperandSlot::None, right: OperandSlot::None },

    OpcodeEntry { key: 0x0f, mask: 0x0000, space: AddrSpace::Null, mnemonic: "RETI",
        left: OperandSlot::None, right: OperandSlot::None },

    OpcodeEntry { key: 0x10, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "ADC",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x11, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "ADC",
        left: OperandSlot::Dynamic, right: OperandSlot::Fixed("A") },

    OpcodeEntry { key: 0x12, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "ADD",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x13, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "ADD",
        left: OperandSlot::Dynamic, right: OperandSlot::Fixed("A") },

    OpcodeEntry { key: 0x14, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "ADD",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x15, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "INCS",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x16, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "INCMS",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x17, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "SWAP",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x18, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "OR",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x19, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "OR",
        left: OperandSlot::Dynamic, right: OperandSlot::Fixed("A") },

    OpcodeEntry { key: 0x1a, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "OR",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x1b, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "XOR",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x1c, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "XOR",
        left: OperandSlot::Dynamic, right: OperandSlot::Fixed("A") },

    OpcodeEntry { key: 0x1d, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "XOR",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x1e, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "MOV",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x1f, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "MOV",
        left: OperandSlot::Dynamic, right: OperandSlot::Fixed("A") },

    OpcodeEntry { key: 0x20, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "SBC",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x21, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "SBC",
        left: OperandSlot::Dynamic, right: OperandSlot::Fixed("A") },

    OpcodeEntry { key: 0x22, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "SUB",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x23, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "SUB",
        left: OperandSlot::Dynamic, right: OperandSlot::Fixed("A") },

    OpcodeEntry { key: 0x24, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "SUB",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x25, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "DECS",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x26, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "DECMS",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x27, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "SWAPM",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x28, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "AND",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x29, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "AND",
        left: OperandSlot::Dynamic, right: OperandSlot::Fixed("A") },

    OpcodeEntry { key: 0x2a, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "AND",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x2b, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "CLR",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x2c, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "XCH",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x2d, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "MOV",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x2e, mask: 0x00ff, space: AddrSpace::Zero, mnemonic: "B0MOV",
        left: OperandSlot::Fixed("A"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x2f, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "B0MOV",
        left: OperandSlot::Dynamic, right: OperandSlot::Fixed("A") },

    OpcodeEntry { key: 0x32, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "B0MOV",
        left: OperandSlot::Fixed("R"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x33, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "B0MOV",
        left: OperandSlot::Fixed("Z"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x34, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "B0MOV",
        left: OperandSlot::Fixed("Y"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x36, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "B0MOV",
        left: OperandSlot::Fixed("PFLAG"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x37, mask: 0x00ff, space: AddrSpace::Imm, mnemonic: "B0MOV",
        left: OperandSlot::Fixed("RBANK"), right: OperandSlot::Dynamic },

    OpcodeEntry { key: 0x40, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "BCLR",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x48, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "BSET",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x50, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "BTS0",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x58, mask: 0x00ff, space: AddrSpace::Ram, mnemonic: "BTS1",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x60, mask: 0x00ff, space: AddrSpace::Zero, mnemonic: "B0BCLR",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x68, mask: 0x00ff, space: AddrSpace::Zero, mnemonic: "B0BSET",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x70, mask: 0x00ff, space: AddrSpace::Zero, mnemonic: "B0BTS0",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x78, mask: 0x00ff, space: AddrSpace::Zero, mnemonic: "B0BTS1",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0x80, mask: 0x3fff, space: AddrSpace::Rom, mnemonic: "JMP",
        left: OperandSlot::Dynamic, right: OperandSlot::None },

    OpcodeEntry { key: 0xc0, mask: 0x3fff, space: AddrSpace::Rom, mnemonic: "CALL",
        left: OperandSlot::Dynamic, right: OperandSlot::None },
];

fn find_opcode(opcode_key: u8) -> Option<&'static OpcodeEntry> {
    OPCODES.iter().find(|op| op.key == opcode_key)
}

fn as_printable(byte: u8) -> char {
    if (0x20..=0x7e).contains(&byte) {
        byte as char
    } else {
        '.'
    }
}

/// Disassemble one 16-bit word at a given address.
fn disassemble_word(address: u16, instruction: u16) -> String {
    let bincode: u8 = (instruction >> 8) as u8;
    // Same opcode_key selection logic as Python version.:contentReference[oaicite:2]{index=2}
    let opcode_key = if bincode >= 0x80 {
        bincode & 0xC0
    } else if bincode >= 0x40 {
        bincode & 0xF8
    } else {
        bincode
    };

    let opcode = match find_opcode(opcode_key) {
        Some(op) => op,
        None => {
            // Unknown / illegal opcode → DW 0xXXXX ; printable bytes
            let hi = (instruction >> 8) as u8;
            let lo = (instruction & 0xff) as u8;
            return format!(
                "DW\t0x{instr:04x}\t; {}{}",
                as_printable(hi),
                as_printable(lo),
                instr = instruction
            );
        }
    };

    // No-operand instruction
    if matches!(opcode.space, AddrSpace::Null) {
        return opcode.mnemonic.to_string();
    }

    let operand_raw = instruction & opcode.mask;
    let is_bit = bincode >= 0x40 && bincode < 0x80;

    // Format operand depending on address space
    let mut symbol = match opcode.space {
        AddrSpace::Rom => format!("0x{operand:04x}", operand = operand_raw),
        AddrSpace::Imm => format!("#0x{val:02x}", val = (operand_raw & 0xff) as u8),
        AddrSpace::Zero | AddrSpace::Ram => {
            let addr = (operand_raw & 0xff) as u8;
            if is_bit {
                let bit = bincode & 0x7;
                format!("0x{addr:02x}.{bit}", addr = addr, bit = bit)
            } else {
                format!("0x{addr:02x}", addr = addr)
            }
        }
        AddrSpace::Null => unreachable!(),
    };

    // For unconditional jumps, mimic Python's special case "jump to self+1" as $+1.
    if matches!(opcode.space, AddrSpace::Rom) && opcode.mnemonic == "JMP" {
        let target = operand_raw & 0x3fff;
        if target == address.wrapping_add(1) {
            symbol = "$+1".to_string();
        }
    }

    // Build operand list based on operand slots
    let mut ops: Vec<String> = Vec::new();
    for slot in [opcode.left, opcode.right] {
        match slot {
            OperandSlot::None => {}
            OperandSlot::Fixed(s) => ops.push(s.to_string()),
            OperandSlot::Dynamic => ops.push(symbol.clone()),
        }
    }

    if ops.is_empty() {
        opcode.mnemonic.to_string()
    } else {
        format!("{}\t{}", opcode.mnemonic, ops.join(", "))
    }
}

/// High-level helper: disassemble a whole firmware image (little endian words).
///
/// - `rom` length must be even; two bytes per instruction.
/// - Addresses start at 0x0000 and increment by word.
pub fn disassemble_sn8(rom: &[u8]) -> String {
    let mut out = String::new();

    for (i, chunk) in rom.chunks_exact(2).enumerate() {
        let instr = u16::from_le_bytes([chunk[0], chunk[1]]);
        let addr = i as u16;
        let line = disassemble_word(addr, instr);
        // Format similar to Python: "ORG" etc are omitted here; only address + opcode.
        let _ = writeln!(&mut out, "ORG 0x{addr:04x}\n\t{line}", addr = addr, line = line);
    }

    out
}

// Optional: simple CLI equivalent to "python dissn8 fw.bin -o fw.asm"
// (no -c cfg, no walker/systematic switch – always systematic).
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nop() {
        // NOP = opcode_key 0x00, full instruction 0x0000
        let code = [0x00u8, 0x00u8];
        let s = disassemble_sn8(&code);
        assert!(s.contains("NOP"));
    }
}
