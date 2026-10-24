[English](README.md) | [日本語](docs/README-ja.md) | [简体中文](docs/README-zh.md) |

# KU-1255 Firmware Modifier

A simple GUI tool for customizing the firmware of the **[Lenovo ThinkPad Compact USB Keyboard with TrackPoint](https://support.lenovo.com/jp/ja/solutions/pd026745-thinkpad-compact-usb-keyboard-with-trackpoint-overview-and-service-parts)**.  
You can remap any key on the keyboard—for example, reassign the `Ctrl` key to the `Fn` key position in the bottom-left corner. 

In addition, several advanced customization features are available:
- Multi Layers: Change key behavior when pressed together with the Mod key.
- Key Macros: Replace combinations of Ctrl, Shift, Alt, and Win keys with a single key press.
- Media Keys: Assign special functions such as volume control or media playback controls.
- TrackPoint Speed: Increase the TrackPoint acceleration beyond the limits of the official Lenovo driver.

Since all modifications are written directly to the keyboard's firmware, **no system-side configuration is required**. The layout remains consistent across all connected devices and operating systems.

![GUI Overview](https://github.com/haborite/ku1255-firmware-modifier/blob/main/docs/gui-overview.png)

---

## 📜 Compatible Models
**[Lenovo ThinkPad Compact USB Keyboard with TrackPoint (KU-1255)](https://support.lenovo.com/us/en/solutions/pd026745-thinkpad-compact-usb-keyboard-with-trackpoint-overview-and-service-parts)**

**Part Number** : 0B47190, 0B47191, 0B47192, 0B47194, 0B47195, 0B47197, 0B47198, 0B47200, 0B47201, 0B47202, 0B47204, 0B47205, 0B47206, 0B47207, 0B47208, 0B47209, 0B47210, 0B47211, 0B47212, 0B47213, 0B47215, 0B47216, 0B47217, 0B47218, 0B47219, 0B47220, 0B47221, 0B47222, 0B47223, 0B47224, 0B47225

## ✅ System Requirements

- **Windows** 10 / 11 (x64)
- **Linux** (x64): Debian / Ubuntu and derivatives via the `.deb` package, other distributions via the AppImage
- **macOS**: not supported yet (an Apple Silicon build exists but flashing has not been verified)
- Once the firmware is installed, the keyboard works on **most operating systems**, regardless of the one used during installation.
- Internet connection is required at the first run of the app to download the official firmware installer from Lenovo web page.

## 🚀 How to Download and Run

Download the file for your OS from the [latest release](https://github.com/haborite/ku1255-firmware-modifier/releases/latest).

**Windows (portable)**
1. Download `*_x64-portable.zip` and extract it.
2. Run `ku1255-firmware-modifier.exe`. No installation is needed.
    - If you see "Windows protected your PC" (Microsoft Defender SmartScreen), click "More info" and then "Run anyway".
    - The app needs the WebView2 runtime, which ships with Windows 10/11. If the app does not start, install the runtime from [Microsoft](https://developer.microsoft.com/microsoft-edge/webview2/), or use the installer below.

**Windows (installer)**
1. Download and run the installer (`*-setup.exe`).
    - If the WebView2 runtime is missing, the installer downloads it.
2. Launch **Ku1255FirmwareModifier** from the Start menu.

**Linux (Debian / Ubuntu)**
1. Download the `.deb` file and install it:
    ```
    sudo apt install ./ku1255-firmware-modifier_*_amd64.deb
    ```
    This also installs a udev rule so that you can flash the keyboard without root.
2. Unplug and reconnect the keyboard, then launch the app from the application menu.

**Linux (other distributions)**
1. Download the `.AppImage` file and make it executable: `chmod +x ku1255-firmware-modifier*.AppImage`
2. Install the [udev rule](packaging/linux/70-ku1255.rules) to flash without root:
    ```
    sudo cp 70-ku1255.rules /etc/udev/rules.d/
    sudo udevadm control --reload && sudo udevadm trigger
    ```
3. Run the AppImage.

**Where files are stored**  
The downloaded official installer, example configs and intermediate firmware files are kept in:
- Windows: `%LOCALAPPDATA%\ku1255-firmware-modifier`
- Linux: `~/.local/share/ku1255-firmware-modifier`

## 🖥️ Interface Overview

![Interface Overview](https://github.com/haborite/ku1255-firmware-modifier/blob/main/docs/interface-overview.png)

1. **Keyboard Selection**  
   Choose your keyboard model. For US layout, select: `0B47190 (84 keys - ANSI)`

2. **Language Selection**  
   Choose your preferred language. Select `US / English` for a typical US keyboard layout.

3. **Main Layer**  
   Defines the default keymap. Click any key to change it, and select a key you want to newly map from the dropdown.

4. **2nd Layer**  
   Defines key behavior when used with the **Mod** key.  
   - This layer is disabled by default because the Mod key isn’t initially mapped in the Main Layer.
   - The Mod key must be assigned in both Main and 2nd layers at the same position.

5. **Macro Keys**  
   Create key macros consisting of combinations of Ctrl, Shift, Alt, and Win keys (up to 24 macros).

6. **Media Keys**  
   Configure media keys such as volume control and display brightness (up to 11 functions).

7. **TrackPoint Settings**  
   Set the trackpoint speed settings. You can fully control the relashionships between trackpoint tilt amount and cursor speed. You can also use easy preset settings.
   Speed 1 to 9 correspond to the 9-level speed setting in the driver control panel. OS without official driver installation will use Speed 5.
   ![Trackpoint Settings](https://github.com/haborite/ku1255-firmware-modifier/blob/main/docs/trackpoint-overview.png)

8. **Other settings**
   Other useful settings (e.g. Assign Fn-key functionality in addition to the original behavior of any selected key.)

9. **Load config**  
   Load a previously saved keymap from a `.json` file. The dialog opens in the `examples` folder of the data directory.

10. **Save config**  
   Save the current keymap to a `.json` file.

11. **Install firmware**  
   Build the firmware and open the install dialog. Press **Start** to flash the keyboard.  
   Make sure the keyboard is plugged in, and do not unplug it while flashing.  
   After installation, unplug and reconnect the keyboard to apply the changes.

## 🔧 Example: Swapping Fn and Ctrl Keys

1. Click `Load config` and open the file: `examples/Swap-Fn-Ctrl.json`
2. Select keyboard you use in the `Keyboard` selection.
3. In the **Main Layer**, verify that the `Fn` and `Left Ctrl` keys are swapped.  
   (Swapped keys will be highlighted in blue.)
4. Click `Install firmware`.
5. In the install dialog, click **Start**.
6. After "Flashing finished" is shown, click **Finish**.
7. Unplug and reconnect the keyboard. The new keymap will take effect.

## ♻️ Firmware recovery
- Firmware can be installed into the keyboard repeatedly.
- If flashing fails midway, the keyboard starts in its firmware update mode; run the installation again (**Retry**).
- To restore your device to its factory firmware settings, use the [Lenovo Official Firmware Installer](https://support.lenovo.com/us/en/solutions/pd026745-thinkpad-compact-usb-keyboard-with-trackpoint-overview-and-service-parts).

## 🧪 Custom firmware template (advanced)
The firmware is built from a template bundled in the app (`template/diff.json` and `template/comments.txt`).
To use your own template, put it in the `template` folder of the data directory (see **Where files are stored**):
- `template/diff.json` (required)
- `template/comments.txt` (optional; the bundled one is used if absent)

While a custom `diff.json` is present, the install dialog shows a warning. Delete it to go back to the bundled template.
A custom template is not updated with the app, and a broken one can produce firmware that does not work; restore with the official Lenovo installer if needed.
See the [development guide](dev/README.md) for how to make one.

---

# Development
See https://github.com/haborite/ku1255-firmware-modifier/tree/main/dev#readme

# License
This project is licensed under the [MIT](LICENSE).

# Acknowledgements
The firmware binary analysis methodology employed in this project is based on the discussion in the following thread
- https://github.com/lentinj/tp-compact-keyboard/issues/32
- https://hohlerde.org/rauch/en/elektronik/projekte/tpkbd-fix

The reffered table of Usage IDs and names
- https://bsakatu.net/doc/usb-hid-to-scancode/

The app is designed to be extendable to support keyboards for various languages.  
Contributions to add keyboards for your own language are very welcome!










