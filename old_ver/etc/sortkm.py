import csv

def sort_csv(input_file1, input_file2, id_output_file, offset_output_file):
    """
    CSVファイルを読み込み、HID_Key_IDとByte_Offsetでソートして新しいCSVファイルに保存する。

    Args:
        input_file (str): 入力CSVファイルのパス。
        id_output_file (str): HID_Key_IDでソートしたCSVファイルの出力パス。
        offset_output_file (str): Byte_OffsetでソートしたCSVファイルの出力パス。
    """

    data = []
    with open(input_file1, 'r', newline='') as csvfile:
        reader = csv.DictReader(csvfile)
        for row in reader:
            data.append(row)
    with open(input_file2, 'r', newline='') as csvfile:
        reader = csv.DictReader(csvfile)
        for row in reader:
            row['Byte_Offset'] ="0x00"
            data.append(row)

    # HID_Key_IDでソート
    id_sorted_data = sorted(data, key=lambda x: int(x['HID_Key_ID'], 16))

    # Byte_Offsetでソート
    offset_sorted_data = sorted(data, key=lambda x: int(x['Byte_Offset'], 16))

    # HID_Key_IDでソートしたデータをCSVファイルに書き込み
    with open(id_output_file, 'w', newline='') as csvfile:
        fieldnames = ['HID_Key_Name', 'HID_Key_ID', 'Byte_Offset', 'XORed_ID']
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(id_sorted_data)

    # Byte_OffsetでソートしたデータをCSVファイルに書き込み
    with open(offset_output_file, 'w', newline='') as csvfile:
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(offset_sorted_data)

# CSVファイルのパスを指定して関数を実行
input_file1 = 'keymaps.csv'  # 入力CSVファイルのパス
input_file2 = 'keymaps2.csv'  # 入力CSVファイルのパス
id_output_file = 'id-sorted.csv'  # HID_Key_IDでソートしたCSVファイルの出力パス
offset_output_file = 'offset-sorted.csv'  # Byte_OffsetでソートしたCSVファイルの出力パス

sort_csv(input_file1, input_file2, id_output_file, offset_output_file)