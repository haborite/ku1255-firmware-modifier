#!/bin/sh
set -eux

# Get version as the argument
VERSION=${1:-0.4.0}  # デフォルトで0.4.0を使用。引数が渡された場合はそれを使用。

# nuitka compile
python -m nuitka --standalone --enable-plugin=pyside6 --windows-console-mode=disable --output-dir=linux ku1255-fw-remapper.py

cd linux
mv ku1255-fw-remapper.dist "ku1255-fw-remapper-linux-${VERSION}"
cp ../keymaps.csv "ku1255-fw-remapper-linux-${VERSION}/keymaps.csv"
cp -r ../examples "ku1255-fw-remapper-linux-${VERSION}/examples"

# Create tar.gz archive
tar -zcvf "../upload/ku1255-fw-remapper-linux-${VERSION}.tar.gz" "ku1255-fw-remapper-linux-${VERSION}"
