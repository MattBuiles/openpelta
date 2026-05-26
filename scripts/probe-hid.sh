#!/usr/bin/env bash
# Blind HID probing — read descriptors without Armoury Crate.
# Requires: hidapitester (https://github.com/todbot/hidapitester) or lsusb -v.

set -euo pipefail

VID="0b05"

echo "=== ASUS HID devices ==="
lsusb -d "${VID}:" || true

if command -v hidapitester &>/dev/null; then
    echo ""
    echo "=== hidapitester list ==="
    hidapitester --vidpid "${VID}/" --list || true
    hidapitester --vidpid "${VID}/" --list-detail || true
else
    echo "Install hidapitester for richer output: https://github.com/todbot/hidapitester"
fi

echo ""
echo "=== Report descriptors (requires root) ==="
for dev in /sys/class/hidraw/hidraw*; do
    name="$(basename "$dev")"
    uevent="$(cat /sys/class/hidraw/$name/device/uevent 2>/dev/null || true)"
    if echo "$uevent" | grep -qi "0B05"; then
        echo "--- $name ---"
        echo "$uevent"
        sudo cat "/sys/class/hidraw/$name/device/report_descriptor" | xxd
        echo ""
    fi
done
