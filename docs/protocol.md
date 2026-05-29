# ROG Pelta — USB HID Protocol

Reverse-engineered protocol notes. **Phase 0 complete** (device + transport + operation inventory). **Phase 1 in progress** (per-command byte payloads).

> Internal ASUS codename: **A501** — found in firmware updater (`A501_FWUpdater_Headset.exe`) and Windows HAL classes (`C_A501_Protocol`, `C_A501_USB_Protocol`).

## Device identification

| Variant       | VID      | PID      | HAL class                | LED        | Notes                                  |
|---------------|----------|----------|--------------------------|------------|----------------------------------------|
| Wired USB-C   | `0x0b05` | `0x1b82` | `C_A501_USB_Protocol`    | RGB        | confirmed                              |
| 2.4 GHz       | `0x0b05` | `0x1b84` | `C_A501_Protocol`        | RGB        | confirmed, dongle as separate composite |
| Bluetooth     | `0x0b05` | `0x1b86` | `C_A501_Protocol` (mono) | mono-color | confirmed via ASUS config JSON; not yet enumerated live |

Source: Windows `Get-PnpDevice` + `hidapitester --list-detail` + `C:\ProgramData\ASUS\ROG Live Service\DeviceDependentVersion\ROG PELTA\ROG PELTA.json`.

## USB composite device layout

Each variant exposes (at minimum):

| Interface | Class               | Purpose                                  |
|-----------|---------------------|------------------------------------------|
| `MI_00`   | USB Audio Class     | Headset audio streaming                  |
| `MI_03`   | HID                 | Control plane (multiple top-level collections) |

## HID collections on `MI_03`

| Collection   | UsagePage | Usage    | Wired | 2.4 GHz | Purpose                                   |
|--------------|-----------|----------|:-----:|:-------:|-------------------------------------------|
| Consumer     | `0x000C`  | `0x0001` |  ✅   |   ✅    | Standard volume/play/mute keys            |
| Telephony    | `0x000B`  | `0x0005` |  —    |   ✅    | Standard headset off-hook / mic mute      |
| **Command**  | `0xFF00`  | `0x0001` |  ✅   |   ✅    | **Vendor command channel — Report `0xCC`** |
| **RF state** | `0xFF07`  | `0x0212` |  —    |   ✅    | Wireless link / dongle telemetry          |
| Telemetry    | `0xFF0B`  | `0x0104` |  ✅   |   ✅    | Generic device telemetry / identity       |

The command channel descriptors are **byte-identical** between wired and 2.4 GHz → same protocol on both.

### Command channel (Report `0xCC`)

```
Report ID:  0xCC
Direction:  Output (host→device)  and  Input (device→host)
Payload:    63 data bytes (64 incl. report ID)
Transport:  HID interrupt I/O on MI_03 / Col02 (0xFF00, 0x0001)
Semantics:  request / response. No spontaneous emission observed during 3s idle.
```

Report-ID-0xCC raw report descriptor (identical wired / wireless):

```
06 00 FF 09 01 A1 00 85 CC 09 01 15 00 26 FF 00
75 08 95 3F 81 02 09 01 15 00 26 FF 00 75 08 95
3F 91 02 C0
```

## Operation inventory

The Windows HAL (`AacAudioHal_x64.dll`) implements two C++ classes for Pelta. Method names give a near-complete picture of what the vendor protocol supports.

### `GET` operations (read state)

| Operation             | Wired | Wireless | Description                                  |
|-----------------------|:-----:|:--------:|----------------------------------------------|
| `getFWVersion`        |  ✅   |    ✅    | Firmware version                             |
| `getPowerInfo`        |  ✅   |    ✅    | Battery % + power state                      |
| `getChargingState`    |  ✅   |    ✅    | Charging yes/no                              |
| `getHeadsetExist`     |   —   |    ✅    | Dongle reports whether headset is in range  |
| `getWDLStatus`        |  ✅   |    ✅    | Wireless Down-Link status                    |
| `getWDLControlStatus` |  ✅   |    ✅    | WDL control plane status                     |
| `getLanguage`         |  ✅   |    ✅    | Voice-prompt locale                          |
| `getLatencyMode`      |   —   |    ✅    | Low-latency mode flag                        |
| `getPowerSavingMode`  |   —   |    ✅    | Wireless sleep/PS state                      |
| `getSidetoneOnOff`    |  ✅   |    ✅    | Sidetone enable                              |
| `getSidetoneVolume`   |  ✅   |    ✅    | Sidetone level                               |
| `getNROnOff`          |  ✅   |    ✅    | Mic AI noise reduction enable                |
| `getLEDOnOff`         |  ✅   |    ✅    | Master LED enable                            |
| `getEffectInfo`       |  ✅   |    ✅    | Current RGB effect + parameters              |
| `getDemoMode`         |  ✅   |    ✅    | Demo loop state                              |

