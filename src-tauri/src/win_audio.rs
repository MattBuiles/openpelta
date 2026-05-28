//! Thin wrapper over the Windows Core Audio API for things the Pelta's
//! vendor HID channel does **not** expose — mic mute being the big one.
//!
//! Mic mute is a USB Audio Class control on the headset's capture endpoint,
//! not a vendor command. There is no `setMicMute` in the C++ HAL we reversed.
//! Windows surfaces it through `IAudioEndpointVolume::SetMute` on the
//! default communications capture device, which is what this module drives.

#![cfg(target_os = "windows")]

use anyhow::{anyhow, Result};
use windows::core::Interface;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    eCapture, eCommunications, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
};

/// Initialize COM for this thread. Safe to call repeatedly; subsequent calls
/// after a successful one return `RPC_E_CHANGED_MODE` or similar which we
/// treat as "already initialized, fine".
fn ensure_com() {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
}

/// Acquire `IAudioEndpointVolume` for the default *communications* capture
/// endpoint (i.e. the headset's mic). This is the same endpoint Windows's
/// volume mixer mute button targets.
fn capture_endpoint() -> Result<IAudioEndpointVolume> {
    ensure_com();
    unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let device: IMMDevice =
            enumerator.GetDefaultAudioEndpoint(eCapture, eCommunications)?;
        let endpoint: IAudioEndpointVolume = device
            .Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
            .map_err(|e| anyhow!("Activate IAudioEndpointVolume failed: {e}"))?;
        Ok(endpoint)
    }
}

pub fn get_mic_mute() -> Result<bool> {
    let ep = capture_endpoint()?;
    unsafe { Ok(ep.GetMute()?.as_bool()) }
}

pub fn set_mic_mute(mute: bool) -> Result<()> {
    let ep = capture_endpoint()?;
    unsafe { ep.SetMute(mute, std::ptr::null())? };
    Ok(())
}
