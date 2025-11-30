#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
SN8F2288 asm を読み込み、各行に対応するアドレスコメントを付けるスクリプト。
前提:
- ORG でアドレスを設定
- 命令 1 行 = 1 ワード
- DW は値の個数だけワード数が増える
"""

import sys
import re
from pathlib import Path


def count_dw_words(dw_body: str) -> int:
    """
    '0035H, 5105H, 2012H' のような DW 部分からワード数を数える。
    かなり素朴なパーサだが、SN8 の典型的な出力なら十分なはず。
    """
    # コメントを落とす
    dw_body = dw_body.split(';', 1)[0]
    # カンマ/空白で分割
    tokens = re.split(r'[,\s]+', dw_body.strip())
    tokens = [t for t in tokens if t]
    return len(tokens)


def add_addresses(in_path: Path, out_path: Path) -> None:
    pc = None  # 現在アドレス（ワードアドレス）

    lines = in_path.read_text(encoding="utf-8").splitlines()
    out_lines = []

    for line in lines:
        original_line = line  # 改行なし
        stripped = line.lstrip()

        # コメントより前の部分
        stripped = stripped.split(";", 1)[0].rstrip()

        addr_comment = ""

        # 1) ORG ディレクティブ
        if stripped.upper().startswith("ORG"):
            # 例: 'ORG 0x0000' or 'ORG 0000H'
            parts = stripped.split()
            if len(parts) >= 2:
                addr_token = parts[1]
                # 0x???? or ????H or ???? のどれかを想定
                addr_token = addr_token.rstrip("Hh")
                if addr_token.lower().startswith("0x"):
                    pc = int(addr_token, 16)
                else:
                    pc = int(addr_token, 16)
            if pc is not None:
                addr_comment = f"; {pc:04x} "

            # ORG 自体は PC を進めない
        elif pc is not None:
            # 2) まだ ORG 前ならアドレスは付けない
            # 3) 純コメント行 or 空行
            if stripped == "" or stripped.startswith(";"):
                # アドレスコメントは付けないし PC も進めない
                pass

            # 4) ラベル行 (func_xxx:, _interrupt_xxx: など)
            elif stripped.endswith(":"):
                addr_comment = f"; {pc:04x} "
                # ラベルは次の命令と同じアドレスなので PC は進めない

            # 5) DW 行
            elif stripped.upper().startswith("DW"):
                addr_comment = f"; {pc:04x} "
                # "DW" の後ろを取り出してワード数カウント
                body = stripped[2:].strip()
                n_words = count_dw_words(body)
                if n_words <= 0:
                    n_words = 1  # 念のため保険
                pc += n_words

            # 6) それ以外は「命令 1 ワード」とみなす
            else:
                addr_comment = f"; {pc:04x} "
                pc += 1

        # アドレスコメントを先頭に付ける（行自体はそのまま温存）
        if addr_comment:
            new_line = addr_comment + original_line
        else:
            new_line = original_line

        out_lines.append(new_line)

    out_path.write_text("\n".join(out_lines) + "\n", encoding="utf-8")


def main(argv=None):
    if argv is None:
        argv = sys.argv[1:]

    if not (1 <= len(argv) <= 2):
        print("Usage: add_sn8_addr.py <input.asm> [output.asm]")
        sys.exit(1)

    in_path = Path(argv[0])
    if len(argv) == 2:
        out_path = Path(argv[1])
    else:
        # デフォルトは foo.asm → foo.addr.asm
        out_path = in_path.with_suffix(".addr.asm")

    add_addresses(in_path, out_path)


if __name__ == "__main__":
    main()
