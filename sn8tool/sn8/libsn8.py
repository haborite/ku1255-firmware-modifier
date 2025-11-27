# Copyright (C) 2016-2019  Vincent Pelletier <plr.vincent@gmail.com>
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
import configparser

NUL_SPACE = None # No operand
ZRO_SPACE = 0    # Operand is zero-page ram address
RAM_SPACE = 1    # Operand is ram address
ROM_SPACE = 2    # Operand is rom address
IMM_SPACE = 3    # Operand is immediate value

class MultiRange:
    def __init__(self, definition, min_boundary, max_boundary):
        self.range_list = range_list = []
        for span in definition.split(","):
            if '-' in span:
                start, stop = span.split('-')
                start = int(start, 0) if start else min_boundary
                stop = int(stop, 0) + 1 if stop else max_boundary
            else:
                start = int(span, 0)
                stop = start + 1
            range_list.append((start, stop))

    def __contains__(self, value):
        for start, stop in self.range_list:
            if start <= value < stop:
                return True
        return False

    def __iter__(self):
        for start, stop, in self.range_list:
            for value in range(start, stop):
                yield value

class CasedSafeConfigParser(configparser.ConfigParser):
    @staticmethod
    def optionxform(optionstr):
        return optionstr

def parseConfig(config_file_list):
    chip_config = CasedSafeConfigParser()
    for chip_file in config_file_list:
        chip_config.read_file(chip_file)
    chip = {
        'rom': {},
    }
    ram_range_symbol_list = []
    for section_name in chip_config.sections():
        section = dict(chip_config.items(section_name))
        if section_name == 'chip':
            section['rom_reserved'] = MultiRange(
                section.get('rom_reserved', ''),
                int(section['rom_start'], 0),
                int(section['rom_stop'], 0),
            )
            section['ram_start'] = ram_start = int(section['ram_start'], 0)
            section['ram_stop'] = ram_stop = int(section['ram_stop'], 0)
            section['ram_reserved'] = MultiRange(
                section.get('ram_reserved', ''),
                ram_start,
                ram_stop,
            )
        elif section_name in ('comment', 'rom', 'callee'):
            section = {
                int(address, 0):  value
                for address, value in section.items()
            }
        elif section_name.startswith('ram'):
            priority = 0
            if section_name.startswith('ram@'):
                _, ram_range = section_name.split('@')
                start, stop = ram_range.split('-')
            else:
                assert section_name in ('ram', 'ram-reserved'), repr(section_name)
                start = chip['chip']['rom_start']
                stop = chip['chip']['rom_stop']
                if section_name == 'ram-reserved':
                    # Low priority, to allow overriding in [ram] section.
                    priority = 1
                    chip[section_name] = section
            ram_range_symbol_list.append([priority, int(start, 0), int(stop, 0), section])
            continue
        elif section_name == 'code-option':
            new_section = {}
            for option_name, option_definition in section.items():
                address, mask, value_names = option_definition.split(' ', 2)
                address = int(address, 0)
                mask = int(mask, 0)
                if mask:
                    # Actual option
                    value_names = {
                        int(value, 0): name
                        for value, name in (
                            x.split('=') for x in value_names.split(' ')
                        )
                    }
                else:
                    # Hard-coded value
                    value_names = int(value_names, 0)
                new_section[option_name] = (address, mask, value_names)
            section = new_section
        chip[section_name] = section
    # Shorter ranges first, overriding larger maps
    ram_range_symbol_list.sort(key=lambda x: (x[0], x[2] - x[1]))
    chip['ram'] = ram_range_symbol_list
    chip['rom'].update(chip['callee'])
    chip['chip']['rom_start'] = int(chip['chip']['rom_start'], 0)
    chip['chip']['rom_stop'] = int(chip['chip']['rom_stop'], 0)
    return chip

# pylint: disable=unused-argument
def NONXT(
    entry_stack, jumper_dict, caller_dict, rom_symbol_dict, function_dict,
    addr, operand, function,
):
    """
    There is no fixed next-instruction address (RET & RETI).
    """

def NEXTI(
    entry_stack, jumper_dict, caller_dict, rom_symbol_dict, function_dict,
    addr, operand, function,
):
    """
    Always execute next instruction (normal instructions).
    """
    entry_stack.append((addr + 1, function))

def BRNCH(
    entry_stack, jumper_dict, caller_dict, rom_symbol_dict, function_dict,
    addr, operand, function,
):
    """
    Execute next instruction or the one after that (conditional branches).
    """
    entry_stack.append((addr + 1, function))
    entry_stack.append((addr + 2, function))

