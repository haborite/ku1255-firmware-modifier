#!/usr/bin/env python
# Copyright (C) 2019  Vincent Pelletier <plr.vincent@gmail.com>
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
"""
FOR INTERACTIVE USE ONLY !
This is not a python module, do not import it - run it.
"""

import argparse
import contextlib
import struct
import sys
import time
import usb1

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

@contextlib.contextmanager
def timer(caption):
    print(caption, end=' ')
    begin = time.time()
    try:
        yield
    except:
        print('Failed')
        raise
    finally:
        print('Done in %.2fs' % (time.time() - begin))


def getCandidateDeviceList(usb, bus_address, vid_pid_list):
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

def hexdump(value):
    return ' '.join('%02x' % x for x in value)

def send(device, data):
    assert len(data) == 8
    device.controlWrite(
        request_type=usb1.REQUEST_TYPE_CLASS | usb1.RECIPIENT_INTERFACE,
        request=0x09, # SET_REPORT
        value=0x0300,
        index=0,
        timeout=500,
        data=data,
    )

def no_send(device, data):
    _ = device # silence pylint
    print('NOT sending ' + hexdump(data))

def recv(device, expected):
    result = device.controlRead(
        request_type=usb1.REQUEST_TYPE_CLASS | usb1.RECIPIENT_INTERFACE,
        request=0x01, # GET_REPORT
        value=0x0300,
        index=0,
        timeout=500,
        length=8,
    )
    if not result.startswith(expected):
        raise UnexpectedResponse(hexdump(result))
    return result

def no_recv(device, expected):
    _ = device # silence pylint
    print('NOT receiving ' + hexdump(expected))

def switchToFlasher(device):
    with timer('Switching to flasher...'):
        send(device, b'\xaa\x55\xa5\x5a\xff\x00\x33\xcc')

def unlockFlash(device):
    with timer('Unlocking flash...'):
        send(device, b'\x01\xaa\x55\x00\x00\x00\x00\x00')
        recv(device, b'\x01\xaa\x55\x00\x00\x03\x00\x00')
        send(device, b'\x02\xaa\x55\x00\x12\x34\x56\x78')
        recv(device, b'\x02\xaa\x55\x00\xfa\xfa\xfa\xfa')

def getFlashUnlockState(device):
    with timer('Getting flash lock state...'):
        send(device, b'\x03\xaa\x55\x00\x00\x00\x00\x00')
        return recv(device, b'\x03\xaa\x55\x00')[4:] == b'\xfa' * 4

def _erase(device, base_address_words, page_count):
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
    send(
        device,
        b'\x04\xaa\x55\x00' + struct.pack(
            '<HH',
            base_address_words,
            page_count,
        ),
    )

def erase(device, base_address_words, page_count):
    # Flasher is not erasing canary page correctly (requesting an erase on
    # CANARY_ADDRESS_WORDS instead of CANARY_PAGE_ADDRESS_WORDS), it is unclear
    # whether that works at all.
    with timer('Erasing %#04x to %#04x...' % (
        base_address_words,
        (
            base_address_words
            + page_count * ERASE_BLOCK_LENGTH_WORDS
            - 1 # Otherwise it would be first non-erased address
        ),
    )):
        _erase(device, base_address_words, page_count)

def program(device, base_address_words, data):
    write_packet_count, remainder = divmod(len(data), 8)
    if remainder:
        # Flasher does not care how many bytes we actually send, it always
        # flashes 4 words / 8 bytes.
        # Which is a spec violation, as chip datasheet explicitely
        # says the base programming address must be 32-words-aligned. But
        # unlike sloppy canary erase, this is at least verified to work,
        # otherwise vendor flashing program would fail.
        raise ValueError('Data length must be a multiple of 8.')
    last_programmed_address_words = (
        base_address_words
        + len(data) // 2
        - 1 # Otherwise it would be first non-erased address
    )
    with timer('Programming from %#04x to %#04x...' % (
        base_address_words,
        last_programmed_address_words,
    )):
        send(
            device,
            b'\x05\xaa\x55\x00' + struct.pack(
                '<HH',
                base_address_words,
                write_packet_count,
            ),
        )
        recv(
            device,
            b'\x05\xaa\x55\x00\xfa\xfa\xfa\xfa',
        )
        sending_offset = 0
        last_offset = len(data) - 8
        while data:
            print(
                '\rSending %#04x / %#04x... ' % (sending_offset, last_offset),
                end='',
            )
            sending_offset += 8
            while True:
                try:
                    send(device, data[:8])
                    # Flash packet is acked immediately, but firmware seems to
                    # clear USB interrupt at a different time, causing next
                    # transmission to be permanently lost. So sleep for 1ms,
                    # which is cheaper than triggering timeouts - and does the
                    # trick on my board at least. An entire flash takes under
                    # 4 seconds with this, so it should be acceptable.
                    time.sleep(.001)
                except usb1.USBErrorTimeout:
                    print('Timed out, retrying', end='')
                    continue
                else:
                    print('                   ', end='')
                    break
            data = data[8:]

