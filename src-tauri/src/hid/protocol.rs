//! ROG Pelta USB HID protocol constants.
//!
//! Values here are PLACEHOLDERS until Fase 0 reversing completes.
//! Replace with real captures from `docs/protocol.md`.

pub const ASUS_VID: u16 = 0x0b05;

/// Pelta wireless dongle PID. Confirm with `lsusb` / USB Device Tree Viewer.
pub const PELTA_WIRELESS_PID: u16 = 0x0000; // TODO

/// Pelta wired PID.
pub const PELTA_WIRED_PID: u16 = 0x0000; // TODO

/// HID report IDs (placeholders).
pub const REPORT_BATTERY: u8 = 0x00;
pub const REPORT_SIDETONE: u8 = 0x00;
pub const REPORT_MIC_MUTE: u8 = 0x00;
pub const REPORT_SLEEP_TIMER: u8 = 0x00;
pub const REPORT_FIRMWARE: u8 = 0x00;
pub const REPORT_RGB: u8 = 0x00;
pub const REPORT_EQ: u8 = 0x00;
