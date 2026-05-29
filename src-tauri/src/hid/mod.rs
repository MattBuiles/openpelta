pub mod protocol;
pub mod pelta;

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub use pelta::Pelta;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const OFF: Rgb = Rgb { r: 0, g: 0, b: 0 };
}

/// LED behavior modes.
///
/// `Static` is driven through the dedicated direct-color command
/// (`setSWLEDColor`). The animated modes go through `setLightEffect`, whose
/// device-side mode bytes were confirmed visually (cyan probe, 2026-05-28):
///   1 = static, 2 = breathing, 3 = strobe, 4 = rainbow.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RgbMode {
    Off,
    Static,
    Breathing,
    Strobe,
    Rainbow,
}

/// Power/connection snapshot returned by [`PeltaDevice::power_info`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PowerInfo {
    /// Best-guess battery percentage — second data byte of the getPowerInfo
    /// response (observed `0x52` = 82, a plausible level). Still to be
    /// confirmed against a real charge/discharge cycle.
    pub percent: u8,
    pub charging: bool,
    /// The four data bytes of the getPowerInfo response, exposed for debugging
    /// the still-uncertain battery encoding.
    pub raw: [u8; 4],
}

/// The vendor control surface of the ROG Pelta, as reverse-engineered in
/// `docs/protocol.md`. Every method maps to a validated Report 0xCC command.
///
/// Notably absent (and intentionally so):
///   - EQ: lives in the system audio backend (`crate::audio`), not on the device.
///   - Sleep timer: the Pelta firmware does not expose one.
///   - Software mic mute: handled at the OS level / hardware button, not via a
///     vendor command.
///   - Sidetone setters: only getters were found in the HAL; the setter path
///     (likely via the generic `setCmd` wrapper) is not decoded yet.
pub trait PeltaDevice: Send + Sync {
    // --- reads ---
    fn firmware_version(&self) -> Result<String>;
    fn power_info(&self) -> Result<PowerInfo>;
    /// Wireless-only: whether the dongle reports the headset paired & in range.
    fn headset_present(&self) -> Result<bool>;
    fn led_enabled(&self) -> Result<bool>;
    fn noise_reduction(&self) -> Result<bool>;
    /// Latency preset in ms (one of 40/60/80/100).
    fn latency_mode(&self) -> Result<u8>;
    fn sidetone_volume(&self) -> Result<u8>;
    fn sidetone_enabled(&self) -> Result<bool>;

    // --- writes ---
    /// Direct static color (software mode). Validated visually.
    fn set_led_color(&self, color: Rgb) -> Result<()>;
    /// Animated effect. `mode` must be one of `protocol::LIGHT_EFFECT_MODES`
    /// (1..4); `intensity` and `color` fill the remaining payload bytes.
    fn set_light_effect(&self, mode: u8, intensity: u8, color: Rgb) -> Result<()>;
    fn set_noise_reduction(&self, on: bool) -> Result<()>;
    /// Latency preset. `value_ms` must be one of
    /// `protocol::LATENCY_MODE_VALUES_MS` (40/60/80/100); other values are
    /// silently rejected by the device.
    fn set_latency_mode(&self, value_ms: u8) -> Result<()>;
    fn set_demo_mode(&self, on: bool) -> Result<()>;

    /// High-level RGB dispatch used by the profile system. Mode-byte mapping
    /// matches what the device firmware actually does (live-confirmed):
    /// breathing=2, strobe=3, rainbow=4.
    fn set_rgb(&self, mode: RgbMode, color: Rgb, intensity: u8) -> Result<()> {
        match mode {
            RgbMode::Off => self.set_led_color(Rgb::OFF),
            RgbMode::Static => self.set_led_color(color),
            RgbMode::Breathing => self.set_light_effect(2, intensity, color),
            RgbMode::Strobe => self.set_light_effect(3, intensity, color),
            RgbMode::Rainbow => self.set_light_effect(4, intensity, color),
        }
    }
}
