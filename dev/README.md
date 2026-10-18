# Development guideline

## Environment setup
1. Install [git](https://git-scm.com/install/)
2. Install [Rust](https://rust-lang.org/tools/install/)
3. Install the [Dioxus CLI](https://crates.io/crates/dioxus-cli) matching the `dioxus` version in `Cargo.lock`
   ```
   cargo install dioxus-cli --version 0.7.10 --locked
   ```
4. Linux only: install the system libraries
   ```
   sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev libwayland-dev libudev-dev
   ```
   and the udev rule `packaging/linux/70-ku1255.rules` to flash without root.

Node.js and the Tailwind CLI are not needed: `dx` downloads the Tailwind v3 standalone CLI and generates `public/tailwind.css` (see `Dioxus.toml`).
Python is only needed for the optional firmware development scripts in `dev/`.

## Repository layout
- `src/`: the Dioxus desktop app (UI, settings, HID transport for flashing).
- `crates/ku1255-core/`: GUI-independent logic: SN8 assembler / disassembler, firmware template pipeline, flashing protocol. Unit and parity tests live here.
- `boards/`, `logical_layouts/`, `settings/`, `template/`, `examples/`: data files, embedded into the executable at build time.
- `packaging/`: platform files (udev rule).

## Run, test and package
```
dx serve                                      # run the app in dev mode
cargo test -p ku1255-core                     # core tests
KU1255_INSTALLER=path/to/tp_compact_usb_kb_with_trackpoint_fw.exe cargo test -p ku1255-core
                                              # + parity tests against the official firmware
dx bundle --release --platform desktop --out-dir dist
                                              # installer for the current OS (NSIS / .app+.dmg / AppImage)
```
CI (`.github/workflows/ci.yml`) runs the tests and bundles for Windows, macOS and Linux on every push; the bundles are available as workflow artifacts.

## Data directory
The app writes into a per-user data directory:
- Windows: `%LOCALAPPDATA%\ku1255-firmware-modifier`
- macOS: `~/Library/Application Support/ku1255-firmware-modifier`
- Linux: `~/.local/share/ku1255-firmware-modifier`

Set `KU1255_DATA_DIR` to use another directory (e.g. `KU1255_DATA_DIR=. dx serve` to keep `firmware/` in the repository).

## Firmware modification flow
In the `firmware` directory of the data directory, following files will be created after pressing `Install` button:

- `tp_compact_usb_kb_with_trackpoint_fw.exe`: The original Lenovo firmware installer exe downloaded from [Lenovo official page](https://support.lenovo.com/jp/ja/solutions/pd026745). This consists of firmware binary and its installer.
- `fw_org.bin`: Binary of the original firmware. This is extracted from `tp_compact_usb_kb_with_trackpoint_fw.exe`.
- `fw_org.asm`: Original firmware source code. This is disassembled from `fw_org.bin` by the built-in disassembler (`crates/ku1255-core/src/sn8/disasm.rs`, a Rust port of `dissn8.py`).
- `fw_fmt.asm`: Formatted firmware source code. This is formatted from `fw_org.bin` by `crates/ku1255-core/src/format.rs`.
- `fw_tmp.asm`: Template firmware source code. This is created from `fw_fmt.asm`, `template/diff.json`, and `template/comments.txt`. **This is the file a develeper will modify**. This file includes the modifications by this project, recorded in `template/diff.json`. This file cannot be assembled because it contains home-made placeholders. 
- `fw_mod.asm`: Modified firmware source code. This is created from `fw_tmp.asm` by `crates/ku1255-core/src/template.rs` (All the placeholders are replaced by actual values).
- `fw_mod.bin`: Modified firmware binary: This is the final product, which is assembeld from `fw_mod.asm` by the built-in assembler (`crates/ku1255-core/src/sn8/asm.rs`, a Rust port of `assn8.py`). 

These .bin and .asm files are not directly saved in the project repository. Instead, we use `template/diff.json` and `template/comments.txt` to save modifications.

A developer modifys manually `fw_tmp.asm` to customize the firmware. Use `dev/make_diff.py` to generate `diff.json` and `comments.txt` from differences between `fw_fmt.asm` and `fw_tmp.asm`.

```
cd dev
python make_diff.py ..\firmware\fw_fmt.asm ..\firmware\fw_tmp.asm ..\template\diff.json ..\template\comments.txt
```

## Placeholder format

e.g.)

```
${s/03/004d}
```
```
${e/tp_accel_0/NOP/CALL func_07b7}
```

These are placeholders I defined. They consist of enclosing `${` `}`, separating `/`, and more than three separated values like `${value_1/value_2/value_3}`:

- If the 1st section is `s`, the placeholer means `string`.
    - 2nd section is name of the placeholder. It can be used to specify the place holders from the Rust code.
    - 3rd section is the default value for the place holder. If the main code does not anything on the place holder, the place holder will be replaced by this value.
- If the 1st section is `e`, the place holder means `enum`:
    - 2nd section is name of the placeholder. It can be used to specify the place holders from the Rust code.
    - 3rd section is the `index 0` value for the place holder. If the main code does not anything on the place holder, the place holder will be replaced by this value.
    - The nth sections (n > 3) are `index n-3` values.
 - `crates/ku1255-core/src/template.rs` is the placeholder replacement engine. `fn modify_asm` in `src/utils/firmware.rs` defines how the placeholders are customized dinamically according to user-input information.

## Assembler references

- KU-1255 uses SONiX [SN8F2288](https://www.sonix.com.tw/article-en-1002-3048) chip.
- You can download several references from the [sonix website](https://www.sonix.com.tw/article-en-1002-3048).
- I often use the [instruction guide](https://www.sonix.com.tw/files/1/44B04F2DC46250C6E050007F010027B5) to learn the operations used in SN8.
- [KU-1255 custom open-source firmware](https://github.com/ranma/ku1255cfw) wrote by [@ranma](https://github.com/ranma) is also very helpful to understand the firmware structure. He wrote it from scratch in the clean room method.

## Notes
- In the assembler source code, the program often refer ROM addresses in the program itself (by `MOVC` operation). It means that increase / decrease the number of operation lines easily breaks the program. 
- I have added safety check in the firmware installer, which will check and prevent a keyboard from being flashed an incorrect firmware.
- The most convenient way is to keep the number of operation lines constant between `DW` regions. You can see several pieces of many continuous `NOP` operations (No operation) in the source code. They have been created by improving code efficiency. You can use these `NOP`s as a buffer of the operations to keep the number of lines constant by increasing / decreasing them.
- It is conviniet to work with address annotator when you modify .asm file:
  ```
  python dev/sn8_addr.py add firmware/fw_tmp.asm firmware/fw_tmp.addr.asm
  ```
  If, you add any modifications, update the address annotations:
  ```
  python dev/sn8_addr.py update firmware/fw_tmp.addr.asm
  ```
  After working with `fw_tmp.addr.asm`, `fw_tmp.asm` can be regenerated:
  ```
  python dev/sn8_addr.py strip firmware/fw_tmp.addr.asm firmware/fw_tmp.asm
  ```
