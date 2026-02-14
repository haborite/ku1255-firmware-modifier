#!/usr/bin/env python3
import sys
from pathlib import Path

if len(sys.argv) < 2:
    print("usage: cut_after_semicolon.py input.asm [output.asm]")
    sys.exit(1)

inp = Path(sys.argv[1])
out = Path(sys.argv[2]) if len(sys.argv) >= 3 else inp.with_suffix(".cut.asm")

lines = inp.read_text(encoding="utf-8").splitlines()

def cut_after_semicolon(line: str) -> str:
    i = line.find(";")
    return line if i == -1 else line[:i+1]

out.write_text("\n".join(cut_after_semicolon(l) for l in lines) + "\n",
               encoding="utf-8")
