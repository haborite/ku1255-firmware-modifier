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
SN8 assembler.
"""
import argparse
import ast
from collections import defaultdict
from io import StringIO
import itertools
import os.path
from struct import pack, unpack
import sys
import traceback
import warnings
from ply.yacc import yacc
from ply.lex import lex
from .libsn8 import (
    opcode_dict,
    BitAddress, Address, Immediate, NoOperand,
    ZRO_SPACE, RAM_SPACE, ROM_SPACE,
    parseConfig,
)

__all__ = ('assemble', )

CONFIG_DIR = os.path.dirname(__file__)

MARKER = object()
INSTRUCTION_TOKEN = 'INSTRUCTION'
NUMBER_TOKEN = 'NUMBER'
STRING_TOKEN = 'STRING'
INSTRUCTION_DICT = defaultdict(list)
for _opcode, (_, _, _, _, _instruction, _, _) in opcode_dict.items():
    INSTRUCTION_DICT[_instruction].append(_opcode)
del _opcode
INSTRUCTION_DICT = dict(INSTRUCTION_DICT)
BYTE_SELECTOR_FUNCTION_DICT = {
    'L': lambda x: x & 0xff,
    'M': lambda x: (x >> 8) & 0xff,
    'H': lambda x: (x >> 16) & 0xff,
}
TOKEN_WORD_LIST = (
    'DB',
    'DW',
    'DS',
    'EQU',
    'ORG',
    'CHIP',
    'ENDP',
    'INCLUDE',
    'INCLUDEBIN',
)
NO_OPERAND = NoOperand()
AUTO_LABEL = '@@'
AUTO_LABEL_FORWARD = '@F'
AUTO_LABEL_BACKWARD = '@B'

class ParserFrame:
    has_set_origin = False
    partial_word = False
    def __init__(self, parser, filename, address):
        self.parser = parser
        self.filename = filename
        self.address_until_origin = self.address = address
        self.identifier_dict = {}
        self.label_referer_dict = defaultdict(list)

    def setAddress(self, address):
        if not self.has_set_origin:
            self.address_until_origin = address
        self.address = address

    def setOrigin(self, address):
        self.has_set_origin = True
        self.address = address
        self.partial_word = False

class Allocator:
    def __init__(self, start, stop):
        # List of uninterupted free ranges, smallest first.
        self.free_list = [[stop - start + 1, start]]

    def allocate(self, length):
        for entry in self.free_list:
            free_length, free_start = entry
            if length < free_length:
                # Free space after, updzte chunk
                entry[0] -= length
                entry[1] += length
                break
            elif length == free_length:
                # No space left in chunk, remove
                self.free_list.remove(entry)
                break
        else:
            raise MemoryError('No large-enough free memory chunk')
        self.free_list.sort()
        return free_start

    def allocateAt(self, at, length):
        for entry in self.free_list:
            free_length, free_start = entry
            free_stop = free_start + free_length
            stop = at + length
            if free_start <= at and stop <= free_stop:
                if free_start < at:
                    # Free space before, update chunk
                    entry[0] = at - free_start
                    if stop < free_stop:
                        # Free space after, create chunk
                        self.free_list.append([
                            free_stop - stop,
                            stop,
                        ])
                elif stop < free_stop:
                    # Free space after only, update chunk
                    entry[0] = free_stop - stop
                    entry[1] = stop
                else:
                    # No space left in chunk, remove
                    self.free_list.remove(entry)
                break
        else:
            raise MemoryError('No large-enough free memory chunk')
        self.free_list.sort()
        return free_start

class Assembler:
    tokens = (
        'CODE',
        'DATA',
        'ALIGN',
        'IDENTIFIER',
        STRING_TOKEN,
        INSTRUCTION_TOKEN,
        'BIT_SELECTOR',
        NUMBER_TOKEN,
        'EOL',
        'CODE_OPTIONS_BEGIN',
        'CODE_OPTIONS_END',
        'CODE_OPTION',
        'RELATIVE_ADDRESS',
        'BYTE_SELECTOR',
    ) + TOKEN_WORD_LIST
    literals = '#,:'
    t_ignore = ' \t'
    t_ignore_COMMENT = ';.*'
    t_CODE = r'\.CODE'
    t_DATA = r'\.DATA'
    t_ALIGN = r'\.ALIGN'
    t_CODE_OPTIONS_BEGIN = '//{{SONIX_CODE_OPTION'
    t_CODE_OPTIONS_END = '//}}SONIX_CODE_OPTION'
    t_CODE_OPTION = r'\.Code_Option'

    @staticmethod
    def t_BYTE_SELECTOR(token):
        r'''\$[HML]'''
        # Note: must be declared above t_RELATIVE_ADDRESS to have precedence.
        token.value = BYTE_SELECTOR_FUNCTION_DICT[token.value[1]]
        return token

    @staticmethod
    def t_RELATIVE_ADDRESS(token):
        r'''\$([+-][0-9]+)?'''
        token.value = int(token.value[1:], 0) if len(token.value) > 1 else 0
        return token

    @staticmethod
    def t_QUOTED_STRING(token):
        r'''"(\\.|[^\\"\n])*"'''
        # A double-quote, followed by escaped double-quotes or anything else
        # than an escape, double-quotes or newline, followed by a double-quote.
        token.value = ast.literal_eval(token.value)
        token.type = STRING_TOKEN
        return token

    # XXX: accept non-ascii identifiers ?
    @staticmethod
    def t_IDENTIFIER(token):
        '''[a-zA-Z_@][a-zA-Z_0-9@]*'''
        if token.value in INSTRUCTION_DICT:
            token.type = INSTRUCTION_TOKEN
        elif token.value in TOKEN_WORD_LIST:
            token.type = token.value
        return token

    @staticmethod
    def t_BIT_SELECTOR(token):
        r'''\.[0-7]'''
        token.value = int(token.value[1], 10)
        return token

    @staticmethod
    def t_HEX_NUMBER(token):
        '''0([Xx][0-9a-fA-F]+|[0-9a-fA-F]*[Hh])'''
        token.value = int(token.value.rstrip('Hh'), 16)
        token.type = NUMBER_TOKEN
        return token

    @staticmethod
    def t_BIN_NUMBER(token):
        '''0([Bb][01]+|[01]*[Bb])'''
        token.value = int(token.value.rstrip('Bb'), 2)
        token.type = NUMBER_TOKEN
        return token

    @staticmethod
    def t_DEC_NUMBER(token):
        '''[0-9]+'''
        token.value = int(token.value, 10)
        token.type = NUMBER_TOKEN
        return token

    @staticmethod
    def t_CHAR(token):
        """'(.)'"""
        token.value = ord(token.value.strip("'").encode('ascii'))
        token.type = NUMBER_TOKEN
        return token

    @staticmethod
    def t_EOL(token):
        r'''\n+'''
        token.lexer.lineno += len(token.value)
        return token

    def t_error(self, token):
        print("%s:%i: Illegal character '%r'" % (
            self.filename,
            token.lexer.lineno,
            token.value[0],
        ))
        token.lexer.skip(1)

    def p_file(self, production):
        '''
        file : early_includes sections ENDP EOL
             | early_includes sections
        '''
        self.label_referer_dict.pop(AUTO_LABEL_BACKWARD, None)
        if len(self._parser_stack) == 1:
            # Any remaining undefined label is an error.
            undefined_identifier_list = list(self.label_referer_dict)
        else:
            # Only local undefined labels are an error.
            undefined_identifier_list = [
                x for x in self.label_referer_dict if x.startswith('_')
            ]
        if undefined_identifier_list:
            undefined_identifier_list.sort()
            raise NameError('Undefined labels:\n  %s' % (
                '\n  '.join(undefined_identifier_list),
            ))

    def p_chip(self, production):
        '''
        chip : CHIP IDENTIFIER EOL
        '''
        if self.chip is not None:
            raise ValueError('%s:%i: Redefining chip type.' % (
                self.filename,
                production.lineno(1),
            ))
        with open(
            os.path.join(CONFIG_DIR, production[2].lower() + '.cfg'),
            'r',
            encoding='utf-8',
        ) as config:
            chip = parseConfig([config])
        self.chip = chip
        # Sanity check
        if chip['chip']['name'].lower() != production[2].lower():
            raise ValueError('Inconsistent chip config file: %r' % config.name)
        for address, mask, value_names in chip['code-option'].values():
            if mask == 0:
                self.rom[address] = value_names # Actually a single value
        for address, name in chip['rom'].items():
            self.chip_identifier_dict[name] = Address(address)
        for _, _, _, ram_dict in chip['ram']:
            for address, name in ram_dict.items():
                if '.' in address:
                    word_address, bit_address = address.split('.')
                    address = BitAddress(
                        int(word_address, 0),
                        int(bit_address, 0),
                    )
                else:
                    address = Address(int(address, 0))
                self.chip_identifier_dict[name] = address
        self.ram_allocator = allocator = Allocator(
            chip['chip']['ram_start'],
            chip['chip']['ram_stop'],
        )
        for start, stop in chip['chip']['ram_reserved'].range_list:
            allocator.allocateAt(start, stop - start)

    @staticmethod
    def p_noop(production):
        '''
        early_includes : early_includes include
                       | early_includes EOL
                       | empty
        code_options_block : CODE_OPTIONS_BEGIN EOL code_options CODE_OPTIONS_END EOL
        code_options : code_options code_option
                     | code_options EOL
                     | empty
        sections : sections section
                 | empty
        section : CODE EOL emitables
                | DATA EOL declarations
                | code_options_block
                | chip
        declarations : declarations declaration
                     | declarations EOL
                     | empty
        emitables : emitables emitable
                  | emitables EOL
                  | empty
        emitable : include
        empty :
        '''

    def p_code_option(self, production):
        '''
        code_option : CODE_OPTION IDENTIFIER IDENTIFIER EOL
                    | CODE_OPTION IDENTIFIER STRING EOL
        '''
        address, mask, value_names = self.chip['code-option'][production[2]]
        assert mask
        if self.rom[address] & mask:
            raise ValueError('Duplicate code option declaration')
        shift = 0
        while mask & 1 == 0:
            shift += 1
            mask >>= 1
        self.rom[address] |= {
            y: x for x, y in value_names.items()
        }[production[3]] << shift

    def _findFile(self, include_filename):
        if self.include is None:
            raise ValueError('Imports are disabled')
        for include_path in [os.path.dirname(self.filename)] + self.include:
            path = os.path.join(include_path, include_filename)
            if os.path.exists(path):
                return path
        return include_filename

    def p_include(self, production):
        '''
        include : INCLUDE STRING EOL
        '''
        try:
            with open(
                self._findFile(production[2]),
                'r',
                encoding='utf-8',
            ) as infile:
                self._parse(infile)
        except Exception:
            print('Error while processing %r, included from %s:%i:' % (
                production[2],
                self.filename,
                production.lineno(1),
            ))
            traceback.print_exc()
            # The important thing is to not re-raise a SyntaxError, as then
            # it becomes handled by current parser while the error comes from
            # another file.
            raise ValueError

    def p_includebin(self, production):
        '''
        include : INCLUDEBIN STRING EOL
        '''
        with open(self._findFile(production[2]), 'rb') as infile:
            while True:
                chunk = infile.read(2)
                if len(chunk) == 2:
                    self.write(unpack('<H', chunk)[0])
                else:
                    if chunk:
                        self.write(ord(chunk))
                    break

    def p_bit_address(self, production):
        '''
        bit_address : address BIT_SELECTOR
        '''
        address = production[1]
        if isinstance(address, BitAddress):
            raise TypeError('%s:%i: %r is already a bit address.' % (
                self.filename,
                production.lineno(1),
                address,
            ))
        production[0] = BitAddress(address.value, production[2])

    @staticmethod
    def p_address_number(production):
        '''
        address : NUMBER
        operand : NUMBER
        '''
        production[0] = Address(production[1])

    def p_identifier(self, production):
        '''
        resolved_identifier : IDENTIFIER
        '''
        try:
            production[0] = self.getIdentifier(production[1])
        except KeyError:
            raise NameError(production[1]) from None

    @staticmethod
    def p_passthrough(production):
        '''
        address : resolved_identifier
        operand : bit_address
        '''
        production[0] = production[1]

    @staticmethod
    def p_partial_identifier(production):
        '''
        address : resolved_identifier BYTE_SELECTOR
        operand : resolved_identifier BYTE_SELECTOR
        '''
        value = production[1]
        if isinstance(value, BitAddress):
            raise TypeError('Cannot select a byte from a bit address.')
        production[0] = Address(production[2](value.value))

    def p_declare_address(self, production):
        '''
        declaration : IDENTIFIER EQU address EOL
                    | IDENTIFIER EQU bit_address EOL
        '''
        self.declare(production[1], production[3])

    def p_declare_size(self, production):
        '''
        declaration : IDENTIFIER DS NUMBER EOL
        '''
        byte_count = production[3]
        if byte_count < 1:
            raise ValueError(
                '%s:%i: Trying to allocate less than one byte.' % (
                    self.filename,
                    production.lineno(1),
                ),
            )
        self.declare(
            production[1],
            Address(self.ram_allocator.allocate(byte_count)),
        )

    def declare(self, identifier, value):
        if self.getIdentifier(identifier, value, acquire=False) != value:
            warnings.warn('Redefining %r (was %r) with different value: %r' % (
                identifier,
                self.getIdentifier(identifier),
                value,
            ))
        self.setIdentifier(identifier, value)

    def p_emitable_org_number(self, production):
        '''
        emitable : ORG NUMBER EOL
        '''
        self._parser_stack[-1].setOrigin(production[2])

    def p_emitable_org_identifier(self, production):
        '''
        emitable : ORG IDENTIFIER EOL
        '''
        address = self.getIdentifier(production[2])
        if not isinstance(address, Address):
            raise TypeError('Wrong identifier type for ORG')
        self._parser_stack[-1].setOrigin(address.value)

    def p_emitable_align(self, production):
        '''
        emitable : ALIGN NUMBER EOL
        '''
        alignment = production[2]
        result, remainder = divmod(self.address, alignment)
        self.partial_word = False
        if remainder:
            self.address = (result + 1) * alignment

    def p_emitable_label(self, production):
        '''
        emitable : IDENTIFIER ':'
        '''
        name = production[1]
        if name in (AUTO_LABEL_FORWARD, AUTO_LABEL_BACKWARD):
            raise NameError('Reserved label name %r' % name)
        is_auto_label = name == AUTO_LABEL
        address = Address(self.address)
        if self.getIdentifier(name, address, acquire=not name.startswith('_')) != address:
            raise NameError('Redefining label %r' % name)
        if is_auto_label:
            # Resolve any forward reference.
            name = AUTO_LABEL_FORWARD
        self._resolveLabel(name, address.value)
        if is_auto_label:
            # Prepare for any back reference
            name = AUTO_LABEL_BACKWARD
        self.setIdentifier(name, address)
        self.partial_word = False

    def _resolveLabel(self, name, address):
        assert address & 0x3fff == address
        for referrer in self.label_referer_dict.pop(name, ()):
            self.rom[referrer] |= address

    def p_emitable_db(self, production):
        '''
        emitable : DB data_list EOL
        '''
        data_iterator = iter(production[2])
        if self.partial_word:
            data_a = self.unwrite()
            if not 0 <= data_a <= 0xff:
                raise ValueError('Value out of bounds: %x' % data_a)
            data_iterator = itertools.chain([data_a], data_iterator)
        while True:
            try:
                data_a = next(data_iterator)
            except StopIteration:
                break
            try:
                data_r = next(data_iterator)
            except StopIteration:
                self.write(data_a)
                self.partial_word = True
            else:
                self.write(data_a | (data_r << 8))

    def p_emitable_dw(self, production):
        '''
        emitable : DW data_list EOL
        '''
        for data in production[2]:
            self.write(data)

    @staticmethod
    def p_data_list_many(production):
        '''
        data_list : data_list ',' data_item
        '''
        production[0] = production[1] + production[3]

    @staticmethod
    def p_data_list_one(production):
        '''
        data_list : data_item
        '''
        production[0] = production[1]

    def p_data_item_identifier(self, production):
        '''
        data_item : IDENTIFIER
        '''
        name = production[1]
        production[0] = (
            self.getIdentifier(name, acquire=not name.startswith('_')).value,
        )

    @staticmethod
    def p_data_item_number(production):
        '''
        data_item : NUMBER
        '''
        production[0] = (production[1], )

    @staticmethod
    def p_data_item_string(production):
        '''
        data_item : STRING
        '''
        production[0] = tuple(ord(x) for x in production[1])

    def p_emitable_insn_0(self, production):
        '''
        emitable : INSTRUCTION EOL
        '''
        self.writeInstruction(production.lineno(1), production[1])

    def p_emitable_insn_1(self, production):
        '''
        emitable : INSTRUCTION operand EOL
        '''
        self.writeInstruction(production.lineno(1), production[1], production[2])

    def p_emitable_insn_2(self, production):
        '''
        emitable : INSTRUCTION operand ',' operand EOL
        '''
        self.writeInstruction(production.lineno(1), production[1], production[2], production[4])

    def p_operand_identifier(self, production):
        '''
        operand : IDENTIFIER
        '''
        identifier = production[1]
        try:
            production[0] = self.getIdentifier(identifier)
        except KeyError:
            exception = NameError(
                '%s:%i: Undefined identifier: %r' % (
                    self.filename,
                    production.lineno(1),
                    identifier,
                ),
                identifier,
            )
            if identifier == AUTO_LABEL_BACKWARD:
                # Backward references must always be immediately resolvable.
                # Otherwise a "@@" label is missing.
                raise exception from None
            production[0] = exception

    @staticmethod
    def p_operand_immediate_identifier(production):
        '''
        operand : '#' address
        '''
        address = production[2]
        if not isinstance(address, Address):
            raise TypeError('Wrong identifier type for immediate')
        production[0] = Immediate(address.value)

    def p_operand_relative_address(self, production):
        '''
        operand : RELATIVE_ADDRESS
        '''
        production[0] = Address(self.address + production[1])

    def p_error(self, production):
        raise SyntaxError("%s:%i: Syntax error, unexpected %s: %r" % (
            self.filename,
            production.lineno,
            production.type,
            production.value,
        ))

    def __init__(self, source_file, debug, include):
        self.debug = debug
        self.include = include
        self._parser_stack = []
        self.chip_identifier_dict = {
            'A': 'A', # Magic identifier for instruction matching
        }
        self.chip = None
        self.rom = {}
        self._parse(source_file)

    def _parse(self, source_file):
        parser = yacc(
            module=self,
            start='file',
            debug=self.debug,
            write_tables=False,
        )
        if self._parser_stack:
            if len(self._parser_stack) == 255:
                raise RuntimeError('maximum inclusion depth exceeded')
            parent_frame = self._parser_stack[-1]
            address = parent_frame.address
        else:
            parent_frame = None
            address = 0
        frame = ParserFrame(
            parser,
            source_file.name,
            address,
        )
        self._parser_stack.append(frame)
        try:
            if self.debug:
                print('Entering %r...' % source_file)
            parser.parse(
                source_file.read(),
                lexer=lex(
                    module=self,
                ),
                debug=self.debug,
            )
        finally:
            self._parser_stack.pop()
            if self.debug:
                print('Left %r' % source_file)
        if parent_frame is not None:
            parent_frame.address = frame.address_until_origin
            for key, value in frame.label_referer_dict.items():
                if key.startswith('_'):
                    continue
                parent_frame.label_referer_dict[key].extend(value)
            for key, value in frame.identifier_dict.items():
                if key.startswith('_'):
                    # Do not export symbols starting with an underscore.
                    continue
                if key in (
                    AUTO_LABEL,
                    AUTO_LABEL_FORWARD,
                    AUTO_LABEL_BACKWARD,
                ):
                    continue
                if parent_frame.identifier_dict.setdefault(
                    key,
                    value,
                ) != value:
                    raise NameError('%s: Redefining identifier %r' % (
                        frame.filename,
                        key,
                    ))
                if isinstance(value, Address):
                    self._resolveLabel(key, value.value)

    @property
    def label_referer_dict(self):
        try:
            return self._parser_stack[-1].label_referer_dict
        except IndexError:
            return None # Because of ply introspections

    @property
    def parser(self):
        try:
            return self._parser_stack[-1].parser
        except IndexError:
            return None # Because of ply introspections

    @property
    def filename(self):
        try:
            return self._parser_stack[-1].filename
        except IndexError:
            return None # Because of ply introspections

    @property
    def address(self):
        try:
            return self._parser_stack[-1].address
        except IndexError:
            return None # Because of ply introspections

    @address.setter
    def address(self, value):
        self._parser_stack[-1].setAddress(value)

    @property
    def partial_word(self):
        try:
            return self._parser_stack[-1].partial_word
        except IndexError:
            return None # Because of ply introspections

    @partial_word.setter
    def partial_word(self, value):
        self._parser_stack[-1].partial_word = value

    def getIdentifier(self, name, default=MARKER, acquire=True):
        try:
            return self.chip_identifier_dict[name]
        except KeyError:
            pass
        for frame in (
            reversed(self._parser_stack)
            if acquire else
            self._parser_stack[-1:]
        ):
            identifier_dict = frame.identifier_dict
            if name in identifier_dict:
                return identifier_dict.get(name, default)
        if default is MARKER:
            raise KeyError(name)
        return default

    def setIdentifier(self, name, value):
        self._parser_stack[-1].identifier_dict[name] = value

    def write(self, value):
        if not 0 <= value <= 0xffff:
            raise ValueError('Value out of bounds: %x' % value)
        if not 0 <= self.address <= 0x3fff:
            raise ValueError('Address out of bounds: %x' % value)
        if self.address in self.rom:
            raise ValueError('Redefining program address %x' % self.address)
        self.rom[self.address] = value
        self.address += 1
        self.partial_word = False

    def unwrite(self):
        self.address -= 1
        return self.rom.pop(self.address)

    def writeInstruction(self, lineno, name, left=NO_OPERAND, right=NO_OPERAND):
        try:
            opcode_list = INSTRUCTION_DICT[name]
        except KeyError:
            raise NameError('%s: %i: No such instruction: %r' % (
                self.filename,
                lineno,
                name,
            )) from None
        for opcode in opcode_list:
            mask, space, _, _, _, left_type, right_type = opcode_dict[opcode]
            if space is ROM_SPACE and isinstance(left, NameError):
                # Note: JUMP & CALL only accept one argument, ignore "right".
                # Mask sanity check
                assert mask == 0x3fff
                # Remember address for label...
                self.label_referer_dict[left.args[1]].append(self.address)
                # ... and use placeholder
                left = Address(0)
            if space in (RAM_SPACE, ZRO_SPACE):
                # pylint: disable=raising-non-exception
                if isinstance(left, NameError):
                    raise left
                if isinstance(right, NameError):
                    raise right
                # pylint: enable=raising-non-exception
            left_is_fixed = isinstance(left_type, str)
            right_is_fixed = isinstance(right_type, str)
            if (
                left_is_fixed and left == self.getIdentifier(left_type) or
                not left_is_fixed and isinstance(left, left_type)
            ) and (
                right_is_fixed and right == self.getIdentifier(right_type) or
                not right_is_fixed and isinstance(right, right_type)
            ):
                opcode <<= 8
                if not left_is_fixed and not isinstance(left, NoOperand):
                    operand_org = left
                elif not right_is_fixed and not isinstance(right, NoOperand):
                    operand_org = right
                else:
                    operand_org = None
                if operand_org is not None:
                    if isinstance(operand_org, BitAddress):
                        opcode |= operand_org.bit << 8
                    operand_org = operand_org.value
                    operand = operand_org & mask
                    if operand != operand_org:
                        raise ValueError(
                            '%s:%s: Operand too large for instruction %r: %x' % (
                                self.filename,
                                lineno,
                                name,
                                operand_org,
                            ),
                        )
                    opcode |= operand
                self.write(opcode)
                break
        else:
            raise NameError('%s:%i: No opcode suitable for %r %r, %r' % (
                self.filename,
                lineno,
                name,
                left,
                right,
            ))

def assemble(source, include=None, debug=False):
    """
    Assemble given source and return binary image.

    include (None, list of strings)
        List of paths to resolve included paths from.
        If None, inclusion is forbidden.
    """
    source_file = StringIO(source)
    source_file.name = 'noname.asm'
    getRomWord = Assembler(
        source_file=source_file,
        debug=debug,
        include=include,
    ).rom.get
    return b''.join(
        pack('<H', getRomWord(x, 0))
        for x in range(0x3000)
    )

def main():
    parser = argparse.ArgumentParser(description='SN8F2288 assembler')
    parser.add_argument(
        '-o',
        '--output',
        type=argparse.FileType('wb'),
        default=sys.stdout,
        help='Path to write binary image to, or - for stdout (default).',
    )
    parser.add_argument(
        '-I',
        '--include',
        default=[],
        nargs='+',
        action='append',
        help='Inclusion path(s)',
    )
    parser.add_argument(
        'input',
        type=argparse.FileType('r'),
        default=sys.stdin,
        help='Source to assemble. - for stdin (default).',
    )
    parser.add_argument(
        '--debug',
        action='store_true',
        help='Debug the assembler itself.',
    )
    args = parser.parse_args()
    with args.input as infile:
        assembler = Assembler(
            infile,
            debug=args.debug,
            include=sum(args.include, []),
        )
    with args.output as outfile:
        write = outfile.write
        for address in range(0x3000):
            write(pack('<H', assembler.rom.get(address, 0)))

if __name__ == '__main__':
    main()
