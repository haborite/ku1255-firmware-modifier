#!/usr/bin/env python
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
import argparse
from collections import defaultdict
import itertools
import os
from struct import unpack
import sys
from .libsn8 import (
    NUL_SPACE, ZRO_SPACE, RAM_SPACE, ROM_SPACE, IMM_SPACE,
    NONXT, NEXTI, BRNCH, JUMPI,
    NoOperand,
    opcode_dict,
    parseConfig,
)

ram_range_symbol_list = []
rom_symbol_dict = {}
entry_stack = []

caller_dict = defaultdict(list)
jumper_dict = defaultdict(list)
ram_symbol_usage_dict = defaultdict(list)
function_dict = {}
line_ownership_dict = {}
jump_dict = {}

def asPrintable(value):
    if 0x20 <= value < 0x7f:
        return chr(value)
    return '.'

branch_opcode_implicit_operand_dict = {
    0x06: 'A',
    0x07: 'A',
    0x15: '0',
    0x16: '0',
    0x25: '0',
    0x26: '0',
    0x50: '0',
    0x58: '1',
    0x70: '0',
    0x78: '1',
}

def disassemble(address, instruction, function):
    bincode = instruction >> 8
    if bincode >= 0x80:
        opcode_key = bincode & 0xc0
        is_bit = False
    elif bincode >= 0x40:
        opcode_key = bincode & 0xf8
        is_bit = True
    else:
        opcode_key = bincode
        is_bit = False
    try:
        (
            mask,
            opspace,
            opmode,
            jump_action,
            opcode,
            left_operand,
            right_operand,
        ) = opcode_dict[opcode_key]
    except KeyError:
        NEXTI(
            entry_stack, jumper_dict, caller_dict, rom_symbol_dict, function_dict,
            address, None, None,
        )
        return 'DW\t0x%04x\t; ILLEGAL OPCODE' % (instruction, )
    if opspace == NUL_SPACE:
        jump_action(
            entry_stack, jumper_dict, caller_dict, rom_symbol_dict, function_dict,
            address, None, function,
        )
        if jump_action is NONXT:
            jump_dict[address] = ()
    else:
        operand = instruction & mask
        jump_action(
            entry_stack, jumper_dict, caller_dict, rom_symbol_dict, function_dict,
            address, operand, function,
        )
        if opspace == ROM_SPACE:
            symbol = rom_symbol_dict.get(operand)
            operand_fmt = '0x%04x'
            assert not is_bit
        elif opspace == IMM_SPACE:
            symbol = None
            operand_fmt = '#0x%02x'
            assert not is_bit
        else: # ZRO & RAM
            bit_dict = {}
            symbol = None
            for _, start, stop, ram_symbol_dict in ram_range_symbol_list:
                if start <= address <= stop and operand in ram_symbol_dict:
                    range_symbol, range_bit_dict = ram_symbol_dict[operand]
                    if symbol is None:
                        symbol = range_symbol
                    for bit_address, bit_symbol in range_bit_dict.items():
                        bit_dict.setdefault(bit_address, bit_symbol)
            operand_fmt = '0x%02x'
            if is_bit:
                bit_address = bincode & 0x7
                bit_symbol = bit_dict.get(bit_address)
                if bit_symbol is None:
                    if symbol is None:
                        operand_fmt += '.%i' % bit_address
                    else:
                        symbol += '.%i' % bit_address
                else:
                    symbol = bit_symbol
        if symbol is None:
            if jump_action is JUMPI and operand == address + 1:
                symbol = '$+1'
            else:
                symbol = operand_fmt % operand
        if opspace in (ZRO_SPACE, RAM_SPACE):
            ram_symbol_usage_dict[symbol].append((address, opmode))
        if jump_action is BRNCH:
            implicit_operand = branch_opcode_implicit_operand_dict[opcode_key]
            if implicit_operand == 'A':
                lhs = implicit_operand
                rhs = symbol
            else:
                lhs = symbol
                rhs = implicit_operand
            jump_dict[address] = (
                (address + 1, '%s != %s' % (lhs, rhs), 'red'),
                (address + 2, '%s == %s' % (lhs, rhs), 'green'),
            )
        elif jump_action is JUMPI:
            if operand != address + 1:
                jump_dict[address] = (
                    (operand, None, 'black'),
                )
        operand_caption = ', '.join(
            x if isinstance(x, str) else symbol
            for x in (left_operand, right_operand)
            if x is not NoOperand
        )
        if operand_caption:
            opcode += '\t' + operand_caption
    return opcode

def systematic(rom):
    disassembled = {}
    for address, instruction in rom.items():
        try:
            opcode = disassemble(address, instruction, None)
        except KeyError:
            opcode = 'DW\t0x%04x\t; %s%s' % (
                instruction,
                asPrintable(instruction >> 8),
                asPrintable(instruction & 0xff),
            )
        disassembled[address] = opcode
    return disassembled

