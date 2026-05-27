use anyhow::{anyhow, Result};
use hidapi::{HidApi, HidDevice};
use std::sync::Mutex;

use super::protocol::*;
use super::{PeltaDevice, PowerInfo, Rgb};

pub struct Pelta {
    device: Mutex<HidDevice>,
}

impl Pelta {
    /// Open the first connected Pelta (wired or 2.4 GHz), selecting the vendor
    /// command collection (UsagePage 0xFF00).
    pub fn open() -> Result<Self> {
        let api = HidApi::new()?;
        let info = api
            .device_list()
            .find(|d| {
                d.vendor_id() == ASUS_VID
                    && matches!(d.product_id(), PELTA_WIRED_PID | PELTA_WIRELESS_PID)
                    && d.usage_page() == 0xff00
            })
            .ok_or_else(|| anyhow!("Pelta vendor interface not found"))?;
        let device = info.open_device(&api)?;
        Ok(Self {
            device: Mutex::new(device),
        })
    }

    /// Fire-and-forget command. `params` (if any) are placed at payload byte 4.
    fn cmd(&self, opcode: &[u8], params: &[u8]) -> Result<()> {
        let frame = build_frame(opcode, params);
        let dev = self.device.lock().unwrap();
        dev.write(&frame)?;
        Ok(())
    }

    /// Request/response. Sends the opcode, then reads the Input 0xCC reply.
    /// Returns the 64-byte payload (report ID stripped).
    fn query(&self, opcode: &[u8]) -> Result<[u8; CMD_PAYLOAD_LEN]> {
        let frame = build_frame(opcode, &[]);
        let dev = self.device.lock().unwrap();
        dev.write(&frame)?;

        let mut buf = [0u8; 1 + CMD_PAYLOAD_LEN];
        let n = dev.read_timeout(&mut buf, 1000)?;
        if n == 0 {
            return Err(anyhow!("no response to query opcode {:02x?}", opcode));
        }
        if buf[0] != REPORT_CMD {
            return Err(anyhow!(
                "unexpected response report id {:#04x} (want {:#04x})",
                buf[0],
                REPORT_CMD
            ));
        }
        let mut payload = [0u8; CMD_PAYLOAD_LEN];
        payload.copy_from_slice(&buf[1..]);
        Ok(payload)
    }

    /// First data byte of a GET response (payload index 4 == wire byte 5).
    fn query_byte(&self, opcode: &[u8]) -> Result<u8> {
        Ok(self.query(opcode)?[4])
    }
}

impl PeltaDevice for Pelta {
    fn firmware_version(&self) -> Result<String> {
        // Data region begins at payload[4] (wire byte 5), consistent with all
        // other GETs. First 4 bytes = HW revision, next 4 = FW revision, each
        // rendered as `%02X.%02X.%02X.%02X` (matching the ASUS HAL).
        let p = self.query(&OP_GET_FW_VERSION)?;
        let hw = &p[4..8];
        let fw = &p[8..12];
        Ok(format!(
            "{:02X}.{:02X}.{:02X}.{:02X} (hw {:02X}.{:02X}.{:02X}.{:02X})",
            fw[0], fw[1], fw[2], fw[3], hw[0], hw[1], hw[2], hw[3]
        ))
    }

    fn power_info(&self) -> Result<PowerInfo> {
        let raw_level = self.query_byte(&OP_GET_POWER_INFO)?;
        let charging = self.query_byte(&OP_GET_CHARGING_STATE)? != 0;
        Ok(PowerInfo { raw_level, charging })
    }

    fn headset_present(&self) -> Result<bool> {
        Ok(self.query_byte(&OP_GET_HEADSET_EXIST)? != 0)
    }

    fn led_enabled(&self) -> Result<bool> {
        Ok(self.query_byte(&OP_GET_LED_ON_OFF)? != 0)
    }

    fn noise_reduction(&self) -> Result<bool> {
        Ok(self.query_byte(&OP_GET_NR_ON_OFF)? != 0)
    }

    fn latency_mode(&self) -> Result<u8> {
        self.query_byte(&OP_GET_LATENCY_MODE)
    }

    fn sidetone_volume(&self) -> Result<u8> {
        self.query_byte(&OP_GET_SIDETONE_VOLUME)
    }

    fn sidetone_enabled(&self) -> Result<bool> {
        Ok(self.query_byte(&OP_GET_SIDETONE_ON_OFF)? != 0)
    }

    fn set_led_color(&self, color: Rgb) -> Result<()> {
        self.cmd(&OP_SET_SW_LED_COLOR, &[color.r, color.g, color.b])
    }

    fn set_light_effect(&self, mode: u8, intensity: u8, color: Rgb) -> Result<()> {
        if !LIGHT_EFFECT_MODES.contains(&mode) {
            return Err(anyhow!(
                "invalid light-effect mode {mode}; valid: {LIGHT_EFFECT_MODES:?}"
            ));
        }
        self.cmd(
            &OP_SET_LIGHT_EFFECT,
            &[mode, intensity, color.r, color.g, color.b],
        )
    }

    fn set_noise_reduction(&self, on: bool) -> Result<()> {
        self.cmd(&OP_SET_NR_ON_OFF, &[on as u8])
    }

    fn set_latency_mode(&self, value_ms: u8) -> Result<()> {
        if !LATENCY_MODE_VALUES_MS.contains(&value_ms) {
            return Err(anyhow!(
                "invalid latency value {value_ms}; valid: {LATENCY_MODE_VALUES_MS:?}"
            ));
        }
        self.cmd(&OP_SET_LATENCY_MODE, &[value_ms])
    }

    fn set_demo_mode(&self, on: bool) -> Result<()> {
        self.cmd(&OP_SET_DEMO_MODE_ON_OFF, &[on as u8])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Live smoke tests — ignored by default, run with a real wireless Pelta:
    //   cargo test -- --ignored --test-threads=1
    #[test]
    #[ignore]
    fn smoke_firmware() {
        let p = Pelta::open().unwrap();
        let v = p.firmware_version().unwrap();
        println!("firmware: {v}");
        assert!(!v.is_empty());
    }

    #[test]
    #[ignore]
    fn smoke_power() {
        let p = Pelta::open().unwrap();
        let info = p.power_info().unwrap();
        println!("power: {info:?}");
    }

    #[test]
    #[ignore]
    fn smoke_led_red() {
        let p = Pelta::open().unwrap();
        p.set_led_color(Rgb { r: 255, g: 0, b: 0 }).unwrap();
    }
}
