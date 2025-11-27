#!/usr/bin/env python
# Copyright (C) 2019  Vincent Pelletier <plr.vincent@gmail.com>
# Modified by haborite
#
# This program is free software; you can redistribute it and/or
# modify it under the terms of the GNU General Public License
# as published by the Free Software Foundation; either version 2
# of the License, or (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program; if not, write to the Free Software
# Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301, USA.

import argparse
import contextlib
import struct
import sys
import time
import usb1

import struct
from PySide6.QtCore import QThread, Signal, Qt
from PySide6.QtWidgets import (
    QApplication, QWidget, QVBoxLayout, QPushButton, QLabel, QTextEdit, QMessageBox
)

ERASE_BLOCK_LENGTH_WORDS = 0x80
IMAGE_LENGTH = 0x3000 * 2
EXPECTED_IMAGE_LENGTH_DICT = {
    IMAGE_LENGTH: lambda x: x, # Plain image
    IMAGE_LENGTH + 0x100: lambda x: x[0x100:], # SN8 image format
}
CANARY_ADDRESS_WORDS = 0x27ff
CANARY_ADDRESS_BYTES = CANARY_ADDRESS_WORDS * 2
CANARY = b'\xaa\xaa'
CANARY_PAGE_ADDRESS_WORDS = CANARY_ADDRESS_WORDS & ~(
    ERASE_BLOCK_LENGTH_WORDS - 1
)
assert CANARY_PAGE_ADDRESS_WORDS == 0x2780
# No writes (neither erase nor program) above this address
FLASHER_BASE_ADDRESS_WORDS = 0x2800
FLASHER_BASE_ADDRESS_BYTES = FLASHER_BASE_ADDRESS_WORDS * 2
UNPROGRAMABLE_PREFIX_WORDS = 8
UNPROGRAMABLE_PREFIX_BYTES = UNPROGRAMABLE_PREFIX_WORDS * 2
# JMP 0x2800 reset vector, jumping to canary checker.
# All other words are cleared.
FIRST_8_WORDS_CHECKSUM = 0x80 + 0xa8
ALL_ERASED_EXPECTED_CHECKSUM = (
    (
        (FLASHER_BASE_ADDRESS_BYTES - UNPROGRAMABLE_PREFIX_BYTES)
        * 0xff       # erased byte value
    )
    + FIRST_8_WORDS_CHECKSUM
) & 0xffff           # 16 bits modular arithmetic
assert ALL_ERASED_EXPECTED_CHECKSUM == 0xa138
ERASABLE_PAGE_COUNT = FLASHER_BASE_ADDRESS_WORDS // ERASE_BLOCK_LENGTH_WORDS
assert ERASABLE_PAGE_COUNT == 0x50


class UnexpectedResponse(Exception):
    pass


