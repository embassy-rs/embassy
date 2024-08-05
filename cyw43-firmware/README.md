# WiFi + Bluetooth firmware blobs

Firmware obtained from https://github.com/georgerobotics/cyw43-driver/tree/main/firmware

Licensed under the [Infineon Permissive Binary License](./LICENSE-permissive-binary-license-1.0.txt)

## Changelog

* 2023-08-21: synced with `a1dc885` - Update 43439 fw + clm to come from `wb43439A0_7_95_49_00_combined.h` + add Bluetooth firmware
* 2023-07-28: synced with `ad3bad0` - Update 43439 fw from 7.95.55 to 7.95.62

## Notes

If you update these files, please update the lengths in the `tests/rp/src/bin/cyw43_perf.rs` test (which relies on these files running from RAM).
