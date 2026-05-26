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

## Report map (Phase 1 — TBD)

For each operation above, fill in the 64-byte template once decoded:

```
Byte 0    : 0xCC                (report ID)
Byte 1    : <opcode>            (TBD per operation)
Byte 2..N : <parameters>        (TBD)
Byte ..63 : 0x00 padding
```

### Output reports (host → device)

| Action                  | Opcode byte | Params       | Notes        |
|-------------------------|:-----------:|--------------|--------------|
| Set RGB effect mode     | `0x??`      | mode index, speed, …  | from `setLightEffect` |
| Set static color (SW)   | `0x??`      | R, G, B               | from `setSWLEDColor`  |
| Toggle SW direct mode   | `0x??`      | bool                  | wireless only |
| Set noise reduction     | `0x??`      | bool                  | from `setNROnOff`     |
| Set demo mode           | `0x??`      | bool                  | from `setDemoModeOnOff` |
| Set latency mode        | `0x??`      | bool                  | wireless only |
| Enable pairing          | `0x??`      | bool                  | wireless only |
| Query FW version        | `0x??`      | —                     | request, response in Input 0xCC |
| Query power info        | `0x??`      | —                     | request, response in Input 0xCC |
| Query effect info       | `0x??`      | —                     | request, response in Input 0xCC |

### Input reports (device → host)

Same Report ID `0xCC`. Returned in response to a GET request — no spontaneous emission observed.

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
