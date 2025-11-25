#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Usage:
    python parse_hid_desc.py hid_report.txt
"""

import re
import sys
from textwrap import indent


def extract_words_from_text(text: str):
    words = []
    for m in re.finditer(r'0x([0-9a-fA-F]{1,4})', text):
        val = int(m.group(1), 16)
        if 0 <= val <= 0xFFFF:
            words.append(val)
    return words


def words_to_le_bytes(words):
    """
    0xABCD -> b'\xCD\xAB'
    """
    b = bytearray()
    for w in words:
        b.append(w & 0xFF)
        b.append((w >> 8) & 0xFF)
    return bytes(b)

MAIN_TAGS = {
    0x8: "Input",
    0x9: "Output",
    0xA: "Collection",
    0xB: "Feature",
    0xC: "End Collection",
}

GLOBAL_TAGS = {
    0x0: "Usage Page",
    0x1: "Logical Minimum",
    0x2: "Logical Maximum",
    0x3: "Physical Minimum",
    0x4: "Physical Maximum",
    0x5: "Unit Exponent",
    0x6: "Unit",
    0x7: "Report Size",
    0x8: "Report ID",
    0x9: "Report Count",
    0xA: "Push",
    0xB: "Pop",
}

LOCAL_TAGS = {
    0x0: "Usage",
    0x1: "Usage Minimum",
    0x2: "Usage Maximum",
    0x3: "Designator Index",
    0x4: "Designator Minimum",
    0x5: "Designator Maximum",
    0x7: "String Index",
    0x8: "String Minimum",
    0x9: "String Maximum",
    0xA: "Delimiter",
}

TYPE_NAMES = {
    0: "Main",
    1: "Global",
    2: "Local",
    3: "Reserved",
}

USAGE_PAGE_NAMES = {
    0x01: "Generic Desktop",
    0x02: "Simulation Controls",
    0x03: "VR Controls",
    0x04: "Sport Controls",
    0x05: "Game Controls",
    0x06: "Generic Device Controls",
    0x07: "Keyboard / Keypad",
    0x08: "LEDs",
    0x09: "Button",
    0x0C: "Consumer",
    0x0D: "Digitizers",
}

GENERIC_DESKTOP_USAGES = {
    0x01: "Pointer",
    0x02: "Mouse",
    0x04: "Joystick",
    0x05: "Game Pad",
    0x06: "Keyboard",
    0x07: "Keypad",
    0x30: "X",
    0x31: "Y",
    0x32: "Z",
    0x33: "Rx",
    0x34: "Ry",
    0x35: "Rz",
    0x38: "Wheel",
}

KEYBOARD_USAGE_PREFIX = "Keyboard"


def decode_input_output_feature_flags(val: int):
    """
    Bit flag -> Human readable string
    Input / Output / Feature 
    """
    flags = []
    flags.append("Data" if (val & 0x01) == 0 else "Constant")
    flags.append("Array" if (val & 0x02) == 0 else "Variable")
    flags.append("Absolute" if (val & 0x04) == 0 else "Relative")
    flags.append("NoWrap" if (val & 0x08) == 0 else "Wrap")
    flags.append("Linear" if (val & 0x10) == 0 else "NonLinear")
    flags.append("PreferredState" if (val & 0x20) == 0 else "NoPreferred")
    flags.append("NoNullPosition" if (val & 0x40) == 0 else "NullState")
    flags.append("NonVolatile" if (val & 0x80) == 0 else "Volatile")
    return ", ".join(flags)


def format_usage(usage_page: int | None, usage: int):
    if usage_page is None:
        return f"Usage 0x{usage:02X} (page unknown)"

    up_name = USAGE_PAGE_NAMES.get(usage_page, f"0x{usage_page:02X}")
    if usage_page == 0x01:    # Generic Desktop
        name = GENERIC_DESKTOP_USAGES.get(usage, f"0x{usage:02X}")
        return f"{up_name}: {name}"
    elif usage_page == 0x07:  # Keyboard
        return f"{up_name}: {KEYBOARD_USAGE_PREFIX} 0x{usage:02X}"
    elif usage_page == 0x0C:  # Consumer
        return f"{up_name}: 0x{usage:02X}"
    else:
        return f"{up_name}: 0x{usage:02X}"


def parse_hid_report_descriptor(data: bytes, start_offset: int = 0):
    i = start_offset
    usage_page = None

    while i < len(data):
        prefix = data[i]
        i += 1

        if prefix == 0xFE:
            if i + 2 > len(data):
                break
            size = data[i]
            long_tag = data[i + 1]
            i += 2
            payload = data[i:i + size]
            i += size
            yield {
                "offset": i - size - 3,
                "raw": bytes([prefix, size, long_tag]) + payload,
                "type": "Long",
                "tag": f"0x{long_tag:02X}",
                "data": payload,
                "desc": f"Long item tag=0x{long_tag:02X}, size={size}",
            }
            continue

        size_code = prefix & 0x03
        type_code = (prefix >> 2) & 0x03
        tag_code = (prefix >> 4) & 0x0F

        size = {0: 0, 1: 1, 2: 2, 3: 4}[size_code]
        item_type_name = TYPE_NAMES.get(type_code, f"Type{type_code}")
        tag_name = None

        if type_code == 0:  # Main
            tag_name = MAIN_TAGS.get(tag_code, f"Tag{tag_code}")
        elif type_code == 1:  # Global
            tag_name = GLOBAL_TAGS.get(tag_code, f"Tag{tag_code}")
        elif type_code == 2:  # Local
            tag_name = LOCAL_TAGS.get(tag_code, f"Tag{tag_code}")
        else:
            tag_name = f"Tag{tag_code}"

        if i + size > len(data):
            break
        payload = data[i:i + size]
        i += size

        desc = f"{item_type_name}: {tag_name}"
        value = None

        if size == 1:
            value = int.from_bytes(payload, "little", signed=False)
            sval = int.from_bytes(payload, "little", signed=True)
        elif size == 2:
            value = int.from_bytes(payload, "little", signed=False)
            sval = int.from_bytes(payload, "little", signed=True)
        elif size == 4:
            value = int.from_bytes(payload, "little", signed=False)
            sval = int.from_bytes(payload, "little", signed=True)
        else:
            value = None
            sval = None

        if item_type_name == "Global" and tag_name == "Usage Page" and value is not None:
            usage_page = value
            up_name = USAGE_PAGE_NAMES.get(value, f"0x{value:02X}")
            desc += f" = {up_name} (0x{value:02X})"
        elif item_type_name == "Local" and tag_name in ("Usage", "Usage Minimum", "Usage Maximum") and value is not None:
            desc += f" = {format_usage(usage_page, value)}"
        elif item_type_name == "Global" and tag_name in ("Logical Minimum", "Logical Maximum",
                                                         "Physical Minimum", "Physical Maximum",
                                                         "Report Size", "Report ID", "Report Count",
                                                         "Unit Exponent", "Unit") and value is not None:
            desc += f" = {sval} (0x{value:0{size*2}X})"
        elif item_type_name == "Main" and tag_name in ("Input", "Output", "Feature") and value is not None:
            desc += f" = 0x{value:02X} ({decode_input_output_feature_flags(value)})"
        elif item_type_name == "Main" and tag_name == "Collection" and value is not None:
            coll_type_names = {
                0x00: "Physical",
                0x01: "Application",
                0x02: "Logical",
                0x03: "Report",
                0x04: "Named Array",
                0x05: "Usage Switch",
                0x06: "Usage Modifier",
            }
            cname = coll_type_names.get(value, f"0x{value:02X}")
            desc += f" = {cname} (0x{value:02X})"
        elif item_type_name == "Main" and tag_name == "End Collection":
            pass
        elif size > 0 and value is not None:
            desc += f" = {sval} (0x{value:0{size*2}X})"

        yield {
            "offset": i - size - 1,
            "raw": bytes([prefix]) + payload,
            "type": item_type_name,
            "tag": tag_name,
            "data": payload,
            "desc": desc,
        }


def find_hid_report_start(data: bytes):
    pattern = b"\x05\x01\x09\x06\xA1\x01"
    idx = data.find(pattern)
    if idx == -1:
        return None
    return idx


def main():
    if len(sys.argv) != 2:
        print("Usage: python parse_thinkpad_hid.py INPUT.txt", file=sys.stderr)
        sys.exit(1)

    path = sys.argv[1]
    with open(path, "r", encoding="utf-8") as f:
        text = f.read()

    words = extract_words_from_text(text)
    print(f"# Extracted {len(words)} words (16bit)")

    data = words_to_le_bytes(words)
    print(f"# Total bytes = {len(data)}")
    print()

    print("## First 256 bytes (little-endian hex):")
    hex_str = " ".join(f"{b:02X}" for b in data[:256])
    print(indent(hex_str, "  "))
    print()

    start = find_hid_report_start(data)
    if start is None:
        print("!! Could not find HID report descriptor signature (05 01 09 06 A1 01)")
        sys.exit(0)

    print(f"## HID Report Descriptor detected at byte offset {start}")
    print()

    print("## Parsed HID Report Descriptor items:")
    for item in parse_hid_report_descriptor(data, start_offset=start):
        raw_hex = " ".join(f"{b:02X}" for b in item["raw"])
        print(f"  @0x{item['offset']:04X}: {raw_hex}")
        print(f"    {item['desc']}")
    print()


if __name__ == "__main__":
    main()
