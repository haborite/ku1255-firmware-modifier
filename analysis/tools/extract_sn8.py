import sys

def extract_sn8(exe_path, output_bin_path, sn8_offset=472208, sn8_size=24576, xor_key=0x5A):
    with open(exe_path, "rb") as f:
        f.seek(sn8_offset)
        exe_offset = f.tell()
        encrypted_sn8 = f.read(sn8_size)

    decrypted_sn8 = bytes(b ^ xor_key for b in encrypted_sn8)

    with open(output_bin_path, "wb") as f:
        f.write(decrypted_sn8)

    print(f"Extracted SN8 file saved as binary: {output_bin_path}")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <fw_installer.exe> <output.bin>")
        sys.exit(1)

    extract_sn8(sys.argv[1], sys.argv[2])
