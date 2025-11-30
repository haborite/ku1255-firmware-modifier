#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Analyze MOVC table accesses in a dissN8-style SN8F2288 assembly listing.

- Scan an .asm file (output of dissn8)
- Collect:
  - labels and their ROM addresses (from '; 0x????' comments)
  - MOVC patterns of the form:
        B0MOV  Y, #0xHH
        B0MOV  Z, #0xLL
        MOVC
- For each MOVC pattern, compute the ROM address HHLL and
  try to associate it with the nearest label (table start).

Usage:
    python analyze_movc_tables.py firmware.asm [firmware.bin]
"""

import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Tuple


# ---------- Data structures ----------

@dataclass
class LineInfo:
    idx: int                 # 0-based line index
    text: str                # original text
    addr: Optional[int]      # ROM address if we could parse from comment


@dataclass
class LabelInfo:
    name: str
    line_idx: int
    addr: Optional[int]      # ROM address (if known)


@dataclass
class MovcUse:
    y_val: int
    z_val: int
    table_addr: int
    line_idx_y: int
    line_idx_movc: int
    movc_addr: Optional[int]  # ROM address of MOVC itself, if known


# ---------- Regex patterns ----------

RE_ADDR_COMMENT = re.compile(r";\s*0x([0-9a-fA-F]+)")
RE_LABEL = re.compile(r"^([A-Za-z_]\w*):")
RE_B0MOV_Y = re.compile(r"^\s*B0MOV\s+Y,\s*#0x([0-9a-fA-F]{2})\b", re.IGNORECASE)
RE_B0MOV_Z = re.compile(r"^\s*B0MOV\s+Z,\s*#0x([0-9a-fA-F]{2})\b", re.IGNORECASE)
RE_MOVC = re.compile(r"^\s*MOVC\b", re.IGNORECASE)


# ---------- Parsing helpers ----------

def parse_asm_lines(path: Path) -> Tuple[List[LineInfo], List[LabelInfo]]:
    """Parse .asm file into LineInfo and LabelInfo lists."""
    lines: List[LineInfo] = []
    labels: List[LabelInfo] = []

    current_addr: Optional[int] = None

    with path.open("r", encoding="utf-8", errors="ignore") as f:
        raw_lines = f.readlines()

    for idx, raw in enumerate(raw_lines):
        line = raw.rstrip("\n")

        # 1) Address comment detection: "; 0x1234"
        m_addr = RE_ADDR_COMMENT.search(line)
        if m_addr:
            try:
                current_addr = int(m_addr.group(1), 16)
            except ValueError:
                current_addr = None

        # 2) Label detection: "LabelName:"
        m_label = RE_LABEL.match(line)
        if m_label:
            label_name = m_label.group(1)
            labels.append(LabelInfo(name=label_name,
                                    line_idx=idx,
                                    addr=current_addr))

        lines.append(LineInfo(idx=idx, text=line, addr=current_addr))

    return lines, labels


def find_movc_uses(lines: List[LineInfo]) -> List[MovcUse]:
    """Find patterns:
        B0MOV Y, #0xHH
        B0MOV Z, #0xLL
        MOVC
    and record them as MovcUse objects.
    """
    uses: List[MovcUse] = []
    n = len(lines)

    for i in range(n - 2):
        line_y = lines[i].text
        line_z = lines[i + 1].text
        line_movc = lines[i + 2].text

        my = RE_B0MOV_Y.match(line_y)
        mz = RE_B0MOV_Z.match(line_z)
        mm = RE_MOVC.match(line_movc)

        if not (my and mz and mm):
            continue

        y_val = int(my.group(1), 16)
        z_val = int(mz.group(1), 16)
        table_addr = (y_val << 8) | z_val

        uses.append(
            MovcUse(
                y_val=y_val,
                z_val=z_val,
                table_addr=table_addr,
                line_idx_y=lines[i].idx,
                line_idx_movc=lines[i + 2].idx,
                movc_addr=lines[i + 2].addr,
            )
        )

    return uses


def build_label_index(labels: List[LabelInfo]) -> List[LabelInfo]:
    """Return labels sorted by addr, dropping those without address."""
    filtered = [lb for lb in labels if lb.addr is not None]
    return sorted(filtered, key=lambda lb: lb.addr)  # type: ignore[arg-type]


def find_label_for_addr(label_index: List[LabelInfo], addr: int) -> Optional[LabelInfo]:
    """Find the label whose addr <= addr and is closest below (table start).
       Return None if not found.
    """
    candidate: Optional[LabelInfo] = None
    for lb in label_index:
        if lb.addr is None:
            continue
        if lb.addr <= addr:
            candidate = lb
        else:
            break
    return candidate


# ---------- Main CLI ----------

def load_bin_len(path: Optional[Path]) -> Optional[int]:
    if path is None:
        return None
    if not path.is_file():
        print(f"[WARN] .bin file not found: {path}", file=sys.stderr)
        return None
    return path.stat().st_size


def main(argv: List[str]) -> None:
    if len(argv) < 2:
        print("Usage: python analyze_movc_tables.py firmware.asm [firmware.bin]")
        sys.exit(1)

    asm_path = Path(argv[1])
    bin_path = Path(argv[2]) if len(argv) >= 3 else None

    if not asm_path.is_file():
        print(f"[ERROR] .asm file not found: {asm_path}")
        sys.exit(1)

    print(f"[*] Parsing ASM: {asm_path}")
    lines, labels = parse_asm_lines(asm_path)
    label_index = build_label_index(labels)
    print(f"    - Parsed {len(lines)} lines, {len(labels)} labels ({len(label_index)} with address).")

    bin_len = load_bin_len(bin_path)
    if bin_len is not None:
        print(f"[*] .bin length = 0x{bin_len:04X} bytes")

    print("\n[*] Scanning for MOVC table accesses...")
    movc_uses = find_movc_uses(lines)
    print(f"    - Found {len(movc_uses)} MOVC patterns (B0MOV Y/Z + MOVC).")

    print("\n=== MOVC table access summary ===")
    for use in movc_uses:
        table_addr = use.table_addr
        lb = find_label_for_addr(label_index, table_addr)

        # Basic info
        print(f"\n- MOVC at line {use.line_idx_movc + 1}", end="")
        if use.movc_addr is not None:
            print(f" (ROM 0x{use.movc_addr:04X})", end="")
        print(":")

        print(f"  Y = 0x{use.y_val:02X}, Z = 0x{use.z_val:02X}  ->  ROM[0x{table_addr:04X}]")

        # .bin range check
        if bin_len is not None:
            if 0 <= table_addr < bin_len:
                print("  [OK] Address is within .bin range.")
            else:
                print("  [WARN] Address is outside .bin range!")

        # Label association
        if lb is not None and lb.addr is not None:
            offset = table_addr - lb.addr
            print(f"  Table label: {lb.name} @ 0x{lb.addr:04X} (offset +0x{offset:X})")
        else:
            print("  Table label: (no label with address <= this addr was found)")

    print("\n[*] Done.")


if __name__ == "__main__":
    main(sys.argv)