def JUMPI(
    entry_stack, jumper_dict, caller_dict, rom_symbol_dict, function_dict,
    addr, operand, function,
):
    """
    Jump to some far address (unconditional branches).
    Keep track of what jumped to where and declare a label if missing.
    """
    entry_stack.append((operand, function))
    if operand == addr + 1:
        # Delay jump
        return
    jumper_dict[operand].append(addr)
    if operand not in rom_symbol_dict:
        if function is None:
            offset = -1 # systematic disassembler, force absolute labels
        else:
            offset = operand - function_dict[function]
        rom_symbol_dict[operand] = (
            '%s_%04x' % (function, offset)
            if offset >= 0 else
            '_label_%04x' % operand
        )

def CALLI(
    entry_stack, jumper_dict, caller_dict, rom_symbol_dict, function_dict,
    addr, operand, function,
):
    """
    Call some address (call instructions).
    Keep track of what called where and declare a label if missing.
    """
    callee_name = rom_symbol_dict.setdefault(operand, 'func_%04x' % operand)
    if callee_name in function_dict:
        assert function_dict[callee_name] == operand, (
            callee_name,
            function_dict[operand],
            operand,
        )
    else:
        function_dict[callee_name] = operand
    entry_stack.append((operand, callee_name))
    entry_stack.append((addr + 1, function))
    caller_dict[operand].append(addr)
# pylint: enable=unused-argument

class Operand:
    def __ne__(self, other):
        return not self == other

class NoOperand(Operand):
    pass

class BitAddress(Operand):
    def __init__(self, address, bit):
        self.value = address
        if not 0 <= bit <= 7:
            raise ValueError('Bad bit index: %r' % bit)
        self.bit = bit

    def __repr__(self):
        return '<%s(%r.%r) at %x>' % (
            self.__class__.__name__,
            self.value,
            self.bit,
            id(self),
        )

    def __eq__(self, other):
        return isinstance(other, self.__class__) and other.value == self.value and other.bit == self.bit

class Address(Operand):
    def __init__(self, address):
        self.value = address

    def __repr__(self):
        return '<%s(%r) at %x>' % (
            self.__class__.__name__,
            self.value,
            id(self),
        )

    def __eq__(self, other):
        return isinstance(other, self.__class__) and other.value == self.value

class Immediate(Operand):
    def __init__(self, value):
        self.value = value

    def __repr__(self):
        return '<%s(%r) at %x>' % (
            self.__class__.__name__,
            self.value,
            id(self),
        )

    def __eq__(self, other):
        return isinstance(other, self.__class__) and other.value == self.value

