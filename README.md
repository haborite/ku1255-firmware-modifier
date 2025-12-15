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
**[Lenovo ThinkPad Compact USB Keyboard with TrackPoint (KU-1255)](https://support.lenovo.com/jp/ja/solutions/pd026745-thinkpad-compact-usb-keyboard-with-trackpoint-overview-and-service-parts)**

**Part Number** : 0B47190, 0B47191, 0B47192, 0B47194, 0B47195, 0B47197, 0B47198, 0B47200, 0B47201, 0B47202, 0B47204, 0B47205, 0B47206, 0B47207, 0B47208, 0B47209, 0B47210, 0B47211, 0B47212, 0B47213, 0B47215, 0B47216, 0B47217, 0B47218, 0B47219, 0B47220, 0B47221, 0B47222, 0B47223, 0B47224, 0B47225

## ✅ System Requirements

- Currently the app only works on MS Windows, but macOS and Linux versions can be developed upon request.
- Microsoft Visual C++ Redistributable is required in MS Windows.
- Once the firmware is installed, the keyboard works on **most operating systems**, regardless of the one used during installation.
- Internet connection is required at the first run of the app to download the official firmware installer from Lenovo web page.

## 🚀 How to Download and Run

1. Download the [latest version](https://github.com/haborite/ku1255-firmware-modifier/releases/latest) of `ku1255-firmware-modifier.zip` from the [Releases](https://github.com/haborite/ku1255-firmware-modifier/releases/latest) page.
2. Extract the downloaded `.zip` file.
3. Launch `ku1255-firmware-modifier.exe`.
    - If you see a warning saying "Windows protected your PC" and "Microsoft Defender SmartScreen prevented an unrecognized app from starting", click "More info" and then select "Run anyway" to proceed.

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

7. **TrackPoint Speed**  
   Set the trackpoint speed (default: 1). This has nothing to do with Lenovo driver settings or OS mouse settings. It is better to adjust these two settings first before modifying this firmware.

8. **Enable middle button click**  
   Enable middle button click (just like on a standard mouse) even with the official driver on MS Windows.

9. **Fn / Media Trigger**  
   Assign Fn-key functionality in addition to the original behavior of any selected key.

10. **Load config**  
   Load a previously saved keymap from a `.json` file.

11. **Save config**  
   Save the current keymap to a `.json` file.

12. **Install firmware**  
   Flash the current configuration to the keyboard.  
   Make sure the keyboard is plugged in before proceeding.  
   After installation, unplug and reconnect the keyboard to apply the changes.

## 🔧 Example: Swapping Fn and Ctrl Keys

1. Click `Load config` and open the file: `example/Swap-Fn-Ctrl.json`
2. In the **Main Layer**, verify that the `Fn` and `Left Ctrl` keys are swapped.  
   (Swapped keys will be highlighted in blue.)
3. Click `Install firmware`.
4. When the firmware installer launches, click **Start**.
5. After installation finishes, close the installer.
6. Unplug and reconnect the keyboard. The new keymap will take effect.

## Limitation of combination keys

The key matrix of KU-1255 is shown below (written in the logical layout of US ANSI).
When pressing three keys at once, if one key shares both the row and column with the other two, the last pressed key is not be recognized (to prevent ghost key).
This comes purely from the physical key matrix limitation. Thus there is nothing we can do from the firmware or driver.

| col1 | col2 | col3 | col4 | col5 | col6 | col7 | col8 | col9 | col10 | col11 | col12 | col13 | col14 | col15 | col16 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ` and ~ | F1 | F2 | 5 | 6 | = and + | F8 | - | F9 |  | Home |  | Del | Left Ctrl |  |  |
| 1 | 2 | 3 | 4 | 7 | 8 | 9 | 0 | F10 | End | F11 | F12 | Insert |  |  |  |
| TAB | CapsLock | F3 | T | Y | ] and } | F7 | [ and { | BackSpace |  |  | Left Win |  |  | Left Shift |  |
| Q | W | E | R | U | I | O | P | International3 |  |  |  |  |  |  |  |
| A | S | D | F | J | K | L | ; and : |  | Fn |  |  | PrtSc |  |  |  |
| Esc |  | F4 | G | H | F6 | International4 | ' and " | F5 | ↑ |  |  |  |  |  | Left Alt |
| Z | X | C | V | M | , | . | NonーUS # and ~ | Enter |  |  |  | PgUp | Rigth Ctrl | Right Shift |  |
| International5 |  |  | B | N | International1 | International2 | / and ? | Space | ← | ↓ | → | PgDn |  |  | Right Alt |

---

# Acknowledgements
The firmware binary analysis methodology employed in this project is based on the discussion in the following thread
- https://github.com/lentinj/tp-compact-keyboard/issues/32

The reffered table of Usage IDs and names
- https://bsakatu.net/doc/usb-hid-to-scancode/

The app is designed to be extendable to support keyboards for various languages.  
Contributions to add keyboards for your own language are very welcome!





