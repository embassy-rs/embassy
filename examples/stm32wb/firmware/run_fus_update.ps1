# Build `fus_update` with the given fw_* features and drive a full FUS OTA with
# probe-rs only: download, reset, then stream RTT, automatically reconnecting
# across the resets the upgrader performs. Ctrl-C to quit.
#
# Usage:
#   ./run_fus_update.ps1 fw_fus fw_hci        # switch to the BLE HCI layer stack
#   ./run_fus_update.ps1 fw_fus fw_mac_ble    # switch to the BLE + MAC combo stack
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot\..

if ($Args.Count -eq 0) {
    Write-Error "usage: run_fus_update.ps1 <fw feature> [fw feature...]   e.g. run_fus_update.ps1 fw_fus fw_hci"
}
$Features = $Args -join ','
$Elf = 'target/thumbv7em-none-eabihf/release/fus_update'

cargo build --release --bin fus_update --features $Features

probe-rs download --chip STM32WB55RG $Elf
probe-rs reset --chip STM32WB55RG

# `probe-rs run` is unreliable on this board (see ../probe-rs-issue.md), so
# stream RTT with `attach`, reconnecting whenever the device resets.
while ($true) {
    probe-rs attach --chip STM32WB55RG $Elf
    Write-Host '=== attach dropped (device reset?); reconnecting (Ctrl-C to quit) ==='
    Start-Sleep -Seconds 2
}
