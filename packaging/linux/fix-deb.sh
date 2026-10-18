#!/usr/bin/env bash
# Repack a dx-built .deb: root ownership, no stray /.dx-default-icon.png.
set -euo pipefail
deb="$1"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
dpkg-deb -R "$deb" "$work/root"
rm -f "$work/root/.dx-default-icon.png"
if [ -f "$work/root/DEBIAN/md5sums" ]; then
  sed -i '/ \.dx-default-icon\.png$/d' "$work/root/DEBIAN/md5sums"
fi
dpkg-deb --root-owner-group --build "$work/root" "$deb"
