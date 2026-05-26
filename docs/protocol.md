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
wire[1..2] : <opcode>            (1 or 2 bytes — operation identifier)
wire[3..]  : <parameters>        (operation-specific)
wire[..64] : 0x00 padding
```

### Decoded commands

#### `getFWVersion`  ✅ decoded, not yet replay-validated

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

### Pending decode (Phase 1 continuation)

| Action               | Opcode | Params               | Notes                          |
|----------------------|:------:|----------------------|--------------------------------|
| Set RGB effect mode  | `0x??` | mode, speed, R, G, B | from `setLightEffect`          |
| Toggle SW direct mode| `0x??` | bool                 | wireless only                  |
| Set noise reduction  | `0x??` | bool                 | from `setNROnOff`              |
| Set demo mode        | `0x??` | bool                 | from `setDemoModeOnOff`        |
| Set latency mode     | `0x??` | bool                 | wireless only                  |
| Enable pairing       | `0x??` | bool                 | wireless only                  |
| Query power info     | `0x??` | —                    | battery % + power state        |
| Query effect info    | `0x??` | —                    | current RGB effect             |
| Query charging state | `0x??` | —                    | wired present / charging       |
| Query headset exist  | `0x??` | —                    | wireless only                  |
| Query WDL status     | `0x??` | —                    | wireless link state            |

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
