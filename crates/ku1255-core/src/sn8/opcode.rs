// Port of `opcode_dict` from libsn8.py.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Space {
    /// No operand
    Nul,
    /// Operand is zero-page ram address
    Zro,
    /// Operand is ram address
    Ram,
    /// Operand is rom address
    Rom,
    /// Operand is immediate value
    Imm,
}

/// What the instruction does to the program counter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    /// No fixed next instruction (RET, RETI)
    NoNext,
    /// Always execute next instruction
    Next,
    /// Execute next instruction or the one after (conditional skips)
    Branch,
    /// Unconditional jump
    Jump,
    /// Call
    Call,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arg {
    None,
    Fixed(&'static str),
    Address,
    BitAddress,
    Immediate,
}

#[derive(Clone, Copy, Debug)]
pub struct Opcode {
    pub key: u8,
    pub mask: u16,
    pub space: Space,
    /// RAM access mode, as shown in disassembly usage listing.
    pub mode: &'static str,
    pub flow: Flow,
    pub mnemonic: &'static str,
    pub left: Arg,
    pub right: Arg,
}

macro_rules! op {
    ($key:expr, $mask:expr, $space:ident, $mode:expr, $flow:ident, $mn:expr, $l:expr, $r:expr) => {
        Opcode {
            key: $key,
            mask: $mask,
            space: Space::$space,
            mode: $mode,
            flow: Flow::$flow,
            mnemonic: $mn,
            left: $l,
            right: $r,
        }
    };
}

use Arg::{Address as Ad, BitAddress as Bt, Fixed as F, Immediate as Im, None as No};

/// Ordered by key, like the Python dict (assembler tries candidates in this order).
pub const OPCODES: &[Opcode] = &[
    op!(0x00, 0x0000, Nul, "", Next, "NOP", No, No),
    op!(0x02, 0x00ff, Zro, "rw", Next, "B0XCH", F("A"), Ad),
    op!(0x03, 0x00ff, Zro, "rw", Next, "B0ADD", Ad, F("A")),
    op!(0x04, 0x0000, Nul, "", Next, "PUSH", No, No),
    op!(0x05, 0x0000, Nul, "", Next, "POP", No, No),
    op!(0x06, 0x00ff, Imm, "", Branch, "CMPRS", F("A"), Im),
    op!(0x07, 0x00ff, Ram, "r ", Branch, "CMPRS", F("A"), Ad),
    op!(0x08, 0x00ff, Ram, "r ", Next, "RRC", Ad, No),
    op!(0x09, 0x00ff, Ram, "rw", Next, "RRCM", Ad, No),
    op!(0x0a, 0x00ff, Ram, "r ", Next, "RLC", Ad, No),
    op!(0x0b, 0x00ff, Ram, "rw", Next, "RLCM", Ad, No),
    op!(0x0d, 0x0000, Nul, "", Next, "MOVC", No, No),
    op!(0x0e, 0x0000, Nul, "", NoNext, "RET", No, No),
    op!(0x0f, 0x0000, Nul, "", NoNext, "RETI", No, No),
    op!(0x10, 0x00ff, Ram, "r ", Next, "ADC", F("A"), Ad),
    op!(0x11, 0x00ff, Ram, "rw", Next, "ADC", Ad, F("A")),
    op!(0x12, 0x00ff, Ram, "r ", Next, "ADD", F("A"), Ad),
    op!(0x13, 0x00ff, Ram, "rw", Next, "ADD", Ad, F("A")),
    op!(0x14, 0x00ff, Imm, "", Next, "ADD", F("A"), Im),
    op!(0x15, 0x00ff, Ram, "r ", Branch, "INCS", Ad, No),
    op!(0x16, 0x00ff, Ram, "rw", Branch, "INCMS", Ad, No),
    op!(0x17, 0x00ff, Ram, "r ", Next, "SWAP", Ad, No),
    op!(0x18, 0x00ff, Ram, "r ", Next, "OR", F("A"), Ad),
    op!(0x19, 0x00ff, Ram, "rw", Next, "OR", Ad, F("A")),
    op!(0x1a, 0x00ff, Imm, "", Next, "OR", F("A"), Im),
    op!(0x1b, 0x00ff, Ram, "r ", Next, "XOR", F("A"), Ad),
    op!(0x1c, 0x00ff, Ram, "rw", Next, "XOR", Ad, F("A")),
    op!(0x1d, 0x00ff, Imm, "", Next, "XOR", F("A"), Im),
    op!(0x1e, 0x00ff, Ram, "r ", Next, "MOV", F("A"), Ad),
    op!(0x1f, 0x00ff, Ram, " w", Next, "MOV", Ad, F("A")),
    op!(0x20, 0x00ff, Ram, "r ", Next, "SBC", F("A"), Ad),
    op!(0x21, 0x00ff, Ram, "rw", Next, "SBC", Ad, F("A")),
    op!(0x22, 0x00ff, Ram, "r ", Next, "SUB", F("A"), Ad),
    op!(0x23, 0x00ff, Ram, "rw", Next, "SUB", Ad, F("A")),
    op!(0x24, 0x00ff, Imm, "", Next, "SUB", F("A"), Im),
    op!(0x25, 0x00ff, Ram, "r ", Branch, "DECS", Ad, No),
    op!(0x26, 0x00ff, Ram, "rw", Branch, "DECMS", Ad, No),
    op!(0x27, 0x00ff, Ram, "rw", Next, "SWAPM", Ad, No),
    op!(0x28, 0x00ff, Ram, "r ", Next, "AND", F("A"), Ad),
    op!(0x29, 0x00ff, Ram, "rw", Next, "AND", Ad, F("A")),
    op!(0x2a, 0x00ff, Imm, "", Next, "AND", F("A"), Im),
    op!(0x2b, 0x00ff, Ram, " 0", Next, "CLR", Ad, No),
    op!(0x2c, 0x00ff, Ram, "rw", Next, "XCH", F("A"), Ad),
    op!(0x2d, 0x00ff, Imm, "", Next, "MOV", F("A"), Im),
    op!(0x2e, 0x00ff, Zro, "r ", Next, "B0MOV", F("A"), Ad),
    op!(0x2f, 0x00ff, Ram, " w", Next, "B0MOV", Ad, F("A")),
    op!(0x32, 0x00ff, Imm, "", Next, "B0MOV", F("R"), Im),
    op!(0x33, 0x00ff, Imm, "", Next, "B0MOV", F("Z"), Im),
    op!(0x34, 0x00ff, Imm, "", Next, "B0MOV", F("Y"), Im),
    op!(0x36, 0x00ff, Imm, "", Next, "B0MOV", F("PFLAG"), Im),
    op!(0x37, 0x00ff, Imm, "", Next, "B0MOV", F("RBANK"), Im),
    op!(0x40, 0x00ff, Ram, " 0", Next, "BCLR", Bt, No),
    op!(0x48, 0x00ff, Ram, " 1", Next, "BSET", Bt, No),
    op!(0x50, 0x00ff, Ram, "r ", Branch, "BTS0", Bt, No),
    op!(0x58, 0x00ff, Ram, "r ", Branch, "BTS1", Bt, No),
    op!(0x60, 0x00ff, Zro, " 0", Next, "B0BCLR", Bt, No),
    op!(0x68, 0x00ff, Zro, " 1", Next, "B0BSET", Bt, No),
    op!(0x70, 0x00ff, Zro, "r ", Branch, "B0BTS0", Bt, No),
    op!(0x78, 0x00ff, Zro, "r ", Branch, "B0BTS1", Bt, No),
    op!(0x80, 0x3fff, Rom, "", Jump, "JMP", Ad, No),
    op!(0xc0, 0x3fff, Rom, "", Call, "CALL", Ad, No),
];

pub fn by_key(key: u8) -> Option<&'static Opcode> {
    OPCODES.iter().find(|op| op.key == key)
}
