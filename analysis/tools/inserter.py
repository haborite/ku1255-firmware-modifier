import sys

# XOR するキー
XOR_KEY = 0x5A
FW_SIZE = 0x6000  # 0x0000 〜 0x5FFF なので 0x6000 バイト
OFFSET = 472208   # 0x73570 に挿入

def main():
    # 引数チェック
    if len(sys.argv) != 4:
        print("Usage: python script.py <fw.bin> <fw_installer.bin> <output.bin>")
        sys.exit(1)

    fw_bin_path = sys.argv[1]
    fw_installer_path = sys.argv[2]
    output_path = sys.argv[3]

    # fw.bin の読み込みと XOR 変換
    with open(fw_bin_path, "rb") as f:
        fw_data = bytearray(f.read(FW_SIZE))

    fw_encrypted = bytearray(b ^ XOR_KEY for b in fw_data)

    # fw_installer.bin の読み込みと挿入
    with open(fw_installer_path, "rb") as f:
        installer_data = bytearray(f.read())

    # 既存のサイズチェック
    if OFFSET + FW_SIZE > len(installer_data):
        print("Error: fw_installer.bin is too small for insertion.")
        sys.exit(1)

    # 置換処理
    installer_data[OFFSET:OFFSET + FW_SIZE] = fw_encrypted

    # 変更後のファイルを書き出し
    with open(output_path, "wb") as f:
        f.write(installer_data)

    print(f"Modified installer saved as {output_path}")

if __name__ == "__main__":
    main()
