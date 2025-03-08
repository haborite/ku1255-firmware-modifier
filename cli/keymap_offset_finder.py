import csv
from collections import defaultdict

# Define constants
KEYMAP_START = 0x73FD0
KEYMAP_END = 0x740D0
FIRMWARE_FILE = 'tp_compact_usb_kb_with_trackpoint_fw.exe'

def extract_keymap_offsets(
    firmware_path,
    hid_list_path,
    output_csv="keymaps.csv",
    duplication_log="duplication.log",
    unmapped_keys_csv="keymaps2.csv"
):
    """
    Extracts HID keymap offsets from firmware and saves results into CSV files.
    
    :param firmware_path: Path to the firmware file
    :param hid_list_path: Path to the CSV file containing HID key names and IDs
    :param output_csv: Path to save mapped key offsets
    :param duplication_log: Path to save redundant offsets
    :param unmapped_keys_csv: Path to save HID keys without valid offsets
    """
    # Read HID Key IDs and Names from CSV
    with open(hid_list_path, mode='r', newline='', encoding='utf-8') as file:
        hid_keys = [(row['HID_Key_Name'], int(row['HID_Key_ID'], 16))
                    for row in csv.DictReader(file)]

    # Read firmware data into a dictionary for fast lookup
    with open(firmware_path, 'rb') as f:
        firmware_data = f.read()

    # Dictionary to store byte occurrences
    byte_positions = defaultdict(list)
    for i, byte in enumerate(firmware_data):
        byte_positions[byte].append(i)

    mapped_keys = []
    extra_offsets = []
    unmapped_keys = []

    for key_name, key_id in hid_keys:
        scan_code = key_id ^ 0x5A
        # Get all matching offsets, filter for keymap range
        valid_offsets = [offset for offset in byte_positions.get(scan_code, [])
                         if KEYMAP_START <= offset <= KEYMAP_END]

        if valid_offsets:
            valid_offsets.sort()
            primary_offset = f"0x{valid_offsets[0]:X}"  # Store as hex
            extra_offsets.extend(f"0x{offset:X}" for offset in valid_offsets[1:])

            mapped_keys.append({
                "HID_Key_Name": key_name,
                "HID_Key_ID": f"0x{key_id:02X}",
                "Byte_Offset": primary_offset,
                "Scan_Code": f"0x{scan_code:02X}",
            })
        else:
            unmapped_keys.append({
                "HID_Key_Name": key_name,
                "HID_Key_ID": f"0x{key_id:02X}",
                "Byte_Offset": None,
                "Scan_Code": f"0x{scan_code:02X}",
            })

    # Write mapped key offsets to CSV
    with open(output_csv, mode='w', newline='', encoding='utf-8') as file:
        writer = csv.DictWriter(file, fieldnames=["HID_Key_Name", "HID_Key_ID", "Byte_Offset", "Scan_Code"])
        writer.writeheader()
        writer.writerows(mapped_keys)
    print(f"Keymap offsets saved to {output_csv}")

    # Save unmapped keys to CSV
    with open(unmapped_keys_csv, mode='w', newline='', encoding='utf-8') as file:
        writer = csv.DictWriter(file, fieldnames=["HID_Key_Name", "HID_Key_ID", "Byte_Offset", "Scan_Code"])
        writer.writeheader()
        writer.writerows(unmapped_keys)
    print(f"Unmapped keys saved to {unmapped_keys_csv}")

    # Save extra offsets log
    if extra_offsets:
        with open(duplication_log, mode='w', encoding='utf-8') as log_file:
            log_file.write("Extra valid offsets:\n" + "\n".join(extra_offsets) + "\n")
        print(f"Duplicated offsets were found. Log: {duplication_log}")
    else:
        print("No duplicated offsets were found.")

    return mapped_keys

# Example usage
extract_keymap_offsets(FIRMWARE_FILE, 'hid_list.csv')