class FlashWorker(QThread):
    progress = Signal(int, int)
    message = Signal(str)
    error = Signal(str)
    finished = Signal()

    def __init__(self, infile_path: str, device_list: list[str], single: str | None):
        super().__init__()
        self.infile_path = infile_path
        self.device_list = device_list
        self.single      = single

    @contextlib.contextmanager
    def timer(self, caption):
        print(caption, end=' ')
        self.message.emit(caption)
        begin = time.time()
        try:
            yield
        except:
            print('Failed')
            raise
        finally:
            print('Done in %.2fs' % (time.time() - begin))
            self.message.emit('Done in %.2fs' % (time.time() - begin))

    def getCandidateDeviceList(self, usb, bus_address, vid_pid_list):
        match_list = []
        if bus_address:
            if ':' in bus_address:
                bus, address = bus_address.split(':')
            else:
                bus = ''
                address = bus_address
            address = int(address, 16)
            if bus:
                match_list.append(
                    lambda x, _expected=(int(bus, 16), address): (
                        x.getBusNumber(),
                        x.getDeviceAddress(),
                    ) == _expected
                )
            else:
                match_list.append(lambda x: x.getDeviceAddress() == address)
        raw_vid_pid_list = []
        for vid_pid in vid_pid_list:
            vid, pid = vid_pid.split(':')
            raw_vid_pid_list.append((
                int(vid, 16),
                int(pid, 16),
            ))
        match_list.append(
            lambda x, _expected=tuple(raw_vid_pid_list): (
                x.getVendorID(), x.getProductID()
            ) in _expected
        )
        candidate_list = []
        for device in usb.getDeviceIterator(skip_on_error=True):
            if all(match(device) for match in match_list):
                candidate_list.append(device)
        return candidate_list

    def hexdump(self, value):
        return ' '.join('%02x' % x for x in value)

    def send(self, device, data):
        assert len(data) == 8
        device.controlWrite(
            request_type=usb1.REQUEST_TYPE_CLASS | usb1.RECIPIENT_INTERFACE,
            request=0x09, # SET_REPORT
            value=0x0300,
            index=0,
            timeout=500,
            data=data,
        )

    def no_send(self, device, data):
        _ = device # silence pylint
        print('NOT sending ' + self.hexdump(data))

    def recv(self, device, expected):
        result = device.controlRead(
            request_type=usb1.REQUEST_TYPE_CLASS | usb1.RECIPIENT_INTERFACE,
            request=0x01, # GET_REPORT
            value=0x0300,
            index=0,
            timeout=500,
            length=8,
        )
        if not result.startswith(expected):
            raise UnexpectedResponse(self.hexdump(result))
        return result

    def no_recv(self, device, expected):
        _ = device # silence pylint
        print('NOT receiving ' + self.hexdump(expected))


    def switchToFlasher(self, device):
        with self.timer('Switching to flasher...'):
            self.send(device, b'\xaa\x55\xa5\x5a\xff\x00\x33\xcc')

    def unlockFlash(self, device):
        with self.timer('Unlocking flash...'):
            self.send(device, b'\x01\xaa\x55\x00\x00\x00\x00\x00')
            self.recv(device, b'\x01\xaa\x55\x00\x00\x03\x00\x00')
            self.send(device, b'\x02\xaa\x55\x00\x12\x34\x56\x78')
            self.recv(device, b'\x02\xaa\x55\x00\xfa\xfa\xfa\xfa')

    def getFlashUnlockState(self, device):
        with self.timer('Getting flash lock state...'):
            self.send(device, b'\x03\xaa\x55\x00\x00\x00\x00\x00')
            return self.recv(device, b'\x03\xaa\x55\x00')[4:] == b'\xfa' * 4

    def _erase(self, device, base_address_words, page_count):
        if (
            base_address_words < 0 or
            page_count < 1 or
            base_address_words & 127
        ):
            raise ValueError(repr(base_address_words, page_count))
        last_erased_address_words = (
            base_address_words +
            page_count * ERASE_BLOCK_LENGTH_WORDS
            - 1 # Otherwise it would be first non-erased address
        )
        # Flasher does not protect itself, do it instead.
        if last_erased_address_words >= FLASHER_BASE_ADDRESS_WORDS:
            raise ValueError('Refusing to erase flasher program')
        self.send(
            device,
            b'\x04\xaa\x55\x00' + struct.pack(
                '<HH',
                base_address_words,
                page_count,
            ),
        )

    def erase(self, device, base_address_words, page_count):
        # Flasher is not erasing canary page correctly (requesting an erase on
        # CANARY_ADDRESS_WORDS instead of CANARY_PAGE_ADDRESS_WORDS), it is unclear
        # whether that works at all.
        with self.timer('Erasing %#04x to %#04x...' % (
            base_address_words,
            (
                base_address_words
                + page_count * ERASE_BLOCK_LENGTH_WORDS
                - 1 # Otherwise it would be first non-erased address
            ),
        )):
            self._erase(device, base_address_words, page_count)

    def getChecksum(self, device):
        with self.timer('Getting 0x0000 to 0x27ff checksum...'):
            self.send(device, b'\x06\xaa\x55\x00\x00\x00\x00\x00')
            result, = struct.unpack(
                '<H',
                self.recv(device, b'\x06\xaa\x55\x00\xfa\xfa')[6:],
            )
        return result

    def getCodeOptions(self, device):
        with self.timer('Retrieving code options...'):
            self.send(device, b'\x09\xaa\x55\x00\x00\x00\x00\x00')
            options_2ffc_2ffd = self.recv(device, b'\x09\xaa\x55\x00')[4:]
            self.send(device, b'\x09\xaa\x55\x01\x00\x00\x00\x00')
            options_2ffe_2fff = self.recv(device, b'\x09\xaa\x55\x01')[4:]
        return options_2ffc_2ffd + options_2ffe_2fff

    def reboot(self, device):
        print('Asking device to reboot...')
        self.message.emit('Asking device to reboot...')
        self.send(device, b'\x07\xaa\x55\x00\x00\x00\x00\x00')
        # There is (should be) no answer
    
    def program(self, device, base_address_words, data):
        write_packet_count, remainder = divmod(len(data), 8)
        if remainder:
            # Flasher does not care how many bytes we actually send, it always
            # flashes 4 words / 8 bytes.
            # Which is a spec violation, as chip datasheet explicitely
            # says the base programming address must be 32-words-aligned. But
            # unlike sloppy canary erase, this is at least verified to work,
            # otherwise vendor flashing program would fail.
            self.error.emit('Data length must be a multiple of 8.')
            raise ValueError('Data length must be a multiple of 8.')
        last_programmed_address_words = (
            base_address_words
            + len(data) // 2
            - 1 # Otherwise it would be first non-erased address
        )
        with self.timer('Programming from %#04x to %#04x...' % (
            base_address_words,
            last_programmed_address_words,
        )):
            self.send(
                device,
                b'\x05\xaa\x55\x00' + struct.pack(
                    '<HH',
                    base_address_words,
                    write_packet_count,
                ),
            )
            self.recv(
                device,
                b'\x05\xaa\x55\x00\xfa\xfa\xfa\xfa',
            )
            sending_offset = 0
            last_offset = len(data) - 8
            while data:
                print('\rSending %#04x / %#04x... ' % (sending_offset, last_offset), end='')
                self.progress.emit(sending_offset, last_offset)
                self.message.emit(f"Sending {sending_offset:#04x} / {last_offset:#04x}...")
                sending_offset += 8
                while True:
                    try:
                        self.send(device, data[:8])
                        # Flash packet is acked immediately, but firmware seems to
                        # clear USB interrupt at a different time, causing next
                        # transmission to be permanently lost. So sleep for 1ms,
                        # which is cheaper than triggering timeouts - and does the
                        # trick on my board at least. An entire flash takes under
                        # 4 seconds with this, so it should be acceptable.
                        time.sleep(.001)
                    except usb1.USBErrorTimeout:
                        print('Timed out, retrying', end='')
                        self.message.emit(f"Timed out, retrying")
                        continue
                    else:
                        print('                   ', end='')
                        break
                data = data[8:]

    def run(self):
        with open(self.infile_path, 'rb') as infile:
            image = infile.read(max(EXPECTED_IMAGE_LENGTH_DICT))

        try:
            image = EXPECTED_IMAGE_LENGTH_DICT[len(image)](image)
        except KeyError:
            self.error.emit(
                f'Invalid image length: {len(image)} bytes. Expected one of: '
                f'{", ".join(hex(x) for x in EXPECTED_IMAGE_LENGTH_DICT)}'         
            )
            raise ValueError(
                f'Invalid image length: {len(image)} bytes. Expected one of: '
                f'{", ".join(hex(x) for x in EXPECTED_IMAGE_LENGTH_DICT)}'
            )

        image_code_options = image[0x2ffc * 2:]
        assert len(image_code_options) == 8
        image = image[UNPROGRAMABLE_PREFIX_BYTES:FLASHER_BASE_ADDRESS_BYTES]

        if image[CANARY_ADDRESS_BYTES - 16:] != CANARY:
            self.error.emit(
                f'Canary missing. Add ".ORG {CANARY_ADDRESS_WORDS:#04x} '
                f'DW {struct.unpack("<H", CANARY)[0]:#04x}" to source and rebuild.'                
            )
            raise ValueError(
                f'Canary missing. Add ".ORG {CANARY_ADDRESS_WORDS:#04x} '
                f'DW {struct.unpack("<H", CANARY)[0]:#04x}" to source and rebuild.'
            )

        assert len(image) / 8 == 0x9fe
        all_programmed_expected_checksum = (
            FIRST_8_WORDS_CHECKSUM + sum(image)
        ) & 0xffff

        with usb1.USBContext() as usb:
            found_devices = self.getCandidateDeviceList(
                usb=usb,
                bus_address=[self.single] if self.single else None,
                vid_pid_list=self.device_list,
            )
            if len(found_devices) != 1:
                self.error.emit(f'{len(found_devices)} device(s) found.')
                raise RuntimeError(f'{len(found_devices)} device(s) found.')

            device = found_devices[0]
            print(f'Using device {device.getVendorID():04x}:{device.getProductID():04x} '
                f'at {device.getBusNumber():02}:{device.getDeviceAddress():03}')
            self.message.emit(f'Using device {device.getVendorID():04x}:{device.getProductID():04x} ')
            self.message.emit(f'at {device.getBusNumber():02}:{device.getDeviceAddress():03}')

            try:
                handle = device.open()
            except usb1.USBErrorAccess:
                self.error.emit(
                    f'Permission denied opening device {device.getBusNumber():02}:{device.getDeviceAddress():03}.'
                )
                raise PermissionError(
                    f'Permission denied opening device {device.getBusNumber():02}:{device.getDeviceAddress():03}.'
                )
            except usb1.USBErrorIO:
                self.error.emit('I/O error opening USB device.')
                raise IOError('I/O error opening USB device.')

            if handle.getConfiguration():
                for iface in range(len(device[handle.getConfiguration() - 1])):
                    try:
                        handle.detachKernelDriver(iface)
                    except (usb1.USBErrorNotFound, usb1.USBErrorNotSupported):
                        pass
                    handle.claimInterface(iface)
            else:
                handle.setConfiguration(1)
                handle.claimInterface(0)

            try:
                unlocked = self.getFlashUnlockState(handle)
            except UnexpectedResponse:
                print('Not in flasher mode. Switching...')
                self.message.emit('Not in flasher mode. Switching...')
                self.switchToFlasher(handle)
                unlocked = self.getFlashUnlockState(handle)

            if not unlocked:
                print('Unlocking flash...')
                self.message.emit('Unlocking flash...')
                self.unlockFlash(handle)
                if not self.getFlashUnlockState(handle):
                    self.error.emit('Failed to unlock flash.')
                    raise RuntimeError('Failed to unlock flash.')

            if self.getCodeOptions(handle) != image_code_options:
                self.error.emit('Code option mismatch between flash and image.')
                raise ValueError('Code option mismatch between flash and image.')

            self.erase(handle, 0, ERASABLE_PAGE_COUNT)
            time.sleep(2.5)

            while True:
                try:
                    erased_checksum = self.getChecksum(handle)
                    break
                except usb1.USBErrorTimeout:
                    continue

            if erased_checksum != ALL_ERASED_EXPECTED_CHECKSUM:
                self.error.emit(
                    f'Post-erase checksum mismatch: expected '
                    f'{ALL_ERASED_EXPECTED_CHECKSUM:#04x}, got {erased_checksum:#04x}'
                )
                raise RuntimeError(
                    f'Post-erase checksum mismatch: expected '
                    f'{ALL_ERASED_EXPECTED_CHECKSUM:#04x}, got {erased_checksum:#04x}'
                )

            self.program(handle, 0x0008, image)
            checksum = self.getChecksum(handle)

            if checksum != all_programmed_expected_checksum:
                print('Checksum mismatch after programming. Attempting re-erase.')
                self.message.emit("Checksum mismatch after programming. Attempting re-erase.")
                with self.timer('Re-erasing...'):
                    self.erase(handle, 0, ERASABLE_PAGE_COUNT)
                self.error.emit(
                    f'Post-program checksum mismatch: expected '
                    f'{all_programmed_expected_checksum:#04x}, got {checksum:#04x}'
                )
                raise RuntimeError(
                    f'Post-program checksum mismatch: expected '
                    f'{all_programmed_expected_checksum:#04x}, got {checksum:#04x}'
                )

            print('Success!')
            self.message.emit("Success!")
            self.reboot(handle)
            time.sleep(0.5)
            self.finished.emit()


