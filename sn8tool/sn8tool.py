#!/usr/bin/env python3
import sys

# ["sn8tool.py", "dissn8", "fw.bin", "-o", ...] → ["dissn8", "fw.bin", "-o", ...]
def run_dissn8():
    from sn8.dissn8 import main as dissn8_main
    sys.argv = ["dissn8"] + sys.argv[2:]
    dissn8_main()


# ["sn8tool.py", "assn8", "fw.asm", "-o", ...] → ["assn8", "fw.asm", "-o", ...]
def run_assn8():
    from sn8.assn8 import main as assn8_main
    sys.argv = ["assn8"] + sys.argv[2:]
    assn8_main()


# ["sn8tool.py", "flashsn8-gui", "fw.bin"] → ["flashsn8-gui", "fw.bin"]
def run_flashsn8_gui():
    from sn8.flashsn8_gui import main as flash_main
    sys.argv = ["flashsn8-gui"] + sys.argv[2:]
    flash_main()


def main():
    if len(sys.argv) < 2:
        print("Usage:")
        print("  python sn8tool.py dissn8        <args...>")
        print("  python sn8tool.py assn8         <args...>")
        print("  python sn8tool.py flashsn8-gui  <args...>")
        sys.exit(1)

    cmd = sys.argv[1]

    if cmd == "dissn8":
        run_dissn8()
    elif cmd == "assn8":
        run_assn8()
    elif cmd == "flashsn8-gui":
        run_flashsn8_gui()
    else:
        print(f"[ERROR] Unknown command: {cmd}")
        print("Available commands: dissn8, assn8, flashsn8-gui")
        sys.exit(1)


if __name__ == "__main__":
    main()
