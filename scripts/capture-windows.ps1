# OpenPelta — Phase 0 capture helper (Windows)
# Requires: USBPcap, Wireshark (tshark), USB Device Tree Viewer
# Run as Administrator.

param(
    [string]$OutDir = "$PSScriptRoot\..\captures",
    [int]$DurationSec = 8
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Path $OutDir | Out-Null }

function Find-USBPcapCmd {
    $candidates = @(
        "C:\Program Files\USBPcap\USBPcapCMD.exe",
        "C:\Program Files (x86)\USBPcap\USBPcapCMD.exe"
    )
    foreach ($c in $candidates) { if (Test-Path $c) { return $c } }
    throw "USBPcap not found. Install from https://desowin.org/usbpcap/"
}

function List-Devices {
    Write-Host "=== USB devices (ASUS VID 0b05) ===" -ForegroundColor Cyan
    Get-PnpDevice -Class USB | Where-Object { $_.InstanceId -match "VID_0B05" } | Format-Table FriendlyName, InstanceId -AutoSize
}

function Capture-Action {
    param([string]$Label, [string]$Instruction)

    $usbpcap = Find-USBPcapCmd
    $outfile = Join-Path $OutDir ("{0}_{1:yyyyMMdd_HHmmss}.pcap" -f $Label, (Get-Date))

    Write-Host ""
    Write-Host "=== $Label ===" -ForegroundColor Yellow
    Write-Host $Instruction -ForegroundColor White
    Read-Host "Press ENTER when ready to start $DurationSec-sec capture"

    Write-Host "Capturing $DurationSec sec..." -ForegroundColor Green
    $proc = Start-Process -FilePath $usbpcap -ArgumentList "-d","\\.\USBPcap1","-o",$outfile,"-b","134217728" -PassThru -WindowStyle Hidden

    Write-Host ">>> PERFORM ACTION NOW IN ARMOURY CRATE <<<" -ForegroundColor Magenta
    Start-Sleep -Seconds $DurationSec

    Stop-Process -Id $proc.Id -Force
    Write-Host "Saved: $outfile" -ForegroundColor Green
}

List-Devices

$actions = @(
    @{ Label = "rgb_static_red";   Instr = "Set RGB mode Static, color RED" }
    @{ Label = "rgb_static_green"; Instr = "Set RGB mode Static, color GREEN" }
    @{ Label = "rgb_static_blue";  Instr = "Set RGB mode Static, color BLUE" }
    @{ Label = "rgb_breathing";    Instr = "Set RGB mode Breathing" }
    @{ Label = "rgb_off";          Instr = "Turn RGB OFF" }
    @{ Label = "eq_preset_fps";    Instr = "Switch EQ preset to FPS" }
    @{ Label = "eq_preset_music";  Instr = "Switch EQ preset to Music" }
    @{ Label = "eq_band_low";      Instr = "Move ONLY lowest EQ band to +6 dB" }
    @{ Label = "eq_band_high";     Instr = "Move ONLY highest EQ band to +6 dB" }
    @{ Label = "sidetone_max";     Instr = "Set Sidetone slider to MAX" }
    @{ Label = "sidetone_min";     Instr = "Set Sidetone slider to MIN" }
    @{ Label = "mic_mute_on";      Instr = "Toggle mic MUTE ON" }
    @{ Label = "mic_mute_off";     Instr = "Toggle mic MUTE OFF" }
    @{ Label = "sleep_timer_5";    Instr = "Set sleep timer to 5 minutes" }
    @{ Label = "sleep_timer_off";  Instr = "Disable sleep timer" }
    @{ Label = "fw_query";         Instr = "Open Firmware section / check for update" }
    @{ Label = "idle_30s";         Instr = "Do NOTHING for 30 sec (passive battery polls)" }
)

foreach ($a in $actions) { Capture-Action -Label $a.Label -Instruction $a.Instr }

Write-Host ""
Write-Host "Done. Send the entire '$OutDir' folder for analysis." -ForegroundColor Cyan
