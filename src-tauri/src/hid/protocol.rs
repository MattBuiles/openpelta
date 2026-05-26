//! ROG Pelta USB HID protocol constants.
//!
//! Phase 0 + Phase 1 (opcode decode) complete by static analysis of the
//! Windows HAL `AacAudioHal_x64.dll` (radare2). Replay validation against a
//! live device is the next step — none of the opcodes below has been
//! confirmed visually yet.
//!
//! Internal ASUS codename for the Pelta family is `A501`. The Windows HAL
//! exposes two C++ classes: `C_A501_USB_Protocol` (wired) and
//! `C_A501_Protocol` (2.4 GHz and Bluetooth).

pub const ASUS_VID: u16 = 0x0b05;

/// Pelta wired (USB-C) PID.
pub const PELTA_WIRED_PID: u16 = 0x1b82;
/// Pelta wireless 2.4 GHz dongle PID.
pub const PELTA_WIRELESS_PID: u16 = 0x1b84;
/// Pelta Bluetooth PID (mono-LED variant).
pub const PELTA_BLUETOOTH_PID: u16 = 0x1b86;

/// HID interface number on the composite device exposing the vendor channels.
pub const HID_CONTROL_INTERFACE: u8 = 3;

/// Vendor command channel — UsagePage 0xFF00, Usage 0x0001.
/// Report 0xCC, 64-byte payload (1 report-ID byte + 63 data bytes? — no:
/// the descriptor declares 64-byte payload + 1 report-ID = 65 bytes on wire).
/// Request/response: write Output 0xCC; read Input 0xCC.
pub const REPORT_CMD: u8 = 0xcc;
pub const CMD_PAYLOAD_LEN: usize = 64;

/// Telemetry channel — usagePage 0xFF0B, usage 0x0104.
pub const REPORT_TELEMETRY_IDENTITY: u8 = 0x2a;

/// Wireless-only RF / dongle state channel — usagePage 0xFF07, usage 0x0212.
pub const REPORT_RF_IDENTITY: u8 = 0x3a;

// =====================================================================
//  Opcode catalog
// =====================================================================
//
// Wire layout for every operation:
//
//   wire[0]      = REPORT_CMD (0xCC, prepended by the HID writer)
//   wire[1..]    = 64-byte payload, zero-filled then patched as follows:
//     payload[0..N]   = opcode bytes (1, 2 or 4 bytes — see below)
//     payload[4..]    = parameters (operation-specific)
//
// Three command families share the channel, differentiated by the first
// opcode byte:
//
//   0x12 → "GET state"           (12 XX, where XX selects which state)
//   0x51 → "SET state"           (51 XX, write a single byte parameter at offset 4)
//   0x41 → "Noise Reduction"     (41 XX, audio-DSP subsystem)
//
// Response for GET ops comes back in an Input report 0xCC; the HAL caches
// fields at offsets inside its `this` object (noted per-operation below).

// ---------- GET (read state) ----------
// All single GETs send opcode bytes [0x12, sub], rest of payload zeros.

/// Firmware version — response is 8 bytes (4 HW + 4 FW), each pair formatted
/// as `%02X.%02X.%02X.%02X` by the HAL.
pub const OP_GET_FW_VERSION: [u8; 2]        = [0x12, 0x00];
/// Current LED effect mode + parameters.
pub const OP_GET_EFFECT_INFO: [u8; 2]       = [0x12, 0x03];
/// Battery level + power state.
pub const OP_GET_POWER_INFO: [u8; 2]        = [0x12, 0x07];
/// Charging yes/no (response at HAL state offset `[this+0xaf]`).
pub const OP_GET_CHARGING_STATE: [u8; 2]    = [0x12, 0x08];
/// Wireless power-saving mode (response at `[this+0xb8]`).
pub const OP_GET_POWER_SAVING_MODE: [u8; 2] = [0x12, 0x0e];
/// Master LED on/off (response at `[this+0xb6]`).
pub const OP_GET_LED_ON_OFF: [u8; 2]        = [0x12, 0x13];
/// Demo loop state (response at `[this+0xb4]`).
pub const OP_GET_DEMO_MODE: [u8; 2]         = [0x12, 0x18];
/// Sidetone volume level.
pub const OP_GET_SIDETONE_VOLUME: [u8; 2]   = [0x12, 0x19];
/// Sidetone enable.
pub const OP_GET_SIDETONE_ON_OFF: [u8; 2]   = [0x12, 0x24];
/// Voice-prompt locale.
pub const OP_GET_LANGUAGE: [u8; 2]          = [0x12, 0x28];
/// Wireless Down-Link status (response at `[this+0xa9]`).
pub const OP_GET_WDL_STATUS: [u8; 2]        = [0x12, 0x29];
/// WDL control-plane status.
pub const OP_GET_WDL_CONTROL_STATUS: [u8; 2] = [0x12, 0x33];
/// Wireless low-latency mode flag (response at `[this+0xba]`).
pub const OP_GET_LATENCY_MODE: [u8; 2]      = [0x12, 0x52];

