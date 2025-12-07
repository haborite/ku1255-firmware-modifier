# patch_bytes.py
# Usage:
#     python patch_bytes.py input.bin output.bin

import sys

ADDRESSES = [475710, 475718, 476076, 481870]
PATCH_VALUE = 0x5A


def main():
    if len(sys.argv) != 3:
        print("Usage: python patch_bytes.py <input.bin> <output.bin>")
        sys.exit(1)

    in_file = sys.argv[1]
    out_file = sys.argv[2]

    # Read entire binary
    with open(in_file, "rb") as f:
        data = bytearray(f.read())

    # Sanity check
    file_len = len(data)
    for addr in ADDRESSES:
        if addr >= file_len:
            print(f"Error: address {addr} is outside file size {file_len}")
            sys.exit(1)

    # Patch each address
    for addr in ADDRESSES:
        data[addr] = PATCH_VALUE
        data[addr + 1] = PATCH_VALUE

    # Write out modified binary
    with open(out_file, "wb") as f:
        f.write(data)

    print(f"Patched file saved to: {out_file}")


if __name__ == "__main__":
    main()
