import csv

def load_hid_data(hid_file):
    hid_data = {}
    with open(hid_file, newline='', encoding='utf-8') as infile:
        reader = csv.DictReader(infile)
        for row in reader:
            hid_data[int(row['ID'], 16)] = {'Byte_Offset': row['Byte_Offset'], 'XORed_ID': row['XORed_ID']}
    return hid_data

def process_csv(input_file, hid_file, output_file):
    hid_data = load_hid_data(hid_file)
    
    with open(input_file, newline='', encoding='utf-8') as infile, open(output_file, 'w', newline='', encoding='utf-8') as outfile:
        reader = csv.DictReader(infile)
        fieldnames = ['ID', 'Byte_Offset', 'XORed_ID', 'Usage Name', '0B47190 (US)', '0B47208 (JIS)', 'ISO']
        writer = csv.DictWriter(outfile, fieldnames=fieldnames)
        
        writer.writeheader()
        for row in reader:
            nid = int(row['ID'], 16)
            new_row = {
                'ID': '{:02X}'.format(nid),
                'Byte_Offset': hid_data.get(nid, {}).get('Byte_Offset', '')[2:],
                'XORed_ID': hid_data.get(nid, {}).get('XORed_ID', '')[2:],
                'Usage Name': row['Usage Name'],
                '0B47190 (US)': row.get('0B47190 (US)'),
                '0B47208 (JIS)': row.get('0B47208 (JIS)'),
                'ISO': row.get('ISO', ''),
            }
            writer.writerow(new_row)

# 使用例
input_csv = 'base-merged.csv'  # 入力CSVファイル名
hid_csv = 'keymaps3.csv'  # HID情報が含まれるCSVファイル名
output_csv = 'output.csv'  # 出力CSVファイル名
process_csv(input_csv, hid_csv, output_csv)
