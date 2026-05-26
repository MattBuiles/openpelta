# ROG Pelta — USB HID Protocol

Reverse-engineered protocol notes. **Work in progress.** Contributions welcome via PR.

## Device identification

| Variant | VID | PID | Source |
|---------|-----|-----|--------|
| Wireless dongle | `0x0b05` | `TBD` | `lsusb -d 0b05:` |
| Wired USB-C | `0x0b05` | `TBD` | `lsusb -d 0b05:` |

To fill: run `scripts/probe-hid.sh` on Linux or USB Device Tree Viewer on Windows.

## HID interfaces

| Interface | Usage page | Usage | Purpose |
|-----------|-----------|-------|---------|
| 0 | TBD | TBD | Audio class (standard) |
| 1 | TBD | TBD | Vendor control (commands live here) |
| 2 | TBD | TBD | Telemetry / events |

## Report map

Each row: capture a single isolated action, diff bytes, record here.

### Output reports (host → device)

| Action | Report ID | Payload bytes | Notes |
|--------|-----------|---------------|-------|
| RGB Static | `0x??` | `[mode, R, G, B, speed, ...]` | TBD |
| RGB Breathing | `0x??` | `[mode, R, G, B, speed, ...]` | TBD |
| RGB Off | `0x??` | `[0, 0, 0, 0, 0, ...]` | TBD |
| EQ apply | `0x??` | `[band0_gain, ..., band9_gain]` | Range? |
| Sidetone | `0x??` | `[level]` | Range 0-100? |
| Mic mute | `0x??` | `[1 or 0]` | TBD |
| Sleep timer | `0x??` | `[mins_lo, mins_hi]` | TBD |
| Firmware query | `0x??` | `[query_op]` | TBD |

### Input reports (device → host)

| Event | Report ID | Payload bytes | Notes |
|-------|-----------|---------------|-------|
| Battery level | `0x??` | `[percent]` | TBD |
| Button (mic mute) | `0x??` | `[state]` | TBD |
| Charging state | `0x??` | TBD | TBD |

### Feature reports (bidirectional)

| Feature | Report ID | Length | Notes |
|---------|-----------|--------|-------|
| Firmware version | `0x??` | TBD | TBD |
| Device info | `0x??` | TBD | TBD |

## Methodology

1. **Find VID/PID** — `scripts/probe-hid.sh` (Linux) or USB Device Tree Viewer (Windows).
2. **Capture baseline** — record `idle_30s.pcap` with nothing happening.
3. **Capture per-action** — for each control, capture isolated single change.
4. **Diff** — open in Wireshark, filter by `usb.src` and `usb.dst` matching Pelta address. Compare per-action vs baseline.
5. **Identify report ID** — first byte of HID output is the report ID.
6. **Document** — fill table above with `[report_id]` and payload pattern.
7. **Replay** — `hidapitester --vidpid 0b05/PID --send-output 0x...` and confirm device reacts.

## Reference: other reversing projects

- [OpenRGB ASUS Aura USB protocols](https://gitlab.com/CalcProgrammer1/OpenRGB/-/tree/master/Controllers/AsusAuraUSBController)
- [HeadsetControl](https://github.com/Sapd/HeadsetControl) — multi-vendor headset HID reference
- [asusctl](https://gitlab.com/asus-linux/asusctl) — ASUS laptop HID patterns
