use anyhow::{anyhow, Result};
use hidapi::{HidApi, HidDevice};
use std::sync::Mutex;

use super::protocol::*;
use super::{PeltaDevice, Rgb, RgbMode};

pub struct Pelta {
    device: Mutex<HidDevice>,
}

impl Pelta {
    pub fn open() -> Result<Self> {
        let api = HidApi::new()?;
        let device = api
            .device_list()
            .find(|d| {
                d.vendor_id() == ASUS_VID
                    && (d.product_id() == PELTA_WIRELESS_PID || d.product_id() == PELTA_WIRED_PID)
            })
            .ok_or_else(|| anyhow!("Pelta not found"))?
            .open_device(&api)?;
        Ok(Self {
            device: Mutex::new(device),
        })
    }

    fn write(&self, report_id: u8, payload: &[u8]) -> Result<()> {
        let mut buf = Vec::with_capacity(payload.len() + 1);
        buf.push(report_id);
        buf.extend_from_slice(payload);
        let dev = self.device.lock().unwrap();
        dev.write(&buf)?;
        Ok(())
    }

    fn read(&self, report_id: u8, len: usize) -> Result<Vec<u8>> {
        let dev = self.device.lock().unwrap();
        let mut buf = vec![0u8; len + 1];
        buf[0] = report_id;
        dev.get_feature_report(&mut buf)?;
        Ok(buf)
    }
}

impl PeltaDevice for Pelta {
    fn battery(&self) -> Result<u8> {
        let r = self.read(REPORT_BATTERY, 8)?;
        Ok(*r.get(1).unwrap_or(&0))
    }

    fn set_sidetone(&self, level: u8) -> Result<()> {
        self.write(REPORT_SIDETONE, &[level])
    }

    fn set_mic_mute(&self, muted: bool) -> Result<()> {
        self.write(REPORT_MIC_MUTE, &[muted as u8])
    }

    fn set_sleep_timer(&self, minutes: u16) -> Result<()> {
        self.write(REPORT_SLEEP_TIMER, &minutes.to_le_bytes())
    }

    fn firmware_version(&self) -> Result<String> {
        let r = self.read(REPORT_FIRMWARE, 16)?;
        Ok(r.iter().skip(1).take(8).map(|b| format!("{:02x}", b)).collect())
    }

    fn set_rgb(&self, mode: RgbMode, color: Rgb, speed: u8) -> Result<()> {
        let mode_byte: u8 = match mode {
            RgbMode::Off => 0,
            RgbMode::Static => 1,
            RgbMode::Breathing => 2,
            RgbMode::Wave => 3,
            RgbMode::Rainbow => 4,
        };
        self.write(REPORT_RGB, &[mode_byte, color.r, color.g, color.b, speed])
    }

    fn set_eq(&self, bands: &[f32; 10]) -> Result<()> {
        let mut payload = Vec::with_capacity(10);
        for &b in bands.iter() {
            let scaled = ((b.clamp(-12.0, 12.0) + 12.0) * 10.0) as u8;
            payload.push(scaled);
        }
        self.write(REPORT_EQ, &payload)
    }
}
