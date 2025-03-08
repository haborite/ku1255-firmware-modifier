import hashlib
import json
import argparse
import csv
import sys

def compute_sha256(file_path):
    """Computes SHA256 hash of a given file."""
    with open(file_path, 'rb') as f:
        return hashlib.sha256(f.read()).hexdigest()

def load_keymap_csv(csv_file):
    """Loads key mapping data from a CSV file."""
    keymap = {}
    with open(csv_file, 'r', encoding='utf-8') as f:
        reader = csv.DictReader(f)
        for row in reader:
            if row['Byte_Offset']:
                keymap[row['HID_Key_Name']] = (int(row['Byte_Offset'], 16), int(row['Scan_Code'], 16))
            else:
                keymap[row['HID_Key_Name']] = (None, int(row['Scan_Code'], 16))
    return keymap

def modify_firmware(original_firmware, modified_firmware, key_remap, keymap_csv, unmapped_keys_csv):
    """
    Modifies the firmware file to swap keys based on given mappings.
    
    :param original_firmware: Path to the original firmware file
    :param modified_firmware: Path to save the modified firmware file
    :param key_remap: Dictionary mapping HID key names to new HID key names
    :param keymap_csv: Path to the CSV file containing key offsets and scan codes
    :param unmapped_keys_csv: Path to the CSV file containing scan codes
    """
    # Load key mapping data from CSV
    keymap = load_keymap_csv(keymap_csv) | load_keymap_csv(unmapped_keys_csv)
    
    # Validate all keys exist in the keymap CSV
    for source_key, target_key in key_remap.items():
        if source_key not in keymap:
            sys.exit(f"Error: Key '{source_key}' not found in keymap CSV.")
        if target_key not in keymap:
            sys.exit(f"Error: Key '{target_key}' not found in keymap CSV.")
    
    # Compute and display SHA256 hash of the original file
    original_sha256 = compute_sha256(original_firmware)
    print(f"Original firmware SHA256: {original_sha256}")
    
    # Read the original firmware
    with open(original_firmware, 'rb') as f:
        data = bytearray(f.read())
    
    # Modify the required bytes
    for source_key, target_key in key_remap.items():
        byte_offset, expected_scan_code = keymap[source_key]
        _, new_scan_code = keymap[target_key]

        if not byte_offset:
            sys.exit(f"Error: Key '{source_key}' is not mapped in the original keyboard.")      
        if data[byte_offset] != expected_scan_code:
            sys.exit(f"Error: Unexpected value at {hex(byte_offset)}. Expected {hex(expected_scan_code)}, found {hex(data[byte_offset])}.")
        
        data[byte_offset] = new_scan_code
    
    # Write the modified firmware
    with open(modified_firmware, 'wb') as f:
        f.write(data)
    
    # Compute and display SHA256 hash of the modified file
    modified_sha256 = compute_sha256(modified_firmware)
    print(f"Modified firmware SHA256: {modified_sha256}")
    
    return modified_sha256

def main():
    parser = argparse.ArgumentParser(description='Modify firmware key mappings.')
    parser.add_argument('--original_firmware', default='tp_compact_usb_kb_with_trackpoint_fw.exe', help='Path to the original firmware file')
    parser.add_argument('--modified_firmware', default='tp_compact_usb_kb_with_trackpoint_fw_mod.exe', help='Path to save the modified firmware file')
    parser.add_argument('--key_remap', default='remaps.json', help='Path to the JSON file containing key remapping')
    parser.add_argument('--keymap_csv', default='keymaps.csv', help='Path to the CSV file containing HID key mapping')
    parser.add_argument('--unmapped_keys_csv', default='keymaps2.csv', help='Path to the CSV file containing HID key mapping of unmapped keys')
    
    args = parser.parse_args()
    
    # Load key mappings from JSON file
    with open(args.key_remap, 'r') as f:
        key_remap = json.load(f)
    
    modify_firmware(args.original_firmware, args.modified_firmware, key_remap, args.keymap_csv, args.unmapped_keys_csv)

if __name__ == '__main__':
    main()
