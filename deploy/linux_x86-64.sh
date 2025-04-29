#!/bin/bash
set -e

# === 1. Tailwind CSS コンパイル ===
echo "Compiling TailwindCSS..."
npm install
npx tailwindcss -i ./input.css -o ./assets/tailwind.css --minify

# === 2. Dioxus デスクトップアプリのビルド ===
echo "Building Dioxus app for desktop..."
dx build --release --platform desktop

# === 3. 必要なファイルとフォルダをコピー ===
echo "Collecting files..."
OUTDIR=deploy/linux_working
DISTDIR=deploy/linux

rm -rf "$OUTDIR"
mkdir -p "$OUTDIR"
rm -rf "$DISTDIR"
mkdir -p "$DISTDIR"

# コピー対象フォルダ
for FOLDER in boards examples firmware logical_layouts settings; do
    cp -r "$FOLDER" "$OUTDIR/"
done

# バイナリのコピー（DioxusのLinuxビルド成果物）
cp target/dx/ku1255-firmware-modifier/release/linux/app/ku1255-firmware-modifier "$OUTDIR/"
cp -r target/dx/ku1255-firmware-modifier/release/linux/app/assets "$OUTDIR/"

# === flashsn8.dist の取得と配置 ===
echo "Downloading flashsn8.dist..."
wget https://github.com/haborite/dissn8/releases/download/v0.0.1/linux_x86-64.tar.gz
tar -zxvf linux_x86-64.tar.gz
cp flashsn8.dist "$OUTDIR/firmware/"
rm linux_x86-64.tar.gz

# === 4. ZIP圧縮 ===
echo "Creating ZIP archive..."
(cd "$OUTDIR" && zip -r ../linux/ku1255-firmware-modifier-linux.zip .)

echo "Done."
