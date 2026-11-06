# SmartDMA waveform

Build `smartdma-waveform` from this directory. This board is **not attached** in
the current setup; build success is not hardware or timing validation.

Output is **P0_4 / SmartDMA PIO0**, camera **J9 pin4**; ground is **J9 pin18**
(SPF-95896_A1 header sheet13). Disconnect the camera/shields and use only a
high-impedance, 3.3V-compatible analyzer input and common ground.

See [the common bring-up instructions](../mcxa266/SMARTDMA.md) for expected
nominal5kHz timing, the one-shot/static ownership restrictions and the
ISP-before-reflash procedure. No stop, restart or resource-reclamation guarantee
is currently provided.
