"""Run the original FlashWorker against a simulated SN8 flasher and record I/O.

Regenerates flash_golden.json used by tests/flash_replay.rs:

    git show c9c4f6a:sn8files/sn8/flashsn8_gui.py > flashsn8_gui.py
    python3 record.py flash_golden.json

PySide6 and usb1 are replaced by the minimal stubs in ./stubs.
"""
import json, random, struct, sys, time
import os; HERE = os.path.dirname(os.path.abspath(__file__)); sys.path[:0] = [os.path.join(HERE, 'stubs'), HERE]
import usb1
import flashsn8_gui as F

FIRST8 = F.FIRST_8_WORDS_CHECKSUM

class Device:
    def __init__(self, log, in_flasher=False, unlocked=False, code_options=b'\xf4\xff\x24\x79\x5a\xfa\x58\x45',
                 corrupt_program=False, timeouts=()):
        self.log = log
        self.in_flasher, self.unlocked = in_flasher, unlocked
        self.code_options = code_options
        self.flash = bytearray(b'\x5a' * 0x6000)
        self.corrupt_program = corrupt_program
        self.timeouts = set(timeouts)   # indexes of send() calls that time out
        self.nsend = 0
        self.pending = None
        self.prog = None
    # usb1 device/handle API
    def getVendorID(self): return 0x17ef
    def getProductID(self): return 0x6047
    def getBusNumber(self): return 1
    def getDeviceAddress(self): return 2
    def open(self): return self
    def getConfiguration(self): return 1
    def __getitem__(self, i): return [object()]
    def detachKernelDriver(self, i): pass
    def claimInterface(self, i): pass
    def checksum(self):
        return (FIRST8 + sum(self.flash[16:0x5000])) & 0xffff
    def controlWrite(self, request_type, request, value, index, timeout, data):
        i = self.nsend; self.nsend += 1
        data = bytes(data)
        if i in self.timeouts:
            self.log.append(['send-timeout', data.hex()])
            raise usb1.USBErrorTimeout()
        self.log.append(['send', data.hex()])
        if self.prog is not None:
            addr, left = self.prog
            chunk = data
            if self.corrupt_program and left == 1:
                chunk = bytes([chunk[0] ^ 1]) + chunk[1:]
            self.flash[addr*2:addr*2+8] = chunk
            self.prog = (addr + 4, left - 1) if left > 1 else None
            return
        cmd = data[0]
        if data == b'\xaa\x55\xa5\x5a\xff\x00\x33\xcc':
            self.in_flasher = True; return
        if not self.in_flasher:
            self.pending = b'\x00' * 8; return
        if cmd == 0x01: self.pending = b'\x01\xaa\x55\x00\x00\x03\x00\x00'
        elif cmd == 0x02:
            self.unlocked = True; self.pending = b'\x02\xaa\x55\x00\xfa\xfa\xfa\xfa'
        elif cmd == 0x03:
            self.pending = b'\x03\xaa\x55\x00' + (b'\xfa' * 4 if self.unlocked else b'\x00' * 4)
        elif cmd == 0x04:
            base, count = struct.unpack('<HH', data[4:])
            for a in range(base, base + count * 0x80):
                if a >= 8: self.flash[a*2:a*2+2] = b'\xff\xff'
            self.flash[0:16] = b'\x00\xa8\x80' + b'\x00' * 13
        elif cmd == 0x05:
            base, count = struct.unpack('<HH', data[4:])
            self.prog = (base, count)
            self.pending = b'\x05\xaa\x55\x00\xfa\xfa\xfa\xfa'
        elif cmd == 0x06:
            self.pending = b'\x06\xaa\x55\x00\xfa\xfa' + struct.pack('<H', self.checksum())
        elif cmd == 0x07: pass
        elif cmd == 0x09:
            half = self.code_options[:4] if data[3] == 0 else self.code_options[4:]
            self.pending = data[:4] + half
    def controlRead(self, request_type, request, value, index, timeout, length):
        r = self.pending if self.pending is not None else b'\x00' * 8
        self.pending = None
        self.log.append(['recv', r.hex()])
        return r

def synthetic_image(seed):
    # Deterministic LCG so the Rust test can regenerate the same image.
    img = bytearray()
    x = seed
    for _ in range(0x6000):
        x = (x * 1103515245 + 12345) & 0x7fffffff
        img.append((x >> 16) & 0xff)
    img[0x4ffe:0x5000] = b'\xaa\xaa'               # canary
    img[0x5ff8:0x6000] = b'\xf4\xff\x24\x79\x5a\xfa\x58\x45'  # code options
    return bytes(img)

def run(name, image, **dev_kw):
    log = []
    usb1.DEVICE = Device(log, **dev_kw)
    real_sleep = time.sleep
    time.sleep = lambda s: log.append(['sleep', s])
    path = f'{name}.bin'
    open(path, 'wb').write(image)
    w = F.FlashWorker(path, ['0c45:7500', '17ef:6047'], None)
    result = 'ok'
    try:
        w.run()
    except Exception as e:
        result = 'error'
    finally:
        time.sleep = real_sleep
    io = [e for e in log if e[0] in ('send', 'send-timeout', 'recv', 'sleep')]
    return {'name': name, 'image_seed': SEED, 'device': {k: (v.hex() if isinstance(v, bytes) else v) for k, v in dev_kw.items() if k != 'timeouts'} | {'timeouts': sorted(dev_kw.get('timeouts', ()))}, 'result': result, 'io': io}

SEED = 1255
import hashlib
def summarize(s):
    lines = []
    for e in s['io']:
        if e[0] == 'sleep':
            lines.append('sleep %d' % round(e[1] * 1000))
        else:
            lines.append('%s %s' % (e[0], e[1]))
    trace = ''.join(l + '\n' for l in lines)
    return {
        'name': s['name'],
        'device': s['device'],
        'corrupt_canary': s.get('corrupt_canary', False),
        'result': s['result'],
        'responses': [e[1] for e in s['io'] if e[0] == 'recv'],
        'trace_len': len(lines),
        'trace_sha256': hashlib.sha256(trace.encode()).hexdigest(),
    }
img = synthetic_image(SEED)
bad_canary = bytearray(img); bad_canary[0x4ffe] = 0
scenarios = [
    run('switch_and_unlock', img),
    run('already_unlocked', img, in_flasher=True, unlocked=True),
    run('send_timeout_retry', img, in_flasher=True, unlocked=True, timeouts=(40, 41, 1000)),
    run('code_option_mismatch', img, in_flasher=True, unlocked=True, code_options=b'\x00' * 8),
    run('program_checksum_mismatch', img, in_flasher=True, unlocked=True, corrupt_program=True),
]
r = run('bad_canary', bytes(bad_canary), in_flasher=True, unlocked=True)
r['corrupt_canary'] = True
scenarios.append(r)
out = {
    'generator': 'flashsn8_gui.py FlashWorker.run() against a simulated device',
    'image': {'length': 0x6000, 'lcg_seed': SEED, 'canary_at': 0x4ffe, 'code_options_at': 0x5ff8},
    'scenarios': [summarize(s) for s in scenarios],
}
json.dump(out, open(sys.argv[1], 'w'), indent=1)
open(sys.argv[1] + '.trace.txt', 'w').write('\n\n'.join(
    s['name'] + '\n' + ''.join(('sleep %d' % round(e[1]*1000) if e[0]=='sleep' else e[0]+' '+e[1]) + '\n' for e in s['io']) for s in scenarios))
