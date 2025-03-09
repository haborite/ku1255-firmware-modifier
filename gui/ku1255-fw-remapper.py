import sys
import json
import hashlib
from PySide6.QtWidgets import (
    QApplication, QWidget, QVBoxLayout, QPushButton, QFileDialog, QLabel,
    QLineEdit, QTextEdit, QMessageBox, QFrame
)

KEYMAPS = {
    "A": (0x73FDA, 0x5E),
    "B": (0x7401E, 0x5F),
    "C": (0x73FEC, 0x5C),
    "D": (0x73FEA, 0x5D),
    "E": (0x73FE6, 0x52),
    "F": (0x7401A, 0x53),
    "G": (0x74012, 0x50),
    "H": (0x74022, 0x51),
    "I": (0x74036, 0x56),
    "J": (0x7402A, 0x57),
    "K": (0x7403A, 0x54),
    "L": (0x7404A, 0x55),
    "M": (0x7402C, 0x4A),
    "N": (0x7402E, 0x4B),
    "O": (0x74046, 0x48),
    "P": (0x74056, 0x49),
    "Q": (0x73FD6, 0x4E),
    "R": (0x74016, 0x4F),
    "S": (0x73FFA, 0x4C),
    "T": (0x74020, 0x4D),
    "U": (0x74026, 0x42),
    "V": (0x7401C, 0x43),
    "W": (0x73FF6, 0x40),
    "X": (0x73FFC, 0x41),
    "Y": (0x74030, 0x46),
    "Z": (0x73FDC, 0x47),
    "1": (0x73FD8, 0x44),
    "2": (0x73FF8, 0x45),
    "3": (0x73FE8, 0x7A),
    "4": (0x74018, 0x7B),
    "5": (0x74014, 0x78),
    "6": (0x74024, 0x79),
    "7": (0x74028, 0x7E),
    "8": (0x74038, 0x7F),
    "9": (0x74048, 0x7C),
    "0": (0x74058, 0x7D),
    "Return Enter": (0x7407C, 0x72),
    "Escape": (0x73FD2, 0x73),
    "Delete": (0x74080, 0x70),
    "Tab": (0x73FE0, 0x71),
    "Spacebar": (0x7407E, 0x76),
    "Dash and Underscore": (0x74054, 0x77),
    "Equals and Plus": (0x74034, 0x74),
    "Left Brace": (0x74060, 0x75),
    "Right Brace": (0x74040, 0x6A),
    "Pipe and Slash": (0x7407A, 0x6B),
    "Non-US": (0x7405C, 0x68),
    "SemiColon and Colon": (0x7405A, 0x69),
    "Apostrophe and Double Quotation Mark": (0x74052, 0x6E),
    "Grave Accent and Tilde": (0x73FD4, 0x6F),
    "Comma": (0x7403C, 0x6C),
    "Period": (0x7404C, 0x6D),
    "QuestionMark": (0x7405E, 0x62),
    "Caps Lock": (0x74000, 0x63),
    "F1": (0x73FF4, 0x60),
    "F2": (0x73FE4, 0x61),
    "F3": (0x73FF0, 0x66),
    "F4": (0x73FE2, 0x67),
    "F5": (0x74072, 0x64),
    "F6": (0x74032, 0x65),
    "F7": (0x74050, 0x1A),
    "F8": (0x74044, 0x1B),
    "F9": (0x74074, 0x18),
    "F10": (0x74078, 0x19),
    "F11": (0x74088, 0x1E),
    "F12": (0x740A8, 0x1F),
    "PrintScreen": (0x740CA, 0x1C),
    "Pause": (0x740BC, 0x12),
    "Insert": (0x740C8, 0x13),
    "Home": (0x74084, 0x10),
    "PageUp": (0x740CC, 0x11),
    "Delete Forward": (0x740C4, 0x16),
    "End": (0x740B8, 0x17),
    "PageDown": (0x740CE, 0x14),
    "RightArrow": (0x740AE, 0x15),
    "LeftArrow": (0x740BE, 0xA),
    "DownArrow": (0x7408E, 0xB),
    "UpArrow": (0x740B2, 0x8),
    "Non-US Slash Bar": (0x73FF2, 0x3E),
    "International1": (0x7403E, 0xDD),
    "International2": (0x7404E, 0xD2),
    "International3": (0x74076, 0xD3),
    "International4": (0x74042, 0xD0),
    "International5": (0x73FDE, 0xD1),
    "LeftControl": (0x74004, 0xBA),
    "LeftShift": (0x74070, 0xBB),
    "LeftAlt": (0x74092, 0xB8),
    "Left GUI": (0x740B0, 0xB9),
    "RightControl": (0x7400C, 0xBE),
    "RightShift": (0x7406C, 0xBF),
    "RightAlt": (0x7409E, 0xBC),
    "Mute": (0x74092, 0xB8),
    "Fn": (0x740BA, 0xF5),
    "System Power Down": (None, 0xDB),
    "System Sleep": (None, 0xD8),
    "System Wake Up": (None, 0xD9),
    "ErrorRollOver": (None, 0x5B),
    "Scroll Lock": (None, 0x1D),
    "Keypad Num Lock and Clear": (None, 0x9),
    "Keypad Forward Slash": (None, 0xE),
    "Keypad Star": (None, 0xF),
    "Keypad Dash": (None, 0xC),
    "Keypad Plus": (None, 0xD),
    "Keypad ENTER": (None, 0x2),
    "Keypad 1 and End": (None, 0x3),
    "Keypad 2 and Down Arrow": (None, 0x0),
    "Keypad 3 and PageDn": (None, 0x1),
    "Keypad 4 and Left Arrow": (None, 0x6),
    "Keypad 5": (None, 0x7),
    "Keypad 6 and Right Arrow": (None, 0x4),
    "Keypad 7 and Home": (None, 0x5),
    "Keypad 8 and Up Arrow": (None, 0x3A),
    "Keypad 9 and PageUp": (None, 0x3B),
    "Keypad 0 and Insert": (None, 0x38),
    "Keypad Period": (None, 0x39),
    "Application": (None, 0x3F),
    "Power": (None, 0x3C),
    "Keypad Equals": (None, 0x3D),
    "F13": (None, 0x32),
    "F14": (None, 0x33),
    "F15": (None, 0x30),
    "F16": (None, 0x31),
    "F17": (None, 0x36),
    "F18": (None, 0x37),
    "F19": (None, 0x34),
    "F20": (None, 0x35),
    "F21": (None, 0x2A),
    "F22": (None, 0x2B),
    "F23": (None, 0x28),
    "F24": (None, 0x29),
    "Keypad Comma": (None, 0xDF),
    "International6": (None, 0xD6),
    "LANG1": (None, 0xCA),
    "LANG2": (None, 0xCB),
    "LANG3": (None, 0xC8),
    "LANG4": (None, 0xC9),
    "LANG5": (None, 0xCE),
    "Right GUI": (None, 0xBD),
    "Scan Next Track": (None, 0xEF),
    "Scan Previous Track": (None, 0xEC),
    "Stop": (None, 0xED),
    "Play/Pause": (None, 0x97),
    "Volume Increment": (None, 0xB3),
    "Volume Decrement": (None, 0xB0),
}


