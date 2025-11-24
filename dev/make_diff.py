#!/usr/bin/env python3
"""
Usage:
    python make_diff.py Origin.asm Modified.asm diff.json comments.txt

Purpose:
- Read Origin.asm and Modified.asm and:
  - Output the code-only diff as an “operation list JSON” into diff.json
  - Output only the comments of B into comments.txt

Specification:
- A comment consists of the first ';' in the line and everything after it.
  Example: "ADD A, 0x2e ; foo"
           -> code = "ADD A, 0x2e "
           -> comment = "; foo"
- Format of diff.json:
  {
    "meta": {...},
    "ops": [
      { "op": "copy",   "from": <line_no_in_A> },
      { "op": "insert", "code": "<code_of_B>" },
      ...
    ]
  }
- The content of A is NOT stored in diff.json, only line numbers.
"""

import sys
import json
import difflib
from typing import List, Tuple, Dict, Any


def split_code_comment(line: str) -> Tuple[str, str]:
    """Split a line into (code_part, comment_part).

    The comment_part includes ';' and everything after it.
    """
    line = line.rstrip("\n")
    idx = line.find(";")
    if idx == -1:
        return line, ""
    return line[:idx], line[idx:]


def read_code_and_comments(path: str) -> Tuple[List[str], List[str]]:
    """Read a file and return lists of code parts and comment parts.

    Returns:
        codes:    ['code...', 'code...', ...]    # no newline
        comments: [';...', '', ';...', ...]      # same length as codes
    """
    codes: List[str] = []
    comments: List[str] = []
    with open(path, encoding="utf-8") as f:
        for raw in f:
            code, comment = split_code_comment(raw)
            codes.append(code)
            comments.append(comment)
    return codes, comments


def build_ops(a_codes: List[str], b_codes: List[str]) -> Dict[str, Any]:
    """Construct the operation list JSON to transform A’s code into B’s code."""
    sm = difflib.SequenceMatcher(None, a_codes, b_codes)
    ops: List[Dict[str, Any]] = []

    # Interpret SequenceMatcher opcodes directly
    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        if tag == "equal":
            # Lines i1..i2-1 of A equal lines j1..j2-1 of B
            for offset in range(i2 - i1):
                line_a = i1 + offset + 1  # 1-based line number
                ops.append({"op": "copy", "from": line_a})

        elif tag in ("replace", "insert"):
            # New or modified lines in B → insert operations
            for bj in range(j1, j2):
                code_b = b_codes[bj]
                ops.append({"op": "insert", "code": code_b})

            # For "replace", A’s lines i1..i2-1 are ignored = deleted

        elif tag == "delete":
            # Lines exist only in A → nothing is inserted to B
            continue

        else:
            raise RuntimeError(f"Unexpected opcode tag: {tag}")

    diff_obj = {
        "meta": {
            "version": 1,
            "description": "Line-based operations from A to B (code only, comments separated)",
        },
        "ops": ops,
    }
    return diff_obj


def main() -> None:
    if len(sys.argv) != 5:
        print(
            "Usage: python make_json_diff_ops.py Origin.asm Modified.asm diff.json comments.txt",
            file=sys.stderr,
        )
        sys.exit(1)

    a_path, b_path, diff_path, comments_path = sys.argv[1:5]

    # Separate code/comment for A and B
    a_codes, _ = read_code_and_comments(a_path)
    b_codes, b_comments = read_code_and_comments(b_path)

    # Build operation list JSON (code-only diff)
    diff_obj = build_ops(a_codes, b_codes)

    # Write diff.json
    with open(diff_path, "w", encoding="utf-8") as f_diff:
        json.dump(diff_obj, f_diff, ensure_ascii=False, indent=2)

    # Write B's comments into comments.txt
    with open(comments_path, "w", encoding="utf-8") as f_com:
        for c in b_comments:
            f_com.write(c + "\n")


if __name__ == "__main__":
    main()
