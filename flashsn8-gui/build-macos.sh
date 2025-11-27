#!/bin/bash
set -e

# Get version as the first argument or default to 0.0.1
VERSION=${1:-0.0.1}

# Nuitka compile
python3 -m nuitka --standalone --enable-plugin=pyside6 \
  --user-package-configuration-file=config.yaml \
  --output-dir=macos \
  flashsn8-gui.py

# Remove old directory if exists
cd macos
rm -rf flashsn8-gui-macos-"$VERSION"
mv flashsn8-gui.dist flashsn8-gui-macos-"$VERSION"

# Create zip archive
cd ..
mkdir -p upload
zip -r "upload/flashsn8-gui-macos-$VERSION.zip" "macos/flashsn8-gui-macos-$VERSION"
