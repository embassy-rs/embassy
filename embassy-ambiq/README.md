# embassy-ambiq

Support for Ambiq Apollo3 microcontrollers.

## Status
- **Supported:** GPIO, Time Driver (STIMER/CTIMER0)
- **Primary Target:** Apollo3 Blue / SparkFun Artemis

## Quirks

### Time Drivers
Timer features (`time-driver-stimer-*` or `time-driver-ctimer0`) bundle the hardware timer, clock source, and `embassy-time` tick rate together. Enable exactly one. 
- `time-driver-stimer-xtal-32768`: Standard continuous timekeeping.
- `time-driver-ctimer0`: Experimental one-shot mode (alarms do not re-arm).

### SVL Bootloader (SparkFun Artemis)
The Artemis SVL bootloader hands off execution at `0x10000`. If you are flashing via UART/SVL, you can enable the `svl-vtor` feature. This repoints the vector table to `0x10000` during `embassy_ambiq::init()`. Do not enable this if you are using a raw SWD probe at `0x0`.

### PAC
The `apollo3-pac` is currently at [apollo3-pac-rs](https://github.com/Fo-Zi/apollo3-pac-rs).
