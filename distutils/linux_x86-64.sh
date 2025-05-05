#!/bin/bash
set -e

# === Settings ===
WORKDIR="deploy/linux_working"
DISTDIR="deploy/linux"
ARCHIVE_NAME="ku1255-firmware-modifier-linux.zip"
FLASH_GUI_ZIP="deploy/flashsn8-gui-linux.zip"
FLASH_GUI_EXTRACTED="deploy/flashsn8-gui-linux"
FLASH_GUI_DEST="$WORKDIR/firmware/flashsn8"
FLASH_GUI_URL="https://github.com/haborite/flashsn8-gui/releases/download/0.0.1/flashsn8-gui-linux-0.0.1.zip"

# === 1. Compile TailwindCSS ===
echo "=== 1. Compiling TailwindCSS ==="
npx tailwindcss -i ./input.css -o ./assets/tailwind.css --minify

# === 2. Build Dioxus App ===
echo "=== 2. Building Dioxus App ==="
dx build --release --platform desktop

# === 3. Prepare Output Directories ===
echo "=== 3. Preparing output directories ==="
rm -rf "$WORKDIR" "$DISTDIR"
mkdir -p "$WORKDIR" "$DISTDIR"

# === 4. Download flashsn8-gui (cached if exists) ===
echo "=== 4. Downloading flashsn8-gui (cached if exists) ==="
if [ ! -f "$FLASH_GUI_ZIP" ]; then
    wget -O "$FLASH_GUI_ZIP" "$FLASH_GUI_URL"
else
    echo "Using cached: $FLASH_GUI_ZIP"
fi

# === 5. Extract flashsn8-gui ===
echo "=== 5. Extracting flashsn8-gui ==="
rm -rf "$FLASH_GUI_EXTRACTED"
unzip -q "$FLASH_GUI_ZIP" -d "$FLASH_GUI_EXTRACTED"

# === 6. Copy flashsn8-gui contents into firmware/flashsn8 ===
echo "=== 6. Copying flashsn8-gui contents ==="
rm -rf "$FLASH_GUI_DEST"
mkdir -p "$FLASH_GUI_DEST"
cp -r "$FLASH_GUI_EXTRACTED"/linux/flashsn8-gui-linux-0.0.1/* "$FLASH_GUI_DEST/"
rm -rf "$FLASH_GUI_EXTRACTED"

# === 7. Copy project directories ===
echo "=== 7. Copying project resources ==="
for dir in boards examples logical_layouts settings; do
    cp -r "$dir" "$WORKDIR/"
done

# === 8. Copy built binary and assets ===
echo "=== 8. Copying built artifacts ==="
cp "dist/ku1255-firmware-modifier" "$WORKDIR/"
cp -r "dist/assets" "$WORKDIR/assets"

# === 9. Create ZIP archive ===
echo "=== 9. Creating ZIP archive ==="
cd "$WORKDIR"
zip -r "../linux/$ARCHIVE_NAME" ./*
cd - > /dev/null

echo "=== Build complete ==="