def getChecksum(device):
    with timer('Getting 0x0000 to 0x27ff checksum...'):
        send(device, b'\x06\xaa\x55\x00\x00\x00\x00\x00')
        result, = struct.unpack(
            '<H',
            recv(device, b'\x06\xaa\x55\x00\xfa\xfa')[6:],
        )
    return result

def reboot(device):
    print('Asking device to reboot...')
    send(device, b'\x07\xaa\x55\x00\x00\x00\x00\x00')
    # There is (should be) no answer

def getCodeOptions(device):
    with timer('Retrieving code options...'):
        send(device, b'\x09\xaa\x55\x00\x00\x00\x00\x00')
        options_2ffc_2ffd = recv(device, b'\x09\xaa\x55\x00')[4:]
        send(device, b'\x09\xaa\x55\x01\x00\x00\x00\x00')
        options_2ffe_2fff = recv(device, b'\x09\xaa\x55\x01')[4:]
    return options_2ffc_2ffd + options_2ffe_2fff

def main():
    parser = argparse.ArgumentParser(
        description='Implement SN8 flashing protocol. USE AT YOUR OWN RISK.',
    )
    parser.add_argument(
        '-d', '--device',
        action='store',
        nargs='+',
        default=[
            '0c45:7500', # Enumerated in flash mode
            '17ef:6047', # Enumerated in firmware mode
        ],
        help='vendor:product of products to search for. If more than a '
          'single device match any of these, the command will fail, '
          'and you must use -s/--single to tell which you intend to use. '
          'Default: %(default)s',
    )
    parser.add_argument(
        '-s', '--single',
        nargs=1,
        help='[[bus]:][devnum] of the device to flash.',
    )
    parser.add_argument(
        'infile',
        help='The firmware to write in persistent device memory.',
    )
    args = parser.parse_args()
    with open(args.infile, 'rb') as infile:
        image = infile.read(max(EXPECTED_IMAGE_LENGTH_DICT))
    try:
        image = EXPECTED_IMAGE_LENGTH_DICT[len(image)](image)
    except KeyError:
        parser.error('Invalid image length, expected one of: %s' % (
            ', '.join(hex(x) for x in EXPECTED_IMAGE_LENGTH_DICT)
        ))
        assert len(image) == IMAGE_LENGTH
    image_code_options = image[0x2ffc * 2:]
    assert len(image_code_options) == 8
    # Strip the first 8 words, as these are programmed by flasher
    # automatically.
    # Strip everything beyond canary page.
    image = image[UNPROGRAMABLE_PREFIX_BYTES:FLASHER_BASE_ADDRESS_BYTES]
    if image[CANARY_ADDRESS_BYTES - 16:] != CANARY:
        parser.error(
            'Canary missing from image. '
            'Add ".ORG %#04x DW %#04x" to your source and rebuild image.' % (
                CANARY_ADDRESS_WORDS,
                struct.unpack('<H', CANARY)[0],
            ),
        )
    # Number of outgoing transactions needed to flash usable area.
    assert len(image) / 8 == 0x9fe
    all_programmed_expected_checksum = (
        FIRST_8_WORDS_CHECKSUM
        + sum(image)
    ) & 0xffff
    with usb1.USBContext() as usb:
        device_list = getCandidateDeviceList(
            usb=usb,
            bus_address=args.single,
            vid_pid_list=args.device,
        )
        try: # pylint: disable=unbalanced-tuple-unpacking
            device, = device_list
        except ValueError:
            parser.error(
                '%i device(s) found matching parameters.' % len(
                    device_list
                ),
            )
        print('Will be using %04x:%04x at %02i:%03i' % (
            device.getVendorID(),
            device.getProductID(),
            device.getBusNumber(),
            device.getDeviceAddress(),
        ))
        try:
            device_handle = device.open()
        except usb1.USBErrorAccess:
            parser.error(
                'Cannot open device %(bus)02i:%(dev)03i: permission denied.\n'
                'You may need to chown and/or chmod '
                '/dev/bus/usb/%(bus)03i/%(dev)03i' % {
                    'bus': device.getBusNumber(),
                    'dev': device.getDeviceAddress(),
                },
            )
        except usb1.USBErrorIO:
            parser.error(
                'Cannot open device %02i:%03i: I/O error.\n'
                'You may need to add the following rule to '
                '/etc/udev/rules.d/:\n'
                'ACTION=="add", SUBSYSTEM=="usb", ATTR{idVendor}=="%04x", ATTR{idProduct}=="%04x", ATTR{power/control}="on"' % (
                    device.getBusNumber(),
                    device.getDeviceAddress(),
                    device.getVendorID(),
                    device.getProductID(),
                ),
            )
        active_configuration = device_handle.getConfiguration()
        if active_configuration:
            # XXX: will detach kernel driver in order to be allowed access to
            # EP0. This means the keyboard (if booted in keyboard mode) will
            # not be usable. Warn user ? Look for another active keyboard ?
            for interface in range(len(device[active_configuration - 1])):
                try:
                    device_handle.detachKernelDriver(interface)
                except usb1.USBErrorNotFound:
                    pass
                except usb1.USBErrorNotSupported:
                    pass
                device_handle.claimInterface(interface)
        else:
            device_handle.setConfiguration(1)
            device_handle.claimInterface(0)
            # XXX: more ?
        try:
            unlocked = getFlashUnlockState(device_handle)
        except UnexpectedResponse:
            print('Not in flasher mode.')
            switchToFlasher(device_handle)
            unlocked = getFlashUnlockState(device_handle)
            assert not unlocked
        if not unlocked:
            print('Flash is locked.')
            unlockFlash(device_handle)
            if not getFlashUnlockState(device_handle):
                print('Failed to unlock flash, this tool need an update.')
                sys.exit(1)
        flash_code_options = getCodeOptions(device_handle)
        if flash_code_options != image_code_options:
            print('Code option mismatch:\n'
                '  Flash contains: %s\n'
                '  Image contains: %s' % (
                    hexdump(flash_code_options),
                    hexdump(image_code_options),
                )
            )
            sys.exit(1)
        # Here goes...
        erase(device_handle, 0, ERASABLE_PAGE_COUNT)
        # Erase is immediately ack'ed, but causes further commands to get
        # dropped (probably firmware is clearing the interrupt *after*
        # flashing, so any new communication after erase order gets lost
        # forever).
        # First, sleep a bit - it seems to take around 2.5s for flash to be
        # erased on my board.
        time.sleep(2.5)
        while True:
            try:
                all_erased_checksum = getChecksum(device_handle)
            except usb1.USBErrorTimeout:
                continue
            else:
                break
        if all_erased_checksum != ALL_ERASED_EXPECTED_CHECKSUM:
            print(
                'Post-erase checksum mismatch: %#04x expected, got '
                '%#04x.' % (
                    ALL_ERASED_EXPECTED_CHECKSUM,
                    all_erased_checksum,
                )
            )
            print(
                'You may have unerasable bits, which could cause the '
                'program to permanently brick your device.\n'
                'Interrupting flashing process.\n'
                'You may attempt another flash.',
            )
            sys.exit(2)
        program(device_handle, 0x0008, image)
        all_programmed_checksum = getChecksum(device_handle)
        if all_programmed_checksum != all_programmed_expected_checksum:
            print (
                'Post-programation checksum mismatch: %#04x expected, got '
                '%#04x.' % (
                    all_programmed_expected_checksum,
                    all_programmed_checksum,
                )
            )
            with timer('Re-erasing to avoid a brick...'):
                erase(device_handle, 0, ERASABLE_PAGE_COUNT)
        print('Success !')
        reboot(device_handle)

if __name__ == '__main__':
    main()
