# LPC55S69 Examples

## Available examples:
- blinky_nop: Blink the integrated RED LED using nops as delay. Useful for flashing simple and known-good software on board.
- button_executor: Turn on/off an LED by pressing the USER button. Demonstrates how to use the PINT and GPIO drivers.
- blinky_embassy_time: Blink the integrated RED LED using `embassy-time`. Demonstrates how to use the time-driver that uses RTC. 
- usb_fs_serial / usb_hs_serial / usb_dual: CDC-ACM serial over USB0 (P10), USB1 (P9), or both.
- usb_fs_throughput / usb_hs_throughput: bulk throughput peers for
  [`scripts/usb_throughput.py`](scripts/usb_throughput.py). Unlike the serial examples these two
  advertise WinUSB, so on Windows they appear as a libusb device rather than a COM port and the
  script drives their endpoints directly; Windows' CDC driver drops packets at the high-speed
  rate. Linux ignores the descriptors and binds `cdc_acm` as usual. See
  [`tests/lpc55/README.md`](../../tests/lpc55/README.md) for the measurements.

## Important Notes

On older version of probe-rs, some examples (such as `blinky_embassy_time`) do not work directly after flashing and the board must be reset after flashing. It is reccomended to update the version of probe-rs to the latest one.

When developing drivers for this board, probe-rs might not be able to flash the board after entering a fault. Either reset the board to clear the fault, or use NXP's proprietary software `LinkServer`/`LinkFlash` to bring the board back to a known-good state.