class ProgressApp(QWidget):

    def __init__(
        self,
        infile_path,
        vid_pid_list=['0c45:7500', '17ef:6047'],
        bus_address=None,
    ):
        super().__init__()
        self.setWindowTitle("SN8 Flasher GUI")
        self.progress_label = QLabel("0/0")
        self.message_box = QTextEdit()
        self.message_box.setReadOnly(True)
        self.start_button = QPushButton("Start")
        self.finish_button = QPushButton("Finish")
        self.finish_button.setEnabled(False)

        layout = QVBoxLayout()
        layout.addWidget(self.progress_label)
        layout.addWidget(self.message_box)
        layout.addWidget(self.start_button)
        layout.addWidget(self.finish_button)
        self.setLayout(layout)

        self.start_button.clicked.connect(self.start_flashing)
        self.finish_button.clicked.connect(QApplication.instance().quit)

        self.vid_pid_list = vid_pid_list
        self.bus_address = bus_address
        self.infile_path = infile_path

    def start_flashing(self):

        flags = self.windowFlags()
        flags &= ~Qt.WindowCloseButtonHint
        self.setWindowFlags(flags)
        self.show()

        self.worker = FlashWorker(self.infile_path, self.vid_pid_list, self.bus_address)
        self.worker.progress.connect(self.update_progress)
        self.worker.message.connect(self.append_message)
        self.worker.error.connect(self.show_error)
        self.worker.finished.connect(self.enable_finish)

        self.start_button.setEnabled(False)
        self.finish_button.setEnabled(False)
        self.message_box.clear()
        self.progress_label.setText("0/100")
        self.worker.start()

    def update_progress(self, current, total):
        self.progress_label.setText(f"{current}/{total}")

    def append_message(self, msg):
        self.message_box.append(msg)

    def show_error(self, msg):
        self.message_box.append(f"Error: {msg}")
        # self.finish_button.setEnabled(True)

    def enable_finish(self):
        self.finish_button.setEnabled(True)

