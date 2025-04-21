KEYMAP_START = 0x73FD0
KEYMAP_END = 0x740D1

FIRMWARE_FILE = 'tp_compact_usb_kb_with_trackpoint_fw.exe'
OUTPUT_CSV_FILE = 'byte_positions.csv'

def extract_byte_positions(firmware_path, output_csv_path):
    with open(firmware_path, 'rb') as f:
        firmware_data = f.read()

    with open(output_csv_path, 'w', encoding='utf-8', newline='') as f:
        f.write("Position,Byte Value\n")
        for i, b in enumerate(firmware_data):
            if i >= KEYMAP_START and i <= KEYMAP_END:
                f.write(f"{hex(i)},{hex(b)}\n")

    print(f"Saved in '{output_csv_path}'.")

extract_byte_positions(FIRMWARE_FILE, OUTPUT_CSV_FILE)
