import sys
import csv
import hashlib
from PySide6.QtWidgets import (
    QApplication, QWidget, QVBoxLayout, QPushButton, QFileDialog, QLabel,
    QLineEdit, QTextEdit, QMessageBox, QFrame, QCheckBox
)
from PySide6.QtGui import QFont

class FirmwareModifierGUI(QWidget):
    def __init__(self):
        super().__init__()
        self.initUI()
        self.keymaps = self.load_keymaps_csv("keymaps.csv")
        self.key_remaps = []
        self.layer2_keymaps = []

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
        
        self.chk_layer2 = QCheckBox('Enable the 2nd Layer')
        self.chk_layer2.stateChanged.connect(self.toggle_layer2_options)
        layout.addWidget(self.chk_layer2)
        
        self.layer2_options = QWidget()
        layer2_layout = QVBoxLayout()
        self.label_layer2 = QLabel('The layer key is "Right GUI" (originally not mapped)')
        self.btn_layer2_csv = QPushButton('2B. Select Layer 2 Keymap CSV')
        self.label_layer2_keymap_csv = QLabel('Layer 2 CSV: Not selected')
        self.btn_layer2_csv.clicked.connect(self.select_layer2_keymap_csv)

        self.layer2_keymap_display = QTextEdit()
        self.layer2_keymap_display.setReadOnly(True)
        self.layer2_keymap_display.setFont(font)

        layer2_layout.addWidget(self.label_layer2)
        layer2_layout.addWidget(self.btn_layer2_csv)
        layer2_layout.addWidget(self.label_layer2_keymap_csv)
        layer2_layout.addWidget(self.layer2_keymap_display)
        
        self.layer2_options.setLayout(layer2_layout)
        layout.addWidget(self.layer2_options)
        self.layer2_options.setVisible(False)
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
        self.layer2_keymap_csv = None
    
    def toggle_layer2_options(self, state):
        self.layer2_options.setVisible(state == 2)
    
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

    def select_layer2_keymap_csv(self):
        self.layer2_keymaps = []
        file, _ = QFileDialog.getOpenFileName(self, 'Select Layer 2 Keymap CSV')
        if file:
            self.layer2_keymap_csv = file
            self.label_layer2_keymap_csv.setText(f'Layer 2 Keymap CSV: {file}')     
            try:
                with open(self.layer2_keymap_csv, 'r', encoding='utf-8') as f:
                    reader = csv.DictReader(f)
                    for row in reader:
                        self.layer2_keymaps.append((int(row["Before_ID"], 16), int(row["After_ID"], 16)))
            except:
                QMessageBox.warning(self, 'Error', 'Invalid CSV format!')
                self.layer2_keymap_csv = None
                self.label_layer2_keymap_csv.setText('Layer 2 Keymap CSV: Not selected')
                return
            self.display_layer2_csv_content()

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
        
    def display_layer2_csv_content(self):
        if not self.layer2_keymap_csv:
            return
        try:
            max_len = max(len(self.keymaps[bf_id][0]) for bf_id, _ in self.key_remaps)
            formatted_text = "\n".join(f'{self.keymaps[bf_id][0].ljust(max_len)} --> {self.keymaps[af_id][0]}' for bf_id, af_id in self.layer2_keymaps)
            self.layer2_keymap_display.setText(formatted_text)
        except:
            QMessageBox.warning(self, 'Error', 'Cannot display layer 2 mapping')
            self.layer2_keymap_csv = None
            self.label_layer2_keymap_csv.setText(f'Key Remap CSV: Not selected')
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
        
        for i in range(len(data)):
            if i >= 0x73FD2 and i <= 0x740CF:
                if i % 2 == 0:
                    if data[i] != 0x5A:
                        data[i + 1] = data[i]

        for row in self.key_remaps:
            source_key, target_key = row[0], row[1]
            if source_key not in self.keymaps or target_key not in self.keymaps:
                QMessageBox.warning(self, 'Error', f'Key IDs {source_key} or {target_key} not found in keymap CSV!')
                return
            byte_offset, expected_xored_id = self.keymaps[source_key][1], self.keymaps[source_key][2]
            if not byte_offset:
                QMessageBox.critical(self, 'Error', f'No offset position of Key ID: "{hex(source_key)}" is recorded in the keymaps CSV.')
                return
            if not expected_xored_id:
                QMessageBox.critical(self, 'Error', f'No XORed ID value of Key ID: "{hex(source_key)}" is recorded in the keymaps CSV.')
                return                
            
            _, new_xored_id = self.keymaps[target_key][1], self.keymaps[target_key][2]
            if not new_xored_id:
                QMessageBox.critical(self, 'Error', f'No XORed ID value of Key ID: "{hex(target_key)}" is recorded in the keymaps CSV.')
                return
            
            if data[byte_offset] != expected_xored_id:
                QMessageBox.critical(self, 'Error', f'Unexpected value at {hex(byte_offset)}. Expected {hex(expected_xored_id)}, found {hex(data[byte_offset])}.')
                return
            
            data[byte_offset] = new_xored_id
            data[byte_offset + 1] = new_xored_id

        # Add Layer logic
        address_start = 0x73A49
        address_end = 0x73A9A
        address_3rd = 0x73D10
        address_4th = 0x73D14
        expected_values = bytes.fromhex(
            "01 B8 D8 15 9F DC 28 13 DE BF D8 A7 9E DC 28 13 DE 55 44 F5 5C AE "
            "D8 2E 74 DC 2A 5A 54 5A 1A 5A 01 5A 12 49 13 49 12 49 16 49 14 49 "
            "15 13 DE 5A 02 44 D9 54 5C A1 D8 1D 77 55 45 1E DE 49 5C 5A D9 12 "
            "77 55 45 1E DE 5F 5C 5C D9 48 17 49 12 65 71 13"
        )
        new_values = bytes.fromhex(
            "0D BE D8 5A 01 B2 D8 15 9F DC 28 13 DE B1 D8 D8 44 55 45 84 D8 5A "
            "5A A7 9E DC 28 13 DE 55 44 F5 5C A0 D8 2E 74 DC 2A 5A 54 5A 1A 5A "
            "01 5A 12 49 13 49 12 49 16 49 14 49 15 13 DE 5A 02 44 D9 0A 5C 5B "
            "D9 10 77 55 45 1E DE 15 5C 5C D9 17 77 55 45 1E"
        )
        if data[address_start:address_end + 1] == expected_values:
            data[address_start:address_end + 1] = new_values
        else:
            QMessageBox.warning(self, 'Error', f'Byte arrays of between {address_start} and {address_end} are illegal!')
        
        if data[address_3rd] == 0x4F:
            data[address_3rd] = 0x5A
        else:
            QMessageBox.warning(self, 'Error', f'Value at {address_3rd} is illegal!')

        if data[address_4th] == 0x4F:
            data[address_4th] = 0x5A
        else:
            QMessageBox.warning(self, 'Error', f'Value at {address_4th} is illegal!')

        for row in self.layer2_keymaps:
            source_key, target_key = row[0], row[1]
            if source_key not in self.keymaps or target_key not in self.keymaps:
                QMessageBox.warning(self, 'Error', f'Key IDs {source_key} or {target_key} not found in keymap CSV!')
                return
            byte_offset, expected_xored_id = self.keymaps[source_key][1], self.keymaps[source_key][2]
            if not byte_offset:
                QMessageBox.critical(self, 'Error', f'No offset position of Key ID: "{hex(source_key)}" is recorded in the keymaps CSV.')
                return
            if not expected_xored_id:
                QMessageBox.critical(self, 'Error', f'No XORed ID value of Key ID: "{hex(source_key)}" is recorded in the keymaps CSV.')
                return                
            
            _, new_xored_id = self.keymaps[target_key][1], self.keymaps[target_key][2]
            if not new_xored_id:
                QMessageBox.critical(self, 'Error', f'No XORed ID value of Key ID: "{hex(target_key)}" is recorded in the keymaps CSV.')
                return
            
            if data[byte_offset] == 0xBD:
                QMessageBox.critical(self, 'Error', f'Right GUI key cannot be converted in the layer 2.')
                return  
            
            data[byte_offset + 1] = new_xored_id       

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
