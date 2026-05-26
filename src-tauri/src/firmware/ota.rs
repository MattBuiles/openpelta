use anyhow::{anyhow, Result};

/// Placeholder. Real OTA flow depends on Fase 0 reversing.
/// Hard-gated behind enable_firmware feature.
pub fn flash(_path: &std::path::Path) -> Result<()> {
    Err(anyhow!("Firmware OTA not implemented"))
}
