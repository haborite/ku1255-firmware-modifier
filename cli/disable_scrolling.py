import sys
import struct
import hashlib

def patch_firmware(original_file, modified_file):
    # Define the offsets to modify
    offsets = [476076, 475710, 475718, 481870]
    patch_value = struct.pack('<H', 0x5A5A)  # Little-endian 0x5A5A
    
    # Read the original file
    with open(original_file, 'rb') as f:
        data = bytearray(f.read())
    
    # Apply the patch
    for offset in offsets:
        data[offset:offset+2] = patch_value
    
    # Write the modified file
    with open(modified_file, 'wb') as f:
        f.write(data)
    
    # Calculate and print SHA-256 hash of the modified file
    sha256_hash = hashlib.sha256(data).hexdigest()
    print(f"SHA-256 of modified file: {sha256_hash}")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python script.py <original_file> <modified_file>")
        sys.exit(1)
    
    original_file = sys.argv[1]
    modified_file = sys.argv[2]
    patch_firmware(original_file, modified_file)
