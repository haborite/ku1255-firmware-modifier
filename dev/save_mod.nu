^python sn8_addr.py ..\firmware\fw_fmt.addr.asm ..\firmware\fw_fmt.asm
^python make_diff.py ..\firmware\fw_fmt.asm ..\firmware\fw_tmp.asm ..\template\diff.json ..\template\comments.txt