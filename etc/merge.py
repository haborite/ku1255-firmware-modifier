import csv

def merge_values(val1, val2):
    if val1 and val2:
        return f"{val1} and {val2}"
    return val1 or val2 or ''

def process_csv(input_file, output_file):
    with open(input_file, newline='', encoding='utf-8') as infile, open(output_file, 'w', newline='', encoding='utf-8') as outfile:
        reader = csv.DictReader(infile)
        fieldnames = ['ID', 'Usage Name', '0B47190 (US)', '0B47208 (JIS)', 'ISO']
        writer = csv.DictWriter(outfile, fieldnames=fieldnames)
        
        writer.writeheader()
        for row in reader:
            print(row)
            new_row = {
                'ID': row['ID'],
                'Usage Name': row['Usage Name'],
                '0B47190 (US)': merge_values(row.get('0B47190 (US)', ''), row.get('ANSI shift', '')),
                '0B47208 (JIS)': merge_values(row.get('0B47208 (JIS)', ''), row.get('JIS shift', '')),
                'ISO': merge_values(row.get('ISO', ''), row.get('ISO shift', '')),
            }
            writer.writerow(new_row)

# 使用例
input_csv = 'ase.csv'  # 入力CSVファイル名
output_csv = 'base-merged.csv'  # 出力CSVファイル名
process_csv(input_csv, output_csv)
