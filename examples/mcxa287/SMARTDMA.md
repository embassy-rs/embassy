# SmartDMA waveform

Build `smartdma-waveform` from this directory. See
[the common bring-up instructions](../mcxa266/SMARTDMA.md) for the unsafe
one-shot/static ownership model, expected nominal5kHz waveform, ISP-before-flash
procedure and explicit probe commands.

This example drives **P0_4 / SmartDMA PIO0 / camera D0**. Its exact connector
numbering has not been independently established for the287 board; check the
board's own P0_4/D0 labeling before probing. Do not silently substitute the577
schematic. No LED/camera/shield should be connected during this diagnostic.
