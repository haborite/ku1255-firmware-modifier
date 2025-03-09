import sys
import json
import csv
import hashlib
from PySide6.QtWidgets import (
    QApplication, QWidget, QVBoxLayout, QPushButton, QFileDialog, QLabel,
    QLineEdit, QTextEdit, QMessageBox, QFrame
)

class FirmwareModifierGUI(QWidget):
    def __init__(self):
        super().__init__()
        self.initUI()

    def initUI(self):
        layout = QVBoxLayout()
        
        self.btn_original = QPushButton('1. Select Original Firmware')
        self.label_original = QLabel('Original Firmware: Not selected')
        self.btn_original.clicked.connect(self.select_original_firmware)
        
        self.sha256_label = QLabel('Original FW SHA256:')
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
        
        self.modified_sha256_label = QLabel('Modified FW SHA256:')
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

    def load_keymap_csv(self, csv_file):
        """Loads key mapping data from a CSV file."""
        keymap = {}
        with open(csv_file, 'r', encoding='utf-8') as f:
            reader = csv.DictReader(f)
            for row in reader:
                if row['Byte_Offset']:
                    keymap[row['HID_Key_Name']] = (int(row['Byte_Offset'], 16), int(row['Scan_Code'], 16))
                else:
                    keymap[row['HID_Key_Name']] = (None, int(row['Scan_Code'], 16))
        return keymap
    
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

        KEYMAPS = self.load_keymap_csv("keymaps.csv")
        
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
