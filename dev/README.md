# Development guideline

## Environment setup
0. Install [git](https://git-scm.com/install/)
1. Install [Rust](https://rust-lang.org/tools/install/)
2. Install [Dioxus CLI](https://crates.io/crates/dioxus-cli)
3. Install [npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm)
4. Install [Tailwind CSS CLI](https://tailwindcss.com/docs/installation/tailwind-cli)
5. Install [Python](https://www.python.org/) (add path to `python`)
6. Install [NuShell](https://www.nushell.sh/) (add path to `nu`)

## DevStart

1. Clone the repository
   ```clone.sh
   git clone https://github.com/haborite/ku1255-firmware-modifier
   ```

2. Deploy test (in the case of MS Windows)
   ```
   cd ku1255-firmware-modifier
   nu distutils\build-windows.nu
   ```
   The file `ku1255-firmware-modifier-windows.zip` will be created.

3. Dev mode run test (Do it after deploy test)
   ```
   dx config set always-on-top false  # Only first time
   dx serve
   ```
   The GUI app will launch. Click button `Install` and check that firmware installer launches in another window. You can see that several `.bin` and `.asm` files are created under the `firmware` directory after clicking that button.

## Firmware modification flow
In the `firmware` directory, following files will be created after pressing `Install` button:

- `tp_compact_usb_kb_with_trackpoint_fw.exe`: The original Lenovo firmware installer exe downloaded from [Lenovo official page](https://support.lenovo.com/jp/ja/solutions/pd026745). This consists of firmware binary and its installer.
- `fw_org.bin`: Binary of the original firmware. This is extracted from `tp_compact_usb_kb_with_trackpoint_fw.exe`.
- `fw_org.asm`: Original firmware source code. This is disassembled from `fw_org.bin` by `sn8tool/sn8tool.exe (dissn.py)`.
- `fw_fmt.asm`: Formatted firmware source code. This is formatted from `fw_org.bin` by `src/utils/format.rs`.
- `fw_tmp.asm`: Template firmware source code. This is created from `fw_fmt.asm`, `template/diff.json`, and `template/comments.txt`. **This is the file a develeper will modify**. This file includes the modifications by this project, recorded in `template/diff.json`. This file cannot be assembled because it contains home-made placeholders. 
- `fw_mod.asm`: Modified firmware source code. This is created from `fw_tmp.asm` by `src/utils/template.rs` (All the placeholders are replaced by actual values).
- `fw_mod.bin`: Modified firmware binary: This is the final product, which is assembeld from `fw_mod.asm` by `sn8tool/sn8tool.exe (assn.py)`. 

These .bin and .asm files are not directly saved in the project repository. Instead, we use `template/diff.json` and `template/comments.txt` to save modifications.

A developer modifys manually `fw_tmp.asm` to customize the firmware. Use `dev/make_diff.py` to generate `diff.json` and `comments.txt` from differences between `fw_fmt.asm` and `fw_tmp.asm`.

```
cd dev
python make_diff.py ..\firmware\fw_fmt.asm ..\firmware\fw_tmp.asm ..\template\diff.json ..\template\comment
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
 - `src/utils/template.rs` is the placeholder replacement engine. `fn modify_asm_file` in `src/utils/firmware/.rs` defines how the placeholders are customized dinamically according to user-input information.

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