def walker(rom):
    disassembled = {}
    while entry_stack:
        address, function = entry_stack.pop()
        try:
            instruction = rom.pop(address)
        except KeyError:
            continue # Already disassembled
        line_owner = line_ownership_dict.setdefault(address, function)
        if line_owner != function:
            print(
                'Line ownership disagreement: 0x%04x claimed by %r and %r' % (
                    address,
                    line_ownership_dict[address],
                    function,
                ),
            )
            function = line_owner
        if instruction in (
                    0x03ce, # B0ADD PCL, A
                    0x13ce, # ADD   PCL, A
                ):
            # A jump table follows.
            # XXX: assumes jump tables are only composed of JMP (0x8000),
            # CALL (0xc000) and NOP (0x0000). Jump table is supposed to end
            # at the first unexpected instruction, meaning that the code
            # after the last entry will not be jumped to as part of the
            # table.
            target_list = []
            for a_value, sub_address in enumerate(itertools.count(address + 1)):
                if sub_address in rom and (
                    rom[sub_address] & 0x8000 or
                    rom[sub_address] == 0x0000
                ):
                    target_list.append((
                        sub_address,
                        'A == %#04x' % a_value,
                        'orange',
                    ))
                    if sub_address not in disassembled:
                        entry_stack.append((sub_address, function))
                else:
                    break
            assert target_list
            jump_dict[address] = tuple(target_list)
        disassembled[address] = disassemble(address, instruction, function)
    # Convert any disassembled word into a data word.
    # Agglutinate non-disassembled portions to know their length.
    # Ignore runs of nulls longer than 6 words (12 bytes), for readability.
    # XXX: 6 words is a totally arbitrary value.
    data_chunk_dict = defaultdict(list)
    next_address = None
    current_chunk = None
    for address, value in sorted(rom.items()):
        if address != next_address:
            current_chunk = data_chunk_dict[address]
        next_address = address + 1
        current_chunk.append(value)
    for chunk_address, value_list in data_chunk_dict.items():
        for count, value in [
                    (len(list(g)), k) for k, g in itertools.groupby(value_list)
                ]:
            if value or count < 7:
                for offset in range(count):
                    disassembled[chunk_address + offset] = (
                        'DW\t0x%04x\t; %s%s' % (
                            value,
                            asPrintable(value >> 8),
                            asPrintable(value & 0xff),
                        )
                    )
            chunk_address += count
    return disassembled

