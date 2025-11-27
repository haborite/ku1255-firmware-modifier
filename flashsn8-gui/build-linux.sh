#!/bin/bash
set -e

# Get version as the first argument or default to 0.0.1
VERSION=${1:-0.0.1}

# Nuitka compile
python3 -m nuitka --standalone --enable-plugin=pyside6 \
  --user-package-configuration-file=config.yaml \
  --output-dir=linux \
  flashsn8-gui.py

# Remove old directory if exists
cd linux
rm -rf flashsn8-gui-linux-"$VERSION"
mv flashsn8-gui.dist flashsn8-gui-linux-"$VERSION"

# Create zip archive
cd ..
mkdir -p upload
zip -r "upload/flashsn8-gui-linux-$VERSION.zip" "linux/flashsn8-gui-linux-$VERSION"
