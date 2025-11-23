python extract_sn8.py tp_compact_usb_kb_with_trackpoint_fw.exe fw.bin
python dissn8 -c .\sn8\sn8f2288.cfg fw.bin -o fw.asm
python format_asm_source.py fw.asm fw_fmt.asm
python apply_diff.py fw_fmt.asm diff.json comments.txt fw_mod.asm