### `SET` operations (write state)

| Operation             | Wired | Wireless | Description                                  |
|-----------------------|:-----:|:--------:|----------------------------------------------|
| `setLightEffect`      |  ✅   |    ✅    | RGB effect mode (Static / Breathing / …)     |
| `setSWLEDColor`       |  ✅   |    ✅    | Direct color in software mode (R/G/B)        |
| `setSWModeOnOff`      |   —   |    ✅    | Toggle direct-control mode                   |
| `setNROnOff`          |  ✅   |    ✅    | Noise reduction on/off                       |
| `setDemoModeOnOff`    |  ✅   |    ✅    | Demo loop on/off                             |
| `setLatencyMode`      |   —   |    ✅    | Low-latency on/off                           |
| `setDeviceWDLEnable`  |   —   |    ✅    | Pairing on/off                               |
| `setCmd`              |  ✅   |    ✅    | Generic command wrapper (likely covers more) |

### Operations **not** in the vendor HID protocol

| Function          | Where it actually lives                                  |
|-------------------|----------------------------------------------------------|
| EQ / equalizer    | ASUS `AudioSDK` module — USB Audio Class extensions or software DSP. Not reachable via Report `0xCC`. |
| Sleep timer       | Pelta does not expose this. Other ASUS headsets (RH200WLE, RH300WL) do. |
| Mic mute (vendor) | Hardware/firmware-controlled. Reported to host via standard Telephony usage page on a separate HID collection — no vendor SET command. |
| Setters for sidetone level | No dedicated `setSidetoneOnOff` / `setSidetoneVolume` symbol found. Likely routed through the generic `setCmd` wrapper with a sub-opcode (TBD). |

## Report map

Each operation puts an opcode + parameters into the 64-byte payload. The HID
writer prepends the report ID `0xCC`, giving 65 bytes on the wire:

```
wire[0]    : 0xCC                (report ID)
wire[1..]  : 64-byte payload, zero-filled then patched as follows:
  payload[0..N] : opcode bytes (1, 2, 4 — see table)
  payload[4..]  : parameters (operation-specific)
```

Three opcode families share the channel, differentiated by the first byte:

| Family | First byte | Meaning                          |
|--------|:----------:|----------------------------------|
| GET    | `0x12`     | Read device state                |
| SET    | `0x51`     | Write device state               |
| NR     | `0x41`     | Noise-reduction (audio DSP)      |

### Complete opcode table (Phase 1 — decoded from `AacAudioHal_x64.dll`)

All 23 operations exposed by `C_A501_Protocol` / `C_A501_USB_Protocol` have
been decoded. **None has been replay-validated against the live device yet.**

#### GET operations (response in Input report `0xCC`)

| Operation              | Payload[0..N]       | Cached at (`[this+offset]`) |
|------------------------|---------------------|-----------------------------|
| `getFWVersion`         | `12 00`             | `0xc3..0xca` (8 bytes: HW + FW versions) |
| `getEffectInfo`        | `12 03`             | (current LED mode + params) |
| `getPowerInfo`         | `12 07`             | (battery % + power state)   |
| `getChargingState`     | `12 08`             | `0xaf`                      |
| `getPowerSavingMode`   | `12 0e`             | `0xb8`                      |
| `getLEDOnOff`          | `12 13`             | `0xb6`                      |
| `getDemoMode`          | `12 18`             | `0xb4`                      |
| `getSidetoneVolume`    | `12 19`             | (TBD)                       |
| `getSidetoneOnOff`     | `12 24`             | (TBD)                       |
| `getLanguage`          | `12 28`             | (TBD)                       |
| `getWDLStatus`         | `12 29`             | `0xa9`                      |
| `getWDLControlStatus`  | `12 33`             | (TBD)                       |
| `getLatencyMode`       | `12 52`             | `0xba`                      |
| `getHeadsetExist` (wl) | `12 00 00 01` (4 B!)| `0xad`                      |
| `getNROnOff`           | `41 20`             | `0xd6`                      |

#### SET operations (response: status byte in Input `0xCC`)

