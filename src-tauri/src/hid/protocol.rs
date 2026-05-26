//! ROG Pelta USB HID protocol constants.
//!
//! Phase 0 reversing status: device identification + operation inventory complete.
//! Specific command byte payloads are still TBD (Phase 1) — see `docs/protocol.md`.
//!
//! Internal ASUS codename for the Pelta family is `A501`. The Windows HAL exposes
//! two C++ classes for it: `C_A501_USB_Protocol` (wired) and `C_A501_Protocol`
//! (2.4 GHz and Bluetooth).

pub const ASUS_VID: u16 = 0x0b05;

/// Pelta wired (USB-C) PID.
/// Confirmed via `Get-PnpDevice` + hidapitester on Windows.
pub const PELTA_WIRED_PID: u16 = 0x1b82;

/// Pelta wireless 2.4 GHz dongle PID.
/// Confirmed via `Get-PnpDevice` + hidapitester on Windows.
pub const PELTA_WIRELESS_PID: u16 = 0x1b84;

/// Pelta Bluetooth PID.
/// Confirmed via ASUS device-config JSON (`ROG PELTA.json` regkey `0B051B86`).
/// Note: BT variant has a single mono LED, not RGB.
pub const PELTA_BLUETOOTH_PID: u16 = 0x1b86;

/// HID interface number on the composite device exposing the vendor command channel.
/// Same on all three variants.
pub const HID_CONTROL_INTERFACE: u8 = 3;

/// Vendor command channel (HID Output / Input report).
/// UsagePage 0xFF00, Usage 0x0001.
/// 64-byte payload (1 report-ID byte + 63 data bytes).
/// Request/response: write Output 0xCC; read Input 0xCC.
pub const REPORT_CMD: u8 = 0xcc;
pub const CMD_PAYLOAD_LEN: usize = 64;

/// Telemetry channel — usagePage 0xFF0B, usage 0x0104.
/// Reports 0x2A (Feature, 60 B identity-style payload) and 0x2B/0x2C/0x2D (Input).
pub const REPORT_TELEMETRY_IDENTITY: u8 = 0x2a;

/// Wireless-only RF / dongle state channel — usagePage 0xFF07, usage 0x0212.
/// Reports 0x3A (Feature identity) and 0x3C/0x3D (uint32 quads — RF link / state).
pub const REPORT_RF_IDENTITY: u8 = 0x3a;

// ---------------------------------------------------------------------------
// Operation opcodes (Phase 1 — TBD).
//
// Inventory of operations exposed by the vendor HID protocol (extracted from
// `AacAudioHal_x64.dll`):
//
//   GET:  FWVersion, PowerInfo, ChargingState, HeadsetExist (wl),
//         WDLStatus, WDLControlStatus, Language,
//         LatencyMode (wl), PowerSavingMode (wl),
//         SidetoneOnOff, SidetoneVolume, NROnOff,
//         LEDOnOff, EffectInfo, DemoMode
//
//   SET:  LightEffect, SWLEDColor, SWModeOnOff (wl),
//         NROnOff, DemoModeOnOff,
//         LatencyMode (wl), DeviceWDLEnable (wl, pairing),
//         Cmd (generic wrapper)
//
// All flow through Report 0xCC. The first payload bytes encode the opcode;
// remaining bytes carry parameters. Reverse-engineering these byte patterns
// is the Phase 1 deliverable.
//
// NOT in the vendor HID protocol:
//   - EQ / equalizer  → handled by ASUS AudioSDK (USB Audio Class extensions or
//                        software DSP); not reachable via Report 0xCC.
//   - Sleep timer     → Pelta does not expose this (other ASUS headsets do).
//   - Mic mute        → hardware/firmware-controlled; reported to host via the
//                        standard Telephony usage page on a separate collection,
//                        not via the vendor channel.
// ---------------------------------------------------------------------------

// Backwards-compat aliases used by the existing pelta.rs scaffolding.
// All vendor operations flow through Report 0xCC; the opcodes that differentiate
// them live in the payload bytes (Phase 1). For now, all aliases point to
// REPORT_CMD and the methods that use them will return garbage until the byte
// patterns are decoded.
pub const REPORT_BATTERY: u8 = REPORT_CMD;
pub const REPORT_SIDETONE: u8 = REPORT_CMD;
pub const REPORT_FIRMWARE: u8 = REPORT_CMD;
pub const REPORT_RGB: u8 = REPORT_CMD;

// These have no vendor-HID equivalent on the Pelta — kept as REPORT_CMD only so
// the placeholder scaffolding compiles. Real implementations will live elsewhere
// (AudioSDK / standard Telephony page) or be removed.
pub const REPORT_EQ: u8 = REPORT_CMD;          // EQ is via AudioSDK, not HID 0xCC
pub const REPORT_MIC_MUTE: u8 = REPORT_CMD;    // mic mute is standard Telephony page
pub const REPORT_SLEEP_TIMER: u8 = REPORT_CMD; // Pelta does not expose this

