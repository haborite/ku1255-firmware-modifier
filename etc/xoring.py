import csv

# 入力ファイルと出力ファイルのパス
input_file = "keymaps.csv"
output_file = "keymaps_new.csv"

# CSVを読み込み、XOR処理を行い、新しいCSVとして保存
def process_csv(input_path, output_path):
    with open(input_path, newline='', encoding='utf-8') as infile:
        reader = csv.reader(infile)
        header = next(reader)  # ヘッダーを取得
        data = list(reader)  # データをリストとして取得
    
    # XORed_ID の列インデックスを取得
    xor_index = header.index("XORed_ID")
    id_index = header.index("ID")
    
    for row in data:
        try:
            if row[id_index]:  # ID が空でない場合
                id_value = int(row[id_index], 16)  # 16進数として変換
                row[xor_index] = f"{id_value ^ 0x5A:X}"  # XOR計算し16進表記
        except ValueError:
            pass  # IDが適切でない場合はスキップ
    
    # 新しいCSVを保存
    with open(output_path, "w", newline='', encoding='utf-8') as outfile:
        writer = csv.writer(outfile)
        writer.writerow(header)  # ヘッダーを書き込む
        writer.writerows(data)  # データを書き込む

# 実行
process_csv(input_file, output_file)