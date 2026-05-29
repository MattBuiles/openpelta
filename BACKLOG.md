# OpenPelta — Backlog

What's still to build before OpenPelta covers every reachable feature of
the ROG Pelta. Items below the "in scope" line are sequenced roughly in
the order we plan to tackle them.

## Tier 1 — Quick wins

| # | Feature | Status |
|---|---|---|
| 1.1 | **Mic mute (software)** | ✅ Done — Windows Core Audio in `win_audio.rs`, exposed in UI + tray + hotkey |
| 1.2 | **EQ on/off toggle** | ✅ Done — `AudioConfig.enabled` + APO renders no-op when off |
| 1.3 | **Surround sound 7.1 (HeSuVi)** | ⏳ Not started — needs bundled / downloaded HRIR for APO |
| 1.4 | **Multiple profiles** | ✅ Done — picker bar with save / overwrite / delete, apply pushes everything |
| 1.5 | **Tray quick-toggles** | ✅ Done — Show / mic mute / NR / Quit |
| 1.6 | **Low-battery notification** | ✅ Done — 5-min poll, Notification API, hysteresis on recovery |

## Tier 2 — Medium effort

| # | Feature | Status |
|---|---|---|
| 2.1 | **Sidetone level setter (UAC)** | ⏳ Not started — needs IKsControl / IDeviceTopology spike |
| 2.2 | **Global hotkeys wired** | ✅ Done — Ctrl+Alt+M (mic mute) handled in backend; Ctrl+Alt+1..9 emit `hotkey:profile` events the frontend applies |
| 2.3 | **Per-app auto-profile switching** | ✅ Done — Windows foreground-window watcher emits `foreground:changed`; each profile carries an `app_matches` list, frontend auto-applies the first matching profile on focus change |
| 2.4 | **OpenPelta auto-update** | ⏳ Not started — needs release signing + hosted `latest.json` |
| 2.5 | **Battery % calibration** | ⏳ Needs a charge / discharge cycle from the user to confirm byte mapping |
| 2.6 | **Light-effect mode labels** | ✅ Done — confirmed visually (2026-05-28): mode 1 = static, 2 = breathing, 3 = strobe, 4 = rainbow. RgbMode + dispatch + UI updated |
| 2.7 | **Probe `setSWModeOnOff`** | 🟡 Wire format pinned: send `[0x51, 0x33, 0x00, 0x00, <on>]` as Output report `0x3A` on the RF collection (0xFF07, wireless). Device accepts both values without error; user-visible semantic still TBD (does not appear to be required for the direct-color path we already use). |

## Out of scope (intentional)

- **Firmware OTA updates** — brick risk; OTA protocol not decoded; ASUS provides its own updater.
- **Active noise cancellation** — Pelta does not have ANC, only passive isolation.
- **Voice-prompt customization** — device exposes `getLanguage` only, no setter.

## Reference

- Reverse-engineered protocol: [`docs/protocol.md`](docs/protocol.md)
- Device-side HAL: `C:\Program Files\ASUS\AacHeadSet\AacAudioHal_x64.dll`
- Codename inside ASUS: `A501`
