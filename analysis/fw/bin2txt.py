import sys

def binary_to_hex(input_path, output_path):
    try:
        with open(input_path, "rb") as f_in, open(output_path, "w") as f_out:
            offset = 0
            while True:
                data = f_in.read(16)  # 16バイトずつ読み込む
                if not data:
                    break  # ファイルの終わりに達したら終了

                hex_line = f"{offset:08X} | " + " ".join(f"{b:02X}" for b in data)
                f_out.write(hex_line + "\n")
                offset += 16

        print(f"バイナリファイルを16進数表記で保存しました: {output_path}")

    except FileNotFoundError:
        print(f"エラー: ファイルが見つかりません。")
    except Exception as e:
        print(f"エラー: {e}")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(f"使い方: python {sys.argv[0]} <入力バイナリファイル> <出力テキストファイル>")
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = sys.argv[2]
    binary_to_hex(input_file, output_file)