class MyArgumentParser(argparse.ArgumentParser):
    def error(self, message):
        raise ValueError(f"Argument parse error:\n{message}")

def show_error_dialog(message: str):
    _app = QApplication(sys.argv)
    QMessageBox.critical(None, "Error", message)
    sys.exit(1)

def main():
    parser = MyArgumentParser(
        description='Implement SN8 flashing protocol. USE AT YOUR OWN RISK.',
    )
    parser.add_argument(
        '-d', '--device',
        action='store',
        nargs='+',
        default=[
            '0c45:7500',
            '17ef:6047',
        ],
        help='vendor:product list.',
    )
    parser.add_argument(
        '-s', '--single',
        nargs=1,
        help='[[bus]:][devnum] of device.',
    )
    parser.add_argument(
        'infile',
        help='Firmware file to flash.',
    )
    try:
        args = parser.parse_args()
    except ValueError as e:
        print(f"{e}", file=sys.stderr)
        show_error_dialog(f"{e}")

    try:
        app = QApplication(sys.argv)
        window = ProgressApp(
            args.infile,
            vid_pid_list=args.device,
            bus_address=args.single[0] if args.single else None,
        )
        window.resize(400, 300)
        window.show()
        sys.exit(app.exec())
    except Exception as e:
        show_error_dialog(f"An error occurred: {e}")
        parser.error(str(e))

if __name__ == "__main__":
    main()
