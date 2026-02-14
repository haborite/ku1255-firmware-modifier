#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
SN8F2288 assembly address annotator.

Features
--------
- Add address prefixes to each instruction line based on ORG/DW rules
- Strip existing address prefixes
- Recompute and update prefixes after the file has been edited

Assumptions
-----------
- ORG sets the word address (program counter)
- One instruction line = one word
- DW consumes as many words as values listed
- Label-only lines do not advance PC
"""

import sys
import re
from pathlib import Path
from typing import List, Optional


# Prefix format: "|1a2b| " or "|    | "
ADDR_PREFIX_RE = re.compile(r"^\|([0-9A-Fa-f]{4}|\s{4})\|\s")


def parse_hex_token(token: str) -> int:
    """
    Parse hex tokens like:
        0x1234
        1234H
        1234
    """
    t = token.strip().rstrip("Hh")
    if t.lower().startswith("0x"):
        return int(t, 16)
    return int(t, 16)


def count_dw_words(dw_body: str) -> int:
    """
    Count number of words in a DW directive body.

    Example:
        "0035H, 5105H, 2012H" -> 3

    This is intentionally simple and assumes typical assembler output.
    """
    body = dw_body.split(";", 1)[0]
    tokens = re.split(r"[,\s]+", body.strip())
    return len([t for t in tokens if t])


# ---------- prefix strip / add core ----------

def strip_address_prefix_line(line: str) -> str:
    """Remove address prefix if present."""
    return ADDR_PREFIX_RE.sub("", line, count=1)


def strip_addresses_text(lines: List[str]) -> List[str]:
    """Remove address prefixes from all lines."""
    return [strip_address_prefix_line(ln) for ln in lines]


def compute_prefixes(lines: List[str]) -> List[str]:
    """
    Compute address prefixes for each line.
    Returns a list of same length as lines.
    """
    pc: Optional[int] = None
    prefixes: List[str] = []

    for line in lines:
        stripped = line.lstrip()
        code_part = stripped.split(";", 1)[0].rstrip()
        prefix = ""

        # ORG directive
        if code_part.upper().startswith("ORG"):
            parts = code_part.split()
            if len(parts) >= 2:
                try:
                    pc = parse_hex_token(parts[1])
                except ValueError:
                    pc = None
            if pc is not None:
                prefix = "|    | "

        elif pc is not None:
            # blank or pure comment
            if code_part == "" or code_part.startswith(";"):
                pass

            # label line
            elif code_part.endswith(":"):
                prefix = "|    | "

            # DW directive
            elif code_part.upper().startswith("DW"):
                prefix = f"|{pc:04x}| "
                n = count_dw_words(code_part[2:].strip())
                pc += max(n, 1)

            # instruction line
            else:
                prefix = f"|{pc:04x}| "
                pc += 1

        prefixes.append(prefix)

    return prefixes


def add_addresses_text(lines: List[str]) -> List[str]:
    """Add computed address prefixes to lines."""
    prefixes = compute_prefixes(lines)
    return [(p + l) if p else l for p, l in zip(prefixes, lines)]


def update_addresses_text(lines: List[str]) -> List[str]:
    """Strip then recompute prefixes."""
    base = strip_addresses_text(lines)
    return add_addresses_text(base)


# ---------- file I/O helpers ----------

def read_lines(path: Path) -> List[str]:
    return path.read_text(encoding="utf-8").splitlines()


def write_lines(path: Path, lines: List[str]) -> None:
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


# ---------- commands ----------

def cmd_add(in_path: Path, out_path: Path) -> None:
    write_lines(out_path, add_addresses_text(read_lines(in_path)))


def cmd_strip(in_path: Path, out_path: Path) -> None:
    write_lines(out_path, strip_addresses_text(read_lines(in_path)))


def cmd_update(in_path: Path, out_path: Path) -> None:
    write_lines(out_path, update_addresses_text(read_lines(in_path)))


# ---------- CLI ----------

def default_out_path(in_path: Path, mode: str) -> Path:
    if mode == "add":
        return in_path.with_suffix(".addr.asm")

    if mode == "strip":
        name = in_path.name
        if name.endswith(".addr.asm"):
            return in_path.with_name(name[:-9] + ".asm")
        return in_path.with_suffix(".stripped.asm")

    if mode == "update":
        return in_path

    raise ValueError(mode)


def usage() -> None:
    print(
        "Usage:\n"
        "  sn8_addr.py add    <input.asm> [output.asm]\n"
        "  sn8_addr.py strip  <input.asm> [output.asm]\n"
        "  sn8_addr.py update <input.asm> [output.asm]\n"
    )


def main(argv=None) -> int:
    if argv is None:
        argv = sys.argv[1:]

    if len(argv) < 2:
        usage()
        return 1

    mode = argv[0].lower()
    if mode not in ("add", "strip", "update"):
        usage()
        return 1

    in_path = Path(argv[1])
    if not in_path.exists():
        print(f"Input file not found: {in_path}")
        return 2

    out_path = Path(argv[2]) if len(argv) >= 3 else default_out_path(in_path, mode)

    if mode == "add":
        cmd_add(in_path, out_path)
    elif mode == "strip":
        cmd_strip(in_path, out_path)
    else:
        cmd_update(in_path, out_path)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
