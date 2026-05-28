# OpenPelta — Backlog

What's still to build before OpenPelta covers every reachable feature of
the ROG Pelta. Items below the "in scope" line are sequenced roughly in
the order we plan to tackle them.

## Tier 1 — Quick wins (current focus)

| # | Feature | Notes |
|---|---|---|
| 1.1 | **Mic mute (software)** | Vendor HID has no mic-mute setter, but Windows Core Audio's `IAudioEndpointVolume::SetMute` on the Pelta capture endpoint does the job. Exposes as a toggle + tray entry + global hotkey. |
| 1.2 | **EQ on/off toggle** | Bypass without resetting bands — write an empty `openpelta.txt` (or comment out the `Include:` line) while keeping the gains in localStorage for the next enable. |
| 1.3 | **Surround sound 7.1 (HeSuVi)** | `AudioConfig.surround_enabled` is already plumbed through. APO supports HeSuVi-format convolution; need to bundle (or download) a default HRIR and add a UI toggle. |
| 1.4 | **Multiple profiles** | `ProfileStore` is half-written. Wire UI: profile picker in header, "Save as…", "Delete", per-profile rgb/eq/nr/latency. |
| 1.5 | **Tray quick-toggles** | Add Mic Mute, NR, and active-profile menu items to the tray icon menu so the user doesn't have to open the window. |
| 1.6 | **Low-battery notification** | Poll `getPowerInfo` on a slow timer (1–5 min). When `percent` crosses a threshold and we're not charging, fire a `tauri-plugin-notification`. |

## Tier 2 — Medium effort

| # | Feature | Notes |
|---|---|---|
| 2.1 | **Sidetone level setter (UAC)** | Vendor HID exposes only getters. The setter lives on the USB Audio Class interface — needs IKsControl / IDeviceTopology on the Pelta capture-side device. ~2 h spike. |
| 2.2 | **Global hotkeys wired** | `hotkeys.rs` already registers `Ctrl+Alt+1..9` + `Ctrl+Alt+M`. Need an event channel into the main app to act on them (switch profile, toggle mic mute). |
| 2.3 | **Per-app auto-profile switching** | Foreground-window watcher → match `process_name` against rules in each profile → apply on focus change. Windows: `GetForegroundWindow` + `GetWindowThreadProcessId`. |
| 2.4 | **OpenPelta auto-update** | Tauri's built-in updater. Needs release signing + a hosted `latest.json`. |
| 2.5 | **Battery % calibration** | Verify byte 5 (`0x52 = 82`) really tracks battery percentage by reading over a charge/discharge cycle. Adjust `power_info` parsing if a different byte tracks better. |
| 2.6 | **Light-effect mode labels** | Modes 1–4 are labelled provisionally (Breathing/Wave/Rainbow + one unused). Observe the LED for each mode and rename. |
| 2.7 | **Probe `setSWModeOnOff`** | The one vendor command that goes through a different HID writer (`fcn.18002b7b0`). Likely targets the wireless RF state collection (0xFF07). Worth one focused replay session. |

## Out of scope (intentional)

- **Firmware OTA updates** — brick risk; OTA protocol not decoded; ASUS provides its own updater.
- **Active noise cancellation** — Pelta does not have ANC, only passive isolation.
- **Voice-prompt customization** — device exposes `getLanguage` only, no setter.

## Reference

- Reverse-engineered protocol: [`docs/protocol.md`](docs/protocol.md)
- Device-side HAL: `C:\Program Files\ASUS\AacHeadSet\AacAudioHal_x64.dll`
- Codename inside ASUS: `A501`