| Operation              | Payload[0..N]       | Params (payload[4..])        |
|------------------------|---------------------|------------------------------|
| `setLightEffect`       | `51 28`             | 5 bytes — mode + 4 params (semantics TBD: speed, R, G, B?) |
| `setSWLEDColor`        | `51 30 00 00`       | `R G B` at bytes 4..6       |
| `setDemoModeOnOff`     | `51 31`             | bool at byte 4               |
| `setDeviceWDLEnable` (wl) | `51 33`          | bool at byte 4 (pairing on/off) |
| `setLatencyMode` (wl)  | `51 52`             | bool at byte 4               |
| `setSWModeOnOff` (wl)  | `51 33` ⚠️          | bool at byte 4 — uses a **different writer** (`fcn.18002b7b0`) than all other SETs; likely targets a separate report ID or interface (possibly RF state collection 0xFF07). Distinct from `setDeviceWDLEnable` despite identical opcode bytes. |
| `setNROnOff`           | `41 02`             | bool at byte 4               |
| `setCmd`               | (passthrough)       | caller provides opcode + params verbatim |

#### Worked example: `setSWLEDColor(R, G, B)`

```
wire: CC 51 30 00 00 RR GG BB 00 00 00 ... (zero-padded to 65 bytes)
       │  ╰──opcode────╯ │  │  │
       │  byte 0..3      │  │  └── B  (payload[6])
       │                 │  └───── G  (payload[5])
       │                 └──────── R  (payload[4])
       └── Report ID
```

The opcode `0x51` matches the `AURA_DIRECT_RGB` pattern documented for other
ASUS Aura USB devices in OpenRGB.

#### Worked example: `getFWVersion`

```
wire request : CC 12 00 00 ... (zero-padded to 65 bytes)
```

Response arrives as Input report `0xCC`; the HAL extracts 8 version bytes
and formats them as two `%02X.%02X.%02X.%02X` strings (HW and FW).

## Live validation

All commands below were exercised against a real wireless Pelta (PID 0x1b84)
using `hidapitester` against UsagePage 0xFF00 / Usage 0x0001. No errors on
write, response bytes echo the opcode and carry meaningful state.

### `setSWLEDColor` — visually confirmed ✅

Sent in sequence with 3 s between commands; LED tracked exactly:

| Command (R, G, B) | Wire bytes (first 8)             | Observed LED |
|-------------------|----------------------------------|--------------|
| `(0xFF, 0, 0)`    | `CC 51 30 00 00 FF 00 00`        | red          |
| `(0, 0xFF, 0)`    | `CC 51 30 00 00 00 FF 00`        | green        |
| `(0, 0, 0xFF)`    | `CC 51 30 00 00 00 00 FF`        | blue         |
| `(0xFF, 0xFF, 0xFF)` | `CC 51 30 00 00 FF FF FF`     | white        |

→ Confirms opcode `0x51 0x30 0x00 0x00`, R/G/B at payload bytes 4/5/6, and
that the wireless device honors software-mode color directly (no prior
`setSWModeOnOff` was needed in this test).

### GET responses — observed values (single sample, wireless idle, FW 03.00.04.00)

Each response is the Input `0xCC` immediately following the corresponding
Output `0xCC`. Bytes shown are the first useful bytes of the 64-byte payload;
remainder is zero-padded.

| Op                     | Echoed opcode | Data bytes      | Interpretation (best guess)             |
|------------------------|---------------|-----------------|------------------------------------------|
| `getFWVersion`         | `12 00`       | `03 00 04 00 03 00 04 00` | 8 version bytes (4 HW + 4 FW). Exact field anchor within the payload still to be pinned against the HAL's parse offset; the data region begins at payload byte 4 like every other GET. |
| `getEffectInfo`        | `12 03`       | `04 32 FF 00`   | mode=4, param=0x32 (50%?), then `FF 00` |
| `getPowerInfo`         | `12 07`       | `05 52 14 01`   | first byte may be battery state code, then 3 more state bytes |
| `getChargingState`     | `12 08`       | `00`            | not charging                              |
| `getPowerSavingMode`   | `12 0e`       | `00`            | disabled                                  |
| `getLEDOnOff`          | `12 13`       | `01`            | LED master enabled                        |
| `getDemoMode`          | `12 18`       | `00`            | disabled                                  |
| `getSidetoneVolume`    | `12 19`       | `0A`            | level = 10 (range likely 0..100)          |
| `getSidetoneOnOff`     | `12 24`       | `00`            | disabled                                  |
| `getLanguage`          | `12 28`       | `01`            | locale id = 1                             |
| `getWDLStatus`         | `12 29`       | `00`            | link OK / nominal                         |
| `getWDLControlStatus`  | `12 33`       | `00`            |                                           |
| `getLatencyMode`       | `12 52`       | `64`            | = 100 — NOT a bool; appears to be a level or threshold (range 0..100?) |
| `getNROnOff`           | `41 20`       | `00`            | NR disabled                               |

