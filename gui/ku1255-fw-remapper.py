import sys
import csv
import hashlib
from PySide6.QtWidgets import (
    QApplication, QWidget, QVBoxLayout, QPushButton, QFileDialog, QLabel,
    QLineEdit, QTextEdit, QMessageBox, QFrame
)
from PySide6.QtGui import QFont

class FirmwareModifierGUI(QWidget):
    def __init__(self):
        super().__init__()
        self.initUI()
        self.keymaps = self.load_keymaps_csv("keymaps.csv")
        self.key_remaps = []

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
        
        self.btn_remap_csv = QPushButton('2. Select Key Remap CSV')
        self.label_remap_csv = QLabel('Key Remap CSV: Not selected')
        self.btn_remap_csv.clicked.connect(self.select_remap_csv)

        font = QFont("Courier New")
        font.setStyleHint(QFont.StyleHint.Monospace)
        self.remap_display = QTextEdit()
        self.remap_display.setReadOnly(True)
        self.remap_display.setFont(font)

        layout.addWidget(self.btn_remap_csv)
        layout.addWidget(self.label_remap_csv)
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
        self.key_remap_csv = None
    
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
    
    def select_remap_csv(self):
        self.key_remaps = []
        file, _ = QFileDialog.getOpenFileName(self, 'Select Key Remap CSV')
        if file:
            self.key_remap_csv = file
            self.label_remap_csv.setText(f'Key Remap CSV: {file}')
            try:
                with open(self.key_remap_csv, 'r', encoding='utf-8') as f:
                    reader = csv.DictReader(f)
                    for row in reader:
                        self.key_remaps.append((int(row["Before_ID"], 16), int(row["After_ID"], 16)))
            except:
                QMessageBox.warning(self, 'Error', 'Invalid CSV format!')
                self.key_remap_csv = None
                self.label_remap_csv.setText(f'Key Remap CSV: Not selected')
                return    
            self.display_remap_csv_content()

    def load_keymaps_csv(self, csv_file):
        """Loads key mapping data from a CSV file."""
        keymaps = {}
        with open(csv_file, 'r', encoding='utf-8') as f:
            reader = csv.DictReader(f)
            for row in reader:
                if row['Byte_Offset']:
                    keymaps[int(row['ID'], 16)] = (row['Usage Name'], int(row['Byte_Offset'], 16), int(row['XORed_ID'], 16))
                else:
                    keymaps[int(row['ID'], 16)] = (row['Usage Name'], None, int(row['XORed_ID'], 16))
        return keymaps
    
    def compute_sha256(self, file_path):
        with open(file_path, 'rb') as f:
            return hashlib.sha256(f.read()).hexdigest()
    
    def display_remap_csv_content(self):
        if not self.key_remap_csv:
            return
        try:
            max_len = max(len(self.keymaps[bf_id][0]) for bf_id, _ in self.key_remaps)
            formatted_text = "\n".join(f'{self.keymaps[bf_id][0].ljust(max_len)} --> {self.keymaps[af_id][0]}' for bf_id, af_id in self.key_remaps)
            self.remap_display.setText(formatted_text)
        except:
            QMessageBox.warning(self, 'Error', 'Cannot display remapping')
            self.key_remap_csv = None
            self.label_remap_csv.setText(f'Key Remap CSV: Not selected')
            return
    
    def modify_firmware(self):
        if not all([self.original_firmware, self.key_remap_csv]):
            QMessageBox.warning(self, 'Error', 'All files must be selected!')
            return
        
        save_path, _ = QFileDialog.getSaveFileName(self, 'Select Save Location', '', 'Executable(*.exe)')
        if not save_path:
            return
        self.modified_firmware = save_path
                
        with open(self.original_firmware, 'rb') as f:
            data = bytearray(f.read())
        
        for row in self.key_remaps:
            source_key, target_key = row[0], row[1]
            if source_key not in self.keymaps or target_key not in self.keymaps:
                QMessageBox.warning(self, 'Error', f'Key IDs {source_key} or {target_key} not found in keymap CSV!')
                return
            byte_offset, expected_xored_id = self.keymaps[source_key][1], self.keymaps[source_key][2]
            _, new_xored_id = self.keymaps[target_key][1], self.keymaps[target_key][2]
            
            if data[byte_offset] != expected_xored_id:
                QMessageBox.critical(self, 'Error', f'Unexpected value at {hex(byte_offset)}. Expected {hex(expected_xored_id)}, found {hex(data[byte_offset])}.')
                return
            
            data[byte_offset] = new_xored_id
        
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