/// Headset-exist query — uses a 4-byte opcode (the only GET that does).
/// Wireless only: the dongle reports whether the headset is paired and
/// in range. Response at `[this+0xad]`.
pub const OP_GET_HEADSET_EXIST: [u8; 4]     = [0x12, 0x00, 0x00, 0x01];

/// Noise-reduction state read (different family from the [0x12, …] block).
/// Response at `[this+0xd6]`.
pub const OP_GET_NR_ON_OFF: [u8; 2]         = [0x41, 0x20];

// ---------- SET (write state) ----------
// 0x51 family: opcode at bytes 0..2, single bool/byte parameter at byte 4.

/// Set RGB effect mode. Payload bytes 4..8 carry (mode, p1, p2, p3, p4) —
/// exact param semantics still need replay-mapping (e.g. which is speed,
/// which are R/G/B).
pub const OP_SET_LIGHT_EFFECT: [u8; 2]      = [0x51, 0x28];
/// Direct (software-mode) RGB color. Payload bytes 4..7 = R, G, B.
/// Opcode is 4 bytes because bytes 2..4 are reserved (zero-filled).
pub const OP_SET_SW_LED_COLOR: [u8; 4]      = [0x51, 0x30, 0x00, 0x00];
/// Demo loop on/off. Byte 4 = 0 or 1.
pub const OP_SET_DEMO_MODE_ON_OFF: [u8; 2]  = [0x51, 0x31];
/// Enable pairing (wireless). Byte 4 = 0 or 1.
pub const OP_SET_DEVICE_WDL_ENABLE: [u8; 2] = [0x51, 0x33];
/// Low-latency mode on/off (wireless). Byte 4 = 0 or 1.
pub const OP_SET_LATENCY_MODE: [u8; 2]      = [0x51, 0x52];

/// Software-mode on/off (wireless only). Same opcode bytes as
/// SET_DEVICE_WDL_ENABLE, but the wrapper routes it through a different HID
/// writer (`fcn.18002b7b0` in the HAL, vs. the standard `fcn.18002b920`).
/// This likely targets a different report ID or interface — possibly the
/// wireless-only RF state collection (0xFF07). Treat as a separate channel.
pub const OP_SET_SW_MODE_ON_OFF: [u8; 2]    = [0x51, 0x33];

/// Noise reduction on/off. Byte 4 = 0 or 1.
pub const OP_SET_NR_ON_OFF: [u8; 2]         = [0x41, 0x02];

// ---------- Operations intentionally absent ----------
//
//   - EQ / equalizer       → handled by ASUS `AudioSDK` (USB Audio Class
//                             extensions or software DSP). Not reachable
//                             via Report 0xCC.
//   - Sleep timer          → Pelta does not expose this. Other ASUS headsets
//                             (RH200WLE, RH300WL) do.
//   - Mic mute (vendor)    → hardware/firmware-controlled; reported via the
//                             standard Telephony usage page on a separate
//                             HID collection.
//
// Legacy aliases — the existing pelta.rs scaffolding still uses these names.
// Every command flows through REPORT_CMD; only the payload opcodes differ.
pub const REPORT_BATTERY: u8 = REPORT_CMD;
pub const REPORT_SIDETONE: u8 = REPORT_CMD;
pub const REPORT_FIRMWARE: u8 = REPORT_CMD;
pub const REPORT_RGB: u8 = REPORT_CMD;
pub const REPORT_EQ: u8 = REPORT_CMD;          // EQ is via AudioSDK, not 0xCC
pub const REPORT_MIC_MUTE: u8 = REPORT_CMD;    // mic mute is standard Telephony
pub const REPORT_SLEEP_TIMER: u8 = REPORT_CMD; // Pelta does not expose this

/// Build a 65-byte wire frame: `[0xCC, opcode_bytes..., 0x00 padding]`.
/// `params` (if any) are placed starting at payload byte 4 (i.e. wire byte 5).
/// The total wire length is always 65 bytes.
pub fn build_frame(opcode: &[u8], params: &[u8]) -> [u8; 1 + CMD_PAYLOAD_LEN] {
    let mut frame = [0u8; 1 + CMD_PAYLOAD_LEN];
    frame[0] = REPORT_CMD;
    // opcode goes at wire bytes [1..1+opcode.len()]
    frame[1..1 + opcode.len()].copy_from_slice(opcode);
    // params go at wire byte 5 onward (= payload byte 4)
    if !params.is_empty() {
        let start = 1 + 4;
        let end = start + params.len();
        debug_assert!(end <= frame.len(), "params overflow wire frame");
        frame[start..end].copy_from_slice(params);
    }
    frame
}
