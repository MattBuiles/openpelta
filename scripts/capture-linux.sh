#!/usr/bin/env bash
# OpenPelta — Phase 0 capture helper (Linux)
# Requires: tshark, usbmon kernel module, root.
# Used for blind probing on Linux (no Armoury Crate available).

set -euo pipefail

OUTDIR="${OUTDIR:-$(dirname "$0")/../captures}"
mkdir -p "$OUTDIR"

if [[ $EUID -ne 0 ]]; then
    echo "Run as root (USB capture needs CAP_NET_ADMIN)." >&2
    exit 1
fi

modprobe usbmon || true

echo "=== USB devices (ASUS VID 0b05) ==="
lsusb -d 0b05: || true

read -rp "Enter usbmon interface (e.g. usbmon1): " IFACE
read -rp "Optional label: " LABEL
LABEL="${LABEL:-capture}"

OUTFILE="$OUTDIR/${LABEL}_$(date +%Y%m%d_%H%M%S).pcap"
echo "Capturing on $IFACE → $OUTFILE (Ctrl+C to stop)"

tshark -i "$IFACE" -w "$OUTFILE"