class FirmwareModifierGUI(QWidget):
    def __init__(self):
        super().__init__()
        self.initUI()

    def initUI(self):
        layout = QVBoxLayout()
        
        self.btn_original = QPushButton('1. Select Original Firmware')
        self.label_original = QLabel('Original Firmware: Not selected')
        self.btn_original.clicked.connect(self.select_original_firmware)
        
        self.sha256_label = QLabel('Original SHA256:')
        self.sha256_display = QLineEdit()
        self.sha256_display.setReadOnly(True)
        
        layout.addWidget(self.btn_original)
        layout.addWidget(self.label_original)
        layout.addWidget(self.sha256_label)
        layout.addWidget(self.sha256_display)
        layout.addWidget(self.create_separator())
        
        self.btn_json = QPushButton('2. Select Key Remap JSON')
        self.label_json = QLabel('Key Remap JSON: Not selected')
        self.btn_json.clicked.connect(self.select_json)
        
        self.remap_display = QTextEdit()
        self.remap_display.setReadOnly(True)
        
        layout.addWidget(self.btn_json)
        layout.addWidget(self.label_json)
        layout.addWidget(self.remap_display)
        layout.addWidget(self.create_separator())
                
        self.btn_execute = QPushButton('3. Modify and Save New Firmware')
        self.btn_execute.clicked.connect(self.modify_firmware)
        
        self.modified_sha256_label = QLabel('Modified SHA256:')
        self.modified_sha256_display = QLineEdit()
        self.modified_sha256_display.setReadOnly(True)
        
        layout.addWidget(self.btn_execute)
        layout.addWidget(self.modified_sha256_label)
        layout.addWidget(self.modified_sha256_display)
        
        self.setLayout(layout)
        self.setWindowTitle('KU-1255 Firmware Key Remapper')
        
        self.original_firmware = None
        self.modified_firmware = None
        self.key_remap_json = None
    
    def create_separator(self):
        separator = QFrame()
        separator.setFrameShape(QFrame.Shape.HLine)
        separator.setFrameShadow(QFrame.Shadow.Sunken)
        return separator
    
    def select_original_firmware(self):
        file, _ = QFileDialog.getOpenFileName(self, 'Select Original Firmware', '', 'Executable(*.exe)')
        if file:
            self.original_firmware = file
            self.label_original.setText(f'Original Firmware: {file}')
            sha256_hash = self.compute_sha256(file)
            self.sha256_display.setText(sha256_hash)
    
    def select_json(self):
        file, _ = QFileDialog.getOpenFileName(self, 'Select Key Remap JSON')
        if file:
            self.key_remap_json = file
            self.label_json.setText(f'Key Remap JSON: {file}')
            self.display_json_content()
    
    def compute_sha256(self, file_path):
        with open(file_path, 'rb') as f:
            return hashlib.sha256(f.read()).hexdigest()
    
    def display_json_content(self):
        if not self.key_remap_json:
            return
        
        try:
            with open(self.key_remap_json, 'r') as f:
                key_remap = json.load(f)
        except json.JSONDecodeError:
            QMessageBox.warning(self, 'Error', 'Invalid JSON format!')
            self.key_remap_json = None
            self.label_json.setText(f'Key Remap JSON: Not selected')
            return
        
        self.remap_display.setText("\n".join(f'{source} --> {target}' for source, target in key_remap.items()))
    
    def modify_firmware(self):
        if not all([self.original_firmware, self.key_remap_json]):
            QMessageBox.warning(self, 'Error', 'All files must be selected!')
            return
        
        save_path, _ = QFileDialog.getSaveFileName(self, 'Select Save Location', '', 'Executable(*.exe)')
        if not save_path:
            return
        self.modified_firmware = save_path
        
        try:
            with open(self.key_remap_json, 'r') as f:
                key_remap = json.load(f)
        except json.JSONDecodeError:
            QMessageBox.warning(self, 'Error', 'Invalid JSON format!')
            self.key_remap_json = None
            self.label_json.setText(f'Key Remap JSON: Not selected')
            return
        
        with open(self.original_firmware, 'rb') as f:
            data = bytearray(f.read())
        
        for source_key, target_key in key_remap.items():
            if source_key not in KEYMAPS or target_key not in KEYMAPS:
                QMessageBox.warning(self, 'Error', f'Key {source_key} or {target_key} not found in CSV!')
                return
            byte_offset, expected_scan_code = KEYMAPS[source_key]
            _, new_scan_code = KEYMAPS[target_key]
            
            if data[byte_offset] != expected_scan_code:
                QMessageBox.critical(self, 'Error', f'Unexpected value at {hex(byte_offset)}. Expected {hex(expected_scan_code)}, found {hex(data[byte_offset])}.')
                return
            
            data[byte_offset] = new_scan_code
        
        with open(self.modified_firmware, 'wb') as f:
            f.write(data)
        
        modified_sha256 = self.compute_sha256(self.modified_firmware)
        self.modified_sha256_display.setText(modified_sha256)
        QMessageBox.information(self, 'Success', 'Firmware modified successfully!')

if __name__ == '__main__':
    app = QApplication(sys.argv)
    ex = FirmwareModifierGUI()
    ex.show()
    sys.exit(app.exec())

