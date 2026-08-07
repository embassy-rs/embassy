# LPC55S69 Examples

## Available examples:
- blinky_nop: Blink the integrated RED LED using nops as delay. Useful for flashing simple and known-good software on board.
- button_executor: Turn on/off an LED by pressing the USER button. Demonstrates how to use the PINT and GPIO drivers.
- blinky_embassy_time: Blink the integrated RED LED using `embassy-time`. Demonstrates how to use the time-driver that uses RTC. 

## Important Notes

On older version of probe-rs, some examples (such as `blinky_embassy_time`) do not work directly after flashing and the board must be reset after flashing. It is reccomended to update the version of probe-rs to the latest one.

When developing drivers for this board, probe-rs might not be able to flash the board after entering a fault. Either reset the board to clear the fault, or use NXP's proprietary software `LinkServer`/`LinkFlash` to bring the board back to a known-good state.