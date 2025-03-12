import csv

def main():
    OUTPUT_PATH = './'
    DATA_FILE = OUTPUT_PATH + 'keymaps.csv'
    OUTPUT_FILE = OUTPUT_PATH + 'keymaps.md'
    output_list = []
    item_count = 0

    with open(DATA_FILE, 'r', encoding='utf-8') as f:
        reader = csv.reader(f)

        for column, line in enumerate(reader):
            if column == 0:
                item_count = len(line)
                s = '|'
                for index in range(item_count):
                    line[index] = line[index].replace('\n', '<br>')
                    line[index] = line[index].replace('-', 'ー')
                    s = s + f' {line[index]} |'
                s = s + '\n'
                output_list.append(s)

                s = '|'
                for header_item in line:
                    s = s + ' --- |'
                s = s + '\n'
                output_list.append(s)
            else:
                s = '|'
                for index in range(item_count):
                    line[index] = line[index].replace('\n', '<br>')
                    line[index] = line[index].replace('-', 'ー')
                    s = s + f' {line[index]} |'
                s = s + '\n'
                output_list.append(s)

    with open(OUTPUT_FILE, 'w', encoding='utf-8') as f2:
        for line in output_list:
            f2.write(line)

if __name__ == "__main__":
  main()