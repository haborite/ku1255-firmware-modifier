#!/usr/bin/env python3
import sys
import os

# ----------------------------------------------------------------------
# Utility Functions
# ----------------------------------------------------------------------

def convert_leading_tabs_to_spaces(line: str, tab_as_spaces: int = 4) -> str:
    """
    Convert leading tabs into a fixed number of spaces (default: 4).
    Tabs appearing after the leading indentation are replaced with a single space.
    """
    i = 0
    new_prefix = []
    while i < len(line) and line[i] == "\t":
        new_prefix.append(" " * tab_as_spaces)
        i += 1
    rest = line[i:].replace("\t", " ")
    return "".join(new_prefix) + rest


def first_word(line: str) -> str:
    """Return the first token of a line, or empty string for blank lines."""
    stripped = line.strip()
    if not stripped:
        return ""
    return stripped.split()[0]


# ----------------------------------------------------------------------
# Main Processing
# ----------------------------------------------------------------------

def main():
    # ------------------------------------------------------------
    # 1. Argument parsing
    # ------------------------------------------------------------
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} INPUT_FILE [OUTPUT_FILE]")
        sys.exit(1)

    in_path = sys.argv[1]

    if not os.path.isfile(in_path):
        print(f"Error: file not found: {in_path}")
        sys.exit(1)
    
    if len(sys.argv) >= 3:
        out_path = sys.argv[2]
    else:
        root, ext = os.path.splitext(in_path)
        out_path = root + "_out" + ext

    # ------------------------------------------------------------
    # 2. Load input lines
    # ------------------------------------------------------------
    with open(in_path, encoding="utf-8") as f:
        raw_lines = f.readlines()

    # ==================================================================
    # 3. Preprocessing
    # ==================================================================
    cleaned_lines = []
    for raw in raw_lines:
        line = raw.rstrip("\r\n")

        if line.lstrip().startswith(";"):
            continue

        if ";" in line:
            line = line[: line.index(";")]

        line = convert_leading_tabs_to_spaces(line)
        line = line.rstrip(" ")

        if not line:
            continue

        cleaned_lines.append(line)

    if not cleaned_lines:
        print("Warning: no lines left after processing.")
        with open(out_path, "w", encoding="utf-8"):
            pass
        print(f"Empty output written to: {out_path}")
        return

    # ==================================================================
    # 4. Parse instruction structure & determine alignment columns
    # ==================================================================
    structured = []
    candidate_widths = []

    for line in cleaned_lines:
        i = 0
        while i < len(line) and line[i] == " ":
            i += 1
        indent = line[:i]
        rest = line[i:]

        if not rest:
            structured.append({"raw": line, "candidate": False})
            continue

        parts = rest.split()
        if len(parts) < 2:
            structured.append({"raw": line, "candidate": False})
            continue

        mnemonic = parts[0]
        operand_str = " ".join(parts[1:]).strip()

        op1 = operand_str
        op2 = None
        if "," in operand_str:
            left, right = operand_str.split(",", 1)
            op1 = left.strip()
            op2 = right.strip() if right.strip() else None

        width = len(indent) + len(mnemonic)
        candidate_widths.append(width)

        structured.append({
            "indent": indent,
            "mnemonic": mnemonic,
            "op1": op1,
            "op2": op2,
            "candidate": True,
        })

    # ==================================================================
    # 5. Align operands
    # ==================================================================
    if candidate_widths:
        max_width = max(candidate_widths)
        operand_col = max_width + 1

        aligned_lines = []
        for item in structured:
            if not item["candidate"]:
                aligned_lines.append(item["raw"])
                continue

            indent   = item["indent"]
            mnemonic = item["mnemonic"]
            op1      = item["op1"]
            op2      = item["op2"]

            base_width = len(indent) + len(mnemonic)
            spaces_between = max(1, operand_col - base_width)

            line = indent + mnemonic + (" " * spaces_between) + op1

            if op2 is not None:
                line += ","
                if len(op1) < 6:
                    line += " " * (6 - len(op1))
                line += op2

            aligned_lines.append(line)
    else:
        aligned_lines = cleaned_lines

    # ==================================================================
    # 6. Insert blank lines before func_* labels
    # ==================================================================
    no_blank_prev = {"RET", "DW", "CALL", "JMP"}
    second_prev_ok = {"CMPRS", "B0BTS0", "B0BTS1", "BTS0", "BTS1"}

    with_blank_lines = []
    for idx, line in enumerate(aligned_lines):
        stripped = line.strip()

        is_func_label = (
            stripped.startswith("func_")
            and stripped.endswith(":")
            and " " not in stripped
        )

        if is_func_label:
            need_blank = True

            if idx == 0:
                need_blank = False
            else:
                prev_first = first_word(aligned_lines[idx - 1])
                cond1 = prev_first not in no_blank_prev

                cond2 = False
                if prev_first in no_blank_prev and idx >= 2:
                    prev2_first = first_word(aligned_lines[idx - 2])
                    cond2 = prev2_first in second_prev_ok

                if cond1 or cond2:
                    need_blank = False

            if need_blank:
                with_blank_lines.append("")

        with_blank_lines.append(line)

    # ==================================================================
    # 7. Align semicolons
    # ==================================================================
    non_empty = [ln for ln in with_blank_lines if ln != ""]
    max_len = max(len(ln) for ln in non_empty) if non_empty else 0

    final_lines = []
    for line in with_blank_lines:
        if line == "":
            final_lines.append("\n")
        else:
            spaces = (max_len - len(line)) + 1
            final_lines.append(line + (" " * spaces) + ";" + "\n")

    # ==================================================================
    # 8. Save
    # ==================================================================
    with open(out_path, "w", encoding="utf-8", newline="\n") as f:
        f.writelines(final_lines)

    print(f"Saved to: {out_path}")


if __name__ == "__main__":
    main()