Notes:
- `getLatencyMode` returning `0x64` is **not** a boolean. Follow-up probe
  (see below) shows it accepts only four discrete byte values — likely a
  millisecond preset.
- `getHeadsetExist` was confirmed in a follow-up: with the dongle paired,
  the 4-byte opcode `12 00 00 01` returns `0x01` at payload byte 5.

### Live SET round-trips and range probes

| Operation             | Round-trip result                                          |
|-----------------------|------------------------------------------------------------|
| `setDemoModeOnOff`    | SET 1 → GET `01`, SET 0 → GET `00` ✅ boolean              |
| `setNROnOff`          | SET 1 → GET `01`, SET 0 → GET `00` ✅ boolean              |
| **`setLatencyMode`**  | Accepts only **`0x28 (40), 0x3C (60), 0x50 (80), 0x64 (100)`** — any other byte is silently rejected (GET returns the previously accepted value). Almost certainly four latency presets in milliseconds (Ultra-low / Low / Normal / Power-save). |
| **`setLightEffect`**  | Accepted modes: **`0x01, 0x02, 0x03, 0x04`**. Mode `0x00` clears params to zero (LED goes black, internal mode tag stays at last value); modes `0x05+` are silently rejected. Payload bytes 4..8 = (mode, intensity 0x32 nominal, R, G, B). |
| `setSWLEDColor`       | Visually confirmed in all four primaries (red / green / blue / white). Works without `setSWModeOnOff` on the wireless variant — direct color mode is the default. |

### Not yet probed (require user gating or destructive action)

| Operation           | Why deferred                                              |
|---------------------|-----------------------------------------------------------|
| Sidetone **setter** | The HAL exposes only `getSidetoneVolume` / `getSidetoneOnOff`, no setter. A live opcode sweep around the `0x12`/`0x51`/`0x41` families (`51 19`, `51 24`, `41 19`, `41 18`, `41 24`, `51 18`, `51 1A`) failed to move the value — it stays at the read-back `0x0A`. Strong evidence the sidetone level is a **USB Audio Class feature-unit control**, not a vendor HID command (consistent with the HAL only reading it for display). Implementing it needs the Windows Core Audio / Kernel Streaming property path, not Report 0xCC. |
| `setSWModeOnOff`    | Uses a different HID writer (`fcn.18002b7b0`); likely targets a different report ID or the RF state collection (0xFF07). Needs separate replay with the right channel. |
| `setDeviceWDLEnable`| Toggling pairing live would disconnect the headset. Defer until we have a recover-by-cable plan. |
| Battery calibration | Requires running the headset down on battery and comparing `getPowerInfo` reads against an external % display over hours. The response `05 52 14 01` has a plausible `0x52 = 82` second byte that may be the real percentage — needs confirming. |
| Bluetooth variant   | PID `0x1b86` is documented in the ASUS config but the device has not been enumerated live in BT mode. |

### Replay procedure (Windows)

```pwsh
# any GET
.\hidapitester.exe `
    --vidpid 0B05/1B84 --usagePage 0xFF00 --usage 0x0001 `
    -l 64 -t 1500 --open `
    --send-output 0xCC,<opcode byte 0>,<opcode byte 1> `
    --read-input

# any SET (single-byte param)
.\hidapitester.exe `
    --vidpid 0B05/1B84 --usagePage 0xFF00 --usage 0x0001 `
    -l 64 --open `
    --send-output 0xCC,<opcode byte 0>,<opcode byte 1>,0,0,<param>
```

Source: `fcn.18002e540` in HAL DLL (xref from `mutex_getFWVersion` log strings).

```
wire: CC 12 00 00 00 ... (63 zeros)
       ^^ ^^^^^
       │  └── opcode 0x0012 (LE) → bytes 0x12, 0x00
       └── report ID
