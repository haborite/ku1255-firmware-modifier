#!/bin/sh
set -e
python -m nuitka --standalone --enable-plugin=pyside6 --windows-console-mode=disable --output-dir=linux ku1255-fw-remapper.py
cd linux
mv ku1255-fw-remapper.dist ku1255-fw-remapper-linux-0.4.0
cp ../keymaps.csv ku1255-fw-remapper-linux-0.4.0/keymaps.csv
cp -r ../examples ku1255-fw-remapper-linux-0.4.0/examples
tar -zcvf ../upload/ku1255-fw-remapper-linux-0.4.0.tar.gz ku1255-fw-remapper-0.4.0