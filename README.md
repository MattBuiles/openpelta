# OpenPelta

Open-source replacement for **ASUS Armoury Crate Gear** targeting the **ROG Pelta** headset.
Cross-platform (Windows, Linux). Built with Tauri + Rust + Svelte.

> Status: **pre-alpha** — protocol reversing in progress. See [docs/protocol.md](docs/protocol.md).

## Why

Armoury Crate Gear is heavy, unstable, and bloated. The Pelta deserves better:

- EQ + presets without 1 GB of ASUS bloat
- RGB control that doesn't fight with Aura
- Battery, sidetone, mic-mute, sleep timer
- Works on Linux (Armoury Crate doesn't)

## Features (planned v1)

- [ ] Parametric EQ + FPS / Music / Cinema / Voice presets
- [ ] Virtual surround (HeSuVi on Win, EasyEffects convolver on Linux)
- [ ] RGB modes (Static, Breathing, Wave, Rainbow)
- [ ] Battery / sidetone / mic-mute / sleep timer
- [ ] Profiles + system tray + global hotkeys (`Ctrl+Alt+1..9`)
- [ ] Firmware update (opt-in, behind feature flag — high brick risk)

## Status

Phase 0 (USB HID reversing) is the blocker. Without it, the device control layer is stubs.

If you can run Wireshark + USBPcap on Windows with Armoury Crate installed, please:

1. Run `scripts/capture-windows.ps1` as Administrator.
2. Open an Issue with the resulting `.pcap` files attached (or `tshark -r file.pcap -V > dump.txt`).

## Build

Requires: Rust stable, Node 20+, pnpm 9+. Linux also: `webkit2gtk-4.1`, `librsvg2-dev`, `libudev-dev`.

```bash
pnpm install
pnpm tauri dev      # dev
pnpm tauri build    # release
```

## Architecture

```
Frontend (Svelte + Tauri webview)
        ▼ IPC
Backend (Rust)
  ├─ hid/        — hidapi-rs, raw protocol
  ├─ audio/      — Equalizer APO (Win) / EasyEffects (Linux) config writer
  ├─ rgb/        — effect state machine
  ├─ profiles/   — TOML store at ~/.config/openpelta/profiles.toml
  ├─ firmware/   — OTA (feature-gated)
  ├─ tray.rs     — system tray
  └─ hotkeys.rs  — global-hotkey
```

## License

GPL-3.0 — same spirit as OpenRGB. Forks must publish reversing work back to the community.

## Credits

Inspired by [OpenRGB](https://gitlab.com/CalcProgrammer1/OpenRGB), [HeadsetControl](https://github.com/Sapd/HeadsetControl), [asusctl](https://gitlab.com/asus-linux/asusctl), [G-Helper](https://github.com/seerge/g-helper).
