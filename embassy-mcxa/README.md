# Embassy NXP MCX-A MCUs HAL

A Hardware Abstraction Layer (HAL) for the NXP MCX-A family of
microcontrollers using the Embassy async framework. This HAL provides
safe, idiomatic Rust interfaces for GPIO, UART, and OSTIMER
peripherals.

## Initial MCXN support

Chip features cover the NXP MCXN23x, N24x, N52x, N53x, N54x, N55x and N94x
device groups through 14 numeric chip selections, such as `mcxn556` and
`mcxn947`. There are no package selectors, separate S/T selectors, or MCXN
compatibility aliases.

Suffix variants share PAC register definitions only where those maps match;
this does not equate their security/ROM behavior, electrical limits, memory
capacity, or errata. Use a linker memory layout appropriate for the actual chip
and pins bonded on the actual package.

The initial profile supports the primary core with FRO12 at 12 MHz, a 3 MHz
slow bus, the 1 MHz OSTIMER time driver, and GPIO on P0_10, P0_27 and P1_2.
MCXN23x lacks P0_10, so only P0_27 and P1_2 are implemented there. N2xx is
single-core; on dual-core N5xx/N9xx, CPU1 must remain held in reset. Other MCXN
peripherals are not supported yet. Only MCXN556S has been hardware validated;
no hardware-validation claim is made for other chip selections. Use the ordinary
`embassy-executor/platform-cortex-m` executor; the HAL's `executor-platform`
and `time-driver-lptmr` features are not supported on MCXN.

The clock setup retains inherited flash wait states, RAM ECC settings and
security/calibration state. Nominal clock frequencies are checked through
source/divider register readback, not measured by an independent reference.

GPIO/PORT clocks are enabled without resetting the whole blocks, retaining
inherited access-protection and pad state. Initialization requires those blocks
to be out of reset, and the boot environment must permit access to the selected
pins. The HAL does not override access policy to make an otherwise denied pin
usable. Initialization preserves the caller's CPU interrupt-mask state.

### Dedicated pins

Initial MCXN support does not implement dedicated-pin reclamation through
`sosc-as-gpio`, `rosc-32k-as-gpio`, `swd-as-gpio`, `swd-swo-as-gpio`,
`jtag-extras-as-gpio`, or `dangerous-reset-as-gpio`. Their pin-mode initialization
applies to MCXA only. In particular, the default `swd-swo-as-gpio` feature does
not change the MCXN SWO pin's mode.

These features may affect which pin tokens the PAC metadata exposes, but they
do not add MCXN GPIO implementations beyond the supported pins listed above.
MCXN initialization preserves inherited dedicated-pin modes.

### Boot policy

The MCXN556 blinky example is a plain, unsigned image. Autonomous startup
requires a device boot policy that permits it; a secure-boot-enabled device
requires an appropriately authenticated image instead. The HAL does not change
that policy. Debug-mailbox mass erase is a separate, destructive maintenance
operation, never part of HAL initialization or ordinary example programming.

### SRAM clock gating

The HAL preserves the inherited SRAM automatic clock-gating settings and does
not install a pre-initialization hook. Automatic clock gating does not remove
SRAM power; the documented cost is one extra bus cycle on the next access.
The reference manual recommends disabling it for code/data requiring
deterministic access latency, not for retaining ordinary SRAM contents.

The boot environment must provide usable SRAM, including any ECC initialization
required by its selected memory layout. The HAL does not change ECC enablement.