def tabstop(prefix, position, suffix, tabwidth=8):
    """
    Returns <prefix> and <suffix> so that <suffix> starts at tabstop
    <position>.
    """
    tabcount = max(0, position - len(prefix.expandtabs(tabwidth)) // tabwidth)
    return prefix + (
      '\t' * tabcount
      if tabcount else
      ' '
    ) + suffix
COMMENT_POSITION = 5 # 5th tab

def getFunctionNameAndOffset(address):
    function_name = line_ownership_dict[address]
    return '%s%#+x' % (function_name, address - function_dict[function_name])

def main():
    method_dict = {
        'walker': walker,
        'systematic': systematic,
    }
    parser = argparse.ArgumentParser(description='SN8F228(8|3|31) USB 2.0'
        ' full-speed 8-bits microcontroller disassembler')
    parser.add_argument('-c', '--chip', action='append', required=True,
        type=argparse.FileType('r'), help='Chip definition (name, regisers, '
            'labels, comments). Document disassembled code in a separate '
            'configuration file to automatically re-annotate on next '
            'disassembly. Can be specified multiple time, files will be '
            'loaded and merged in memory in the order they are specified.')
    parser.add_argument('-m', '--method', default='walker', choices=method_dict,
        help='Disassembly strategy: "walker" follows code path to avoid '
            'disassembling data, "systematic" just disassembes all memory. '
            'Default: %(default)s.')
    parser.add_argument('-H', '--skip-header', action='store_true',
        help='Skip the SH8 header used by SN8 C Studio for object files')
    parser.add_argument('-o', '--output', type=argparse.FileType('w'),
        default=sys.stdout, help='Path to write assembly code to, or - '
            'for stdout (default).')
    parser.add_argument('--dot', action='store_true',
        help='Write function graphs in dot format in <basename(input)> folder.')
    parser.add_argument('input', nargs='?', default='-',
        help='Path of binary rom image file to disassemble, or - for stdin '
            '(default).')
    args = parser.parse_args()
    write = args.output.write
    chip = parseConfig(args.chip)
    ram_range_symbol_list.extend(chip['ram'])
    rom_symbol_dict.update(chip['rom'])
    rom_reserved_set = chip['chip']['rom_reserved']
    # Note: only look for duplicate ram symbols, not rom.
    flat_ram_symbol_dict = defaultdict(lambda: ([], defaultdict(list)))
    reverse_dict = {}
    for ram_range_symbol_entry in chip['ram']:
        _, _, _, ram_symbol_dict = ram_range_symbol_entry
        ram_range_symbol_entry[3] = new_ram_symbol_dict = {}
        for address, name in ram_symbol_dict.items():
            if name:
                if reverse_dict.get(name, address) != address:
                    raise ValueError(
                        'Name %r is used for different addresses: %r and %r' % (
                            name, reverse_dict[name], address,
                        )
                    )
                reverse_dict[name] = address
            else:
                # To un-define a defined symbol, by giving an empty name.
                # Ex: to override register names over a piece of code using
                # non-zero bank. XXX: B0MOV will not properly resolve
                name = None
            if '.' in address:
                address, bit = address.split('.')
            else:
                bit = None
            address = int(address, 0)
            flat_name_list, flat_bit_dict = flat_ram_symbol_dict[address]
            try:
                entry = new_ram_symbol_dict[address]
            except KeyError:
                entry = new_ram_symbol_dict[address] = [None, {}]
            if bit is None:
                assert entry[0] is None, (address, name, entry)
                entry[0] = name
                flat_name_list.append(name)
            else:
                bit = int(bit)
                entry[1][bit] = name
                flat_bit_dict[bit].append(name)
    # chip name must be set
    write('CHIP\t' + chip['chip']['name'] + '\n')
    function_addr_name_dict = chip['callee']
    line_ownership_dict.update(function_addr_name_dict)
    function_dict.update((y, x) for x, y in function_addr_name_dict.items())
    entry_stack.extend(line_ownership_dict.items())
    comment_dict = chip['comment']
    for reserved in rom_reserved_set:
        comment_dict.setdefault(reserved, 'Reserved')
    if args.input == '-':
        input = sys.stdin
        try:
            input = input.buffer
        except AttributeError:
            pass
    else:
        input = open(args.input, 'rb')
    read = input.read
    if args.skip_header:
        read(88)
    rom = {}
    for address in range(
        chip['chip']['rom_start'],
        chip['chip']['rom_stop'] + 1,
    ):
        instruction = read(2)
        if len(instruction) != 2:
            break
        rom[address], = unpack('<H', instruction)
    if read(1):
        print('Ignoring data past end of ROM')
    write('//{{SONIX_CODE_OPTION\n')
    for option_name, (address, mask, value_names) in chip['code-option'].items():
        if mask:
            try:
                option_value = rom[address] & mask
            except KeyError:
                continue
            while not mask & 1:
                mask >>= 1
                option_value >>= 1
            write(tabstop(
                '\t.Code_Option\t' + option_name,
                5,
                '"' + value_names[option_value] + '"\n',
            ))
    write('//}}SONIX_CODE_OPTION\n')
    disassembled_dict = method_dict[args.method](rom)
    write('.DATA\n')
    ram_reserved_name_set = set(chip.get('ram-reserved', {}).values())
    for address, (flat_name_list, flat_bit_dict) in sorted(flat_ram_symbol_dict.items()):
        for name in flat_name_list:
            if name in ram_reserved_name_set:
                continue
            write('%s\tEQU\t0x%02x\n' % (name, address))
        for bit_number, bit_name_list in sorted(flat_bit_dict.items()):
            for bit_name in bit_name_list:
                if bit_name in ram_reserved_name_set:
                    continue
                write('%s\tEQU\t0x%02x.%i\n' % (bit_name, address, bit_number))
    if ram_symbol_usage_dict:
        write('; RAM address usage\n')
        for address, accessor_list in sorted(ram_symbol_usage_dict.items()):
            write('; %s:\n' % (address, ))
            for accessor_address, mode in sorted(accessor_list):
                write(';\t%s %s\n' % (
                    mode,
                    getFunctionNameAndOffset(accessor_address),
                ))
    write('.CODE\n')
    next_key = None
    dot = args.dot
    if dot:
        dot_dict = defaultdict(list)
        dot_path = os.path.splitext(input.name)[0]
        if not os.path.exists(dot_path):
            os.mkdir(dot_path)
        def dot_escape(value):
            return value.replace('\\', '\\\\').replace('"', '\\"')
        def dot_quote(value):
            return '"' + dot_escape(value) + '"'
        def dot_addline(address, line, is_instruction=True):
            try:
                function_id = line_ownership_dict[address]
            except KeyError:
                return
            dot_dict[function_id].append((
                address,
                dot_escape(
                    (
                        (
                            ('%04x ' % address)
                            if is_instruction else
                            '     '
                        ) + line
                    ).expandtabs(),
                ) + '\\l',
                is_instruction,
            ))
        def dot_write():
            def writenode():
                node_id = dot_quote(getFunctionNameAndOffset(node_base_address))
                dot_file.write('%s[label="%s"]\n' % (
                    node_id,
                    ''.join(node_line_list)
                ))
                del node_line_list[:]
                return node_id
            jumped_set = {x for y in jump_dict.values() for x, _, _ in y}
            for function, line_list in dot_dict.items():
                with open(
                    os.path.join(dot_path, function + '.dot'),
                    'w',
                    encoding='ascii',
                ) as dot_file:
                    dot_file.write('strict digraph {\nedge [fontname="Courier"]\nnode [shape="box",fontname="Courier"]\n')
                    node_line_list = []
                    node_base_address = None
                    for address, line, is_instruction in line_list:
                        if node_base_address is None:
                            if is_instruction:
                                node_base_address = address
                        elif address in jumped_set:
                            node_id = writenode()
                            dot_file.write('%s->%s[color="black"]\n' % (
                                node_id,
                                dot_quote(getFunctionNameAndOffset(address)),
                            ))
                            node_base_address = address if is_instruction else None
                        node_line_list.append(line)
                        if is_instruction and address in jump_dict:
                            node_id = writenode()
                            node_base_address = None
                            for to_address, label, color in jump_dict[address]:
                                to_function = line_ownership_dict[to_address]
                                dot_file.write('%s->%s[color="%s"' % (
                                    node_id,
                                    dot_quote(getFunctionNameAndOffset(to_address)),
                                    color,
                                ))
                                if label:
                                    dot_file.write(',label="%s"' % label)
                                dot_file.write(']\n')
                                if to_function != function:
                                    dot_file.write('%s[shape="ellipse"]' % (
                                        dot_quote(getFunctionNameAndOffset(to_address)),
                                    ))
                    if node_line_list:
                        writenode()
                    dot_file.write('}\n')
        with open(
            os.path.join(dot_path, '_index.dot'),
            'w',
            encoding='ascii',
        ) as dot_file:
            dot_file.write('strict digraph {\nedge [fontname="Courier"]\nnode [shape="box",fontname="Courier"]\n')
            for callee, caller_list in caller_dict.items():
                try:
                    callee = line_ownership_dict[callee]
                except KeyError:
                    callee = 'func_%04x' % callee
                callee = dot_quote(callee)
                for caller in caller_list:
                    try:
                        caller = line_ownership_dict[caller]
                    except KeyError:
                        caller = '%#06x' % caller
                    dot_file.write('%s->%s\n' % (
                        dot_quote(caller),
                        callee,
                    ))
            dot_file.write('}\n')
    else:
        dot_addline = lambda address, line, is_instruction=True: None
        dot_write = lambda: None
    for key in sorted(disassembled_dict):
        disassembled = disassembled_dict[key]
        if key != next_key:
            if key in rom_reserved_set:
                write(';')
            write('ORG 0x%04x\n' % key)
        if not key & 0xf:
            write(tabstop('', COMMENT_POSITION, '; 0x%04x\n' % key))
        next_key = key + 1
        if key in rom_symbol_dict:
            dot_addline(key, rom_symbol_dict[key] + ':', is_instruction=False)
            write(tabstop(rom_symbol_dict[key] + ':', COMMENT_POSITION, ';'))
            if key in caller_dict:
                write(' Called from ' + ', '.join(
                    getFunctionNameAndOffset(x)
                    for x in sorted(caller_dict.pop(key))
                ))
            if key in jumper_dict:
                write(' Jumped from ' + ', '.join(
                    getFunctionNameAndOffset(x)
                    for x in sorted(jumper_dict.pop(key))
                ))
            write('\n')
        else:
            assert key not in caller_dict, key
            assert key not in jumper_dict, key
        line = (';' if key in rom_reserved_set else '') + '\t' + disassembled
        if key in comment_dict:
            line = tabstop(line, COMMENT_POSITION, '; ' + comment_dict[key])
        write(line + '\n')
        dot_addline(key, line)
    dot_write()
    if caller_dict:
        write('; Unknown calls:\n')
        for address, caller_list in caller_dict.items():
            write('ORG 0x%04x\n' % address)
            write(
                rom_symbol_dict[address] + ': ; Called from ' +
                ', '.join(
                    getFunctionNameAndOffset(x)
                    for x in caller_list
                ) + '\n'
            )
            write('\tJMP\t' + rom_symbol_dict[address] + '\n')
    if jumper_dict:
        write('; Unknown jumps:\n')
        for address, jumper_list in jumper_dict.items():
            write('ORG 0x%04x\n' % address)
            write(
                rom_symbol_dict[address] + ': ; Jumped from ' +
                ', '.join(
                    getFunctionNameAndOffset(x)
                    for x in jumper_list
                ) + '\n'
            )
            write('\tJMP\t' + rom_symbol_dict[address] + '\n')
    write('ENDP\n')

if __name__ == '__main__':
    main()