```

Response (Input 0xCC, 65 bytes on wire) carries 8 version bytes: 4 HW + 4 FW,
each formatted as `%02X.%02X.%02X.%02X`. Offsets in the response payload are
internal to the HAL's state object (cached at `[this+0xc3..0xca]`); the wire
layout still needs to be cross-checked against a live response capture.

#### `setSWLEDColor(r, g, b)`  ✅ decoded, not yet replay-validated

Source: `fcn.180031300` in HAL DLL (xref from `mutex_setSWLEDColor` log strings).
Sets the headset LED to a direct color in software mode. Has no effect unless
the device is currently in software/direct mode (use `setSWModeOnOff` first on
wireless; wired may default to direct mode).

```
wire: CC 51 30 00 00 RR GG BB 00 ... (56 zeros)
       ^^ ^^^^^^^^^^^^^ ^^ ^^ ^^
       │  └── opcode    │  │  │
       │      0x51 0x30 │  │  └── blue
       │      00 00     │  └───── green
       │                └──────── red
       └── report ID
```

The opcode `0x51` matches the `AURA_DIRECT_RGB` pattern documented for other
ASUS Aura USB devices (see OpenRGB), and `0x30` is plausibly the headset-LED
sub-channel selector.

### Input reports (device → host)

Same Report ID `0xCC`. Returned only in response to a GET request — no
spontaneous emission was observed in a 3-second idle listen on the command
collection.

## Methodology (Phase 1)

The Windows HAL `AacAudioHal_x64.dll` contains compiled C++ implementations of
`C_A501_Protocol` and `C_A501_USB_Protocol`. Symbols are stripped, but each
method has its own set of UTF-16 log strings in `.rdata` that uniquely identify
it (e.g. `"C_A501_Protocol::mutex_setSWLEDColor() : Mutex Error \n"`).

Per-command extraction flow:

1. `rabin2 -z` on the DLL — list all log strings with virtual addresses.
2. Pick a target operation (e.g. `mutex_setSWLEDColor`). Note any one of its
   log-string addresses, e.g. `0x180114160`.
3. In radare2: `aac; axt 0x180114160` — finds the function that emits that
   string. That is the `mutex_*` wrapper.
4. The wrapper either (a) calls an inner implementation (e.g. `getFWVersion`
   wraps `fcn.18002e540`) or (b) builds the buffer and calls the HID writer
   `fcn.18002b920` directly.
5. Find the basic block immediately before `call fcn.18002b920`. It will:
   - Zero a 64-byte buffer with four `movups xmm0, ...` instructions, then
   - Write the opcode with `mov word ptr [buf], <imm>`, and
   - Write each parameter byte with `mov byte ptr [buf+N], <reg>`.
6. Record the opcode bytes and the offset / source register for each parameter.
7. The HID writer adds the report ID at byte 0 (passed as `byte [this+0x48]`,
   confirmed to be `0xCC` for Pelta from the report descriptor).

Replay validation (next step) uses `hidapitester`:

```pwsh
.\hidapitester.exe --vidpid 0B05/1B84 --usagePage 0xFF00 --usage 0x0001 `
    -l 64 --open --send-output 0xCC,0x51,0x30,0,0,0xFF,0x00,0x00  # red
```

— and visually confirms the headset LED changes.

## Methodology

Phase 0 used **static reverse-engineering** of the Windows HAL DLL rather than packet capture, because the Armoury Crate UI is unreliable on this hardware (it can launch but crashes when navigating to the Pelta device page on some setups), which makes timed Wireshark captures fragile.

Tools used:

- `Get-PnpDevice` (Windows built-in) — enumerate USB devices
- [`hidapitester`](https://github.com/todbot/hidapitester) — dump HID report descriptors and probe reports
- [`radare2`](https://github.com/radareorg/radare2) (`rabin2 -z`) — extract strings and class symbols from `AacAudioHal_x64.dll`

Phase 1 will combine:

1. Function-by-function disassembly of each `C_A501_Protocol::mutex_*` method, locating the byte template it writes to Report `0xCC`.
2. Replay validation via `hidapitester --send-output 0xCC,...` while observing the headset (LED color change, audible sidetone, battery readback matching reality).

## Reference: other reversing projects

- [OpenRGB ASUS Aura USB protocols](https://gitlab.com/CalcProgrammer1/OpenRGB/-/tree/master/Controllers/AsusAuraUSBController)
- [HeadsetControl](https://github.com/Sapd/HeadsetControl) — multi-vendor headset HID reference
- [asusctl](https://gitlab.com/asus-linux/asusctl) — ASUS laptop HID patterns
