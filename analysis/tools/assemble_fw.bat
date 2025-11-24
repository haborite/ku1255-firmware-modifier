python assn8 fw_mod.asm -o fw_mod.bin
:: python bin2txt.py fw_mod.bin fw_mod.txt
python inserter.py fw_mod.bin tp_compact_usb_kb_with_trackpoint_fw.exe fw_mod.exe
:: certutil -hashfile fw_mod_4.exe SHA256