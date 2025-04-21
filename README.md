[English](README.md) | [日本語](docs/README-ja.md) | 

# KU-1255 Firmware Modifier

A simple GUI tool for customizing the firmware of the **[Lenovo ThinkPad Compact USB Keyboard with TrackPoint](https://support.lenovo.com/jp/ja/solutions/pd026745-thinkpad-compact-usb-keyboard-with-trackpoint-overview-and-service-parts)** (US model: **0B47190**).  
You can remap any key on the keyboard—for example, reassign the `Ctrl` key to the `Fn` key position in the bottom-left corner.

Because changes are written directly to the keyboard's firmware, **no system-side configuration is required**. The layout remains consistent across all connected devices and operating systems.

![GUI Overview](https://github.com/haborite/ku1255-firmware-modifier/blob/main/old_ver/img/gui-overview-new.png)

---

## ✅ System Requirements

- Since the app use the official firmware installer, your system must meet the [software requirements listed here](https://support.lenovo.com/jp/ja/solutions/pd026745-thinkpad-compact-usb-keyboard-with-trackpoint-overview-and-service-parts).
  - Once the firmware is installed, the keyboard works on **most operating systems**, regardless of the one used during installation.
- Internet connection is required at the first run of the app to download the official firmware installer from Lenovo web page.

---

## 🚀 How to Download and Run

1. Download the [latest version](https://github.com/haborite/ku1255-firmware-modifier/releases/latest) of `ku1255-firmware-modifier.zip` from the [Releases](https://github.com/haborite/ku1255-firmware-modifier/releases) page.
2. Extract the downloaded `.zip` file.
3. Launch `ku1255-firmware-modifier.exe`.

---

## 🖥️ Interface Overview

1. **Keyboard Selection**  
   Choose your keyboard model. For US layout, select:  
   `0B47190 (84 keys - ANSI)`

2. **Language Selection**  
   Choose your preferred language. Select `US / English` for a typical US keyboard layout.

3. **Main Layer**  
   Defines the default keymap. Click any key to change it, and select a key you want to newly map from the dropdown.

4. **2nd Layer**  
   Defines key behavior when used with the **Mod** key.  
   - This layer is disabled by default because the Mod key isn’t initially defined in the Main Layer.
   - The Mod key must be assigned in both Main and 2nd layers at the same position.

5. **Load config**  
   Load a previously saved keymap from a `.json` file.

6. **Save config**  
   Save the current keymap to a `.json` file.

7. **Install firmware**  
   Flash the current configuration to the keyboard.  
   Make sure the keyboard is plugged in before proceeding.  
   After installation, unplug and reconnect the keyboard to apply the changes.

---

## 🔧 Example: Swapping Fn and Ctrl Keys

1. Click `Load config` and open the file:  
   `example/Swap-Fn-Ctrl.json`
2. In the **Main Layer**, verify that the `Fn` and `Left Ctrl` keys are swapped.  
   (Swapped keys will be highlighted in blue.)
3. Click `Install firmware`.
4. When the firmware installer launches, click **Start**.
5. After installation finishes, close the installer.
6. Unplug and reconnect the keyboard. The new keymap will take effect.

---

# Acknowledgements
The firmware binary analysis methodology employed in this project is based on the discussion in the following thread
- https://github.com/lentinj/tp-compact-keyboard/issues/32

The reffered table of Usage IDs and names
- https://bsakatu.net/doc/usb-hid-to-scancode/

The app is designed to be extendable to support keyboards for various languages.  
Contributions to add keyboards for your own language are very welcome!

