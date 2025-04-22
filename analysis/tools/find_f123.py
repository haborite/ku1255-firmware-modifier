def find_patterns_in_binary(filepath, start_addr, pattern1, pattern2, output_file1, output_file2):
    """
    バイナリファイルから指定されたパターンを検索し、アドレスを抽出する関数。

    Args:
        filepath (str): バイナリファイルのパス。
        start_addr (int): 検索を開始するアドレス。
        pattern1 (bytes): 検索するパターン1（例: b'\x25\xDA\xDB'）。
        pattern2 (bytes): 検索するパターン2（例: b'\x25\x5A\xDA\x5A\xDB'）。
        output_file1 (str): パターン1が見つかったアドレスを保存するファイルパス。
        output_file2 (str): パターン2が見つかったアドレスを保存するファイルパス。
    """

    with open(filepath, 'rb') as f:
        f.seek(start_addr)
        binary_data = f.read()

    pattern1_addresses = []
    pattern2_addresses = []

    for i in range(len(binary_data) - len(pattern1) + 1):
        if binary_data[i:i + len(pattern1)] == pattern1:
            pattern1_addresses.append(start_addr + i)

    for i in range(len(binary_data) - len(pattern2) + 1):
        if binary_data[i:i + len(pattern2)] == pattern2:
            pattern2_addresses.append(start_addr + i)

    with open(output_file1, 'w') as f:
        for addr in pattern1_addresses:
            f.write(f'{addr:X}\n')

    with open(output_file2, 'w') as f:
        for addr in pattern2_addresses:
            f.write(f'{addr:X}\n')


# 使用例
filepath = 'output.sn8'  # 検索対象のバイナリファイル
start_addr = 0x00000  # 検索開始アドレス
pattern1 = b'\xAF\x00'  # 検索パターン1
pattern2 = b'\x25\x5A\xDB\x5A\xDA'  # 検索パターン2
output_file1 = 'pattern1_addresses.txt'  # パターン1のアドレスを保存するファイル
output_file2 = 'pattern2_addresses.txt'  # パターン2のアドレスを保存するファイル

find_patterns_in_binary(filepath, start_addr, pattern1, pattern2, output_file1, output_file2)