opcode_dict = {
    0x00: (0x0000, NUL_SPACE, None, NEXTI, 'NOP',    NoOperand, NoOperand),
    0x02: (0x00ff, ZRO_SPACE, 'rw', NEXTI, 'B0XCH',  'A', Address),
    0x03: (0x00ff, ZRO_SPACE, 'rw', NEXTI, 'B0ADD',  Address, 'A'),
    0x04: (0x0000, NUL_SPACE, None, NEXTI, 'PUSH',   NoOperand, NoOperand),
    0x05: (0x0000, NUL_SPACE, None, NEXTI, 'POP',    NoOperand, NoOperand),
    0x06: (0x00ff, IMM_SPACE, None, BRNCH, 'CMPRS',  'A', Immediate),
    0x07: (0x00ff, RAM_SPACE, 'r ', BRNCH, 'CMPRS',  'A', Address),
    0x08: (0x00ff, RAM_SPACE, 'r ', NEXTI, 'RRC',    Address, NoOperand),
    0x09: (0x00ff, RAM_SPACE, 'rw', NEXTI, 'RRCM',   Address, NoOperand),
    0x0a: (0x00ff, RAM_SPACE, 'r ', NEXTI, 'RLC',    Address, NoOperand),
    0x0b: (0x00ff, RAM_SPACE, 'rw', NEXTI, 'RLCM',   Address, NoOperand),
    0x0d: (0x0000, NUL_SPACE, None, NEXTI, 'MOVC',   NoOperand, NoOperand),
    0x0e: (0x0000, NUL_SPACE, None, NONXT, 'RET',    NoOperand, NoOperand),
    0x0f: (0x0000, NUL_SPACE, None, NONXT, 'RETI',   NoOperand, NoOperand),
    0x10: (0x00ff, RAM_SPACE, 'r ', NEXTI, 'ADC',    'A', Address),
    0x11: (0x00ff, RAM_SPACE, 'rw', NEXTI, 'ADC',    Address, 'A'),
    0x12: (0x00ff, RAM_SPACE, 'r ', NEXTI, 'ADD',    'A', Address),
    0x13: (0x00ff, RAM_SPACE, 'rw', NEXTI, 'ADD',    Address, 'A'),
    0x14: (0x00ff, IMM_SPACE, None, NEXTI, 'ADD',    'A', Immediate),
    0x15: (0x00ff, RAM_SPACE, 'r ', BRNCH, 'INCS',   Address, NoOperand),
    0x16: (0x00ff, RAM_SPACE, 'rw', BRNCH, 'INCMS',  Address, NoOperand),
    0x17: (0x00ff, RAM_SPACE, 'r ', NEXTI, 'SWAP',   Address, NoOperand),
    0x18: (0x00ff, RAM_SPACE, 'r ', NEXTI, 'OR',     'A', Address),
    0x19: (0x00ff, RAM_SPACE, 'rw', NEXTI, 'OR',     Address, 'A'),
    0x1a: (0x00ff, IMM_SPACE, None, NEXTI, 'OR',     'A', Immediate),
    0x1b: (0x00ff, RAM_SPACE, 'r ', NEXTI, 'XOR',    'A', Address),
    0x1c: (0x00ff, RAM_SPACE, 'rw', NEXTI, 'XOR',    Address, 'A'),
    0x1d: (0x00ff, IMM_SPACE, None, NEXTI, 'XOR',    'A', Immediate),
    0x1e: (0x00ff, RAM_SPACE, 'r ', NEXTI, 'MOV',    'A', Address),
    0x1f: (0x00ff, RAM_SPACE, ' w', NEXTI, 'MOV',    Address, 'A'),
    0x20: (0x00ff, RAM_SPACE, 'r ', NEXTI, 'SBC',    'A', Address),
    0x21: (0x00ff, RAM_SPACE, 'rw', NEXTI, 'SBC',    Address, 'A'),
    0x22: (0x00ff, RAM_SPACE, 'r ', NEXTI, 'SUB',    'A', Address),
    0x23: (0x00ff, RAM_SPACE, 'rw', NEXTI, 'SUB',    Address, 'A'),
    0x24: (0x00ff, IMM_SPACE, None, NEXTI, 'SUB',    'A', Immediate),
    0x25: (0x00ff, RAM_SPACE, 'r ', BRNCH, 'DECS',   Address, NoOperand),
    0x26: (0x00ff, RAM_SPACE, 'rw', BRNCH, 'DECMS',  Address, NoOperand),
    0x27: (0x00ff, RAM_SPACE, 'rw', NEXTI, 'SWAPM',  Address, NoOperand),
    0x28: (0x00ff, RAM_SPACE, 'r ', NEXTI, 'AND',    'A', Address),
    0x29: (0x00ff, RAM_SPACE, 'rw', NEXTI, 'AND',    Address, 'A'),
    0x2a: (0x00ff, IMM_SPACE, None, NEXTI, 'AND',    'A', Immediate),
    0x2b: (0x00ff, RAM_SPACE, ' 0', NEXTI, 'CLR',    Address, NoOperand),
    0x2c: (0x00ff, RAM_SPACE, 'rw', NEXTI, 'XCH',    'A', Address),
    0x2d: (0x00ff, IMM_SPACE, None, NEXTI, 'MOV',    'A', Immediate),
    0x2e: (0x00ff, ZRO_SPACE, 'r ', NEXTI, 'B0MOV',  'A', Address),
    0x2f: (0x00ff, RAM_SPACE, ' w', NEXTI, 'B0MOV',  Address, 'A'),
    0x32: (0x00ff, IMM_SPACE, None, NEXTI, 'B0MOV',  'R', Immediate),
    0x33: (0x00ff, IMM_SPACE, None, NEXTI, 'B0MOV',  'Z', Immediate),
    0x34: (0x00ff, IMM_SPACE, None, NEXTI, 'B0MOV',  'Y', Immediate),
    0x36: (0x00ff, IMM_SPACE, None, NEXTI, 'B0MOV',  'PFLAG', Immediate),
    0x37: (0x00ff, IMM_SPACE, None, NEXTI, 'B0MOV',  'RBANK', Immediate),
    0x40: (0x00ff, RAM_SPACE, ' 0', NEXTI, 'BCLR',   BitAddress, NoOperand),
    0x48: (0x00ff, RAM_SPACE, ' 1', NEXTI, 'BSET',   BitAddress, NoOperand),
    0x50: (0x00ff, RAM_SPACE, 'r ', BRNCH, 'BTS0',   BitAddress, NoOperand),
    0x58: (0x00ff, RAM_SPACE, 'r ', BRNCH, 'BTS1',   BitAddress, NoOperand),
    0x60: (0x00ff, ZRO_SPACE, ' 0', NEXTI, 'B0BCLR', BitAddress, NoOperand),
    0x68: (0x00ff, ZRO_SPACE, ' 1', NEXTI, 'B0BSET', BitAddress, NoOperand),
    0x70: (0x00ff, ZRO_SPACE, 'r ', BRNCH, 'B0BTS0', BitAddress, NoOperand),
    0x78: (0x00ff, ZRO_SPACE, 'r ', BRNCH, 'B0BTS1', BitAddress, NoOperand),
    0x80: (0x3fff, ROM_SPACE, None, JUMPI, 'JMP',    Address, NoOperand),
    0xc0: (0x3fff, ROM_SPACE, None, CALLI, 'CALL',   Address, NoOperand),
}
