# embassy-nxp-drivers

HAL-independent driver cores for NXP IP blocks, shared by the Embassy NXP HALs.

Each module is the instance-erased body of one IP's driver: a read-only `Info` (register block,
interrupt line), a zero-initialised `State`, a driver struct entered through an `unsafe fn new`,
and an `on_interrupt(info, state)` entry point. The crate defines no traits for HALs to implement
and names no HAL or chip item. The HAL that depends on it owns the public driver: sealed
`Instance` and pin traits, `InterruptHandler<T>` on its `interrupt::typelevel`, clocks, pin mux and
the `embedded-hal` impls. This is the model of `embassy-usb-synopsys-otg` and embassy-stm32's
`usb::Driver`.

| Module | IP (SDK driver) | Feature |
|--------|-----------------|---------|
| `gpio` | GPIO (`fsl_gpio`) | `gpio` |

The chip is selected by the HAL through `nxp-pac`. A module compiles when its feature is enabled
and nxp-pac reports the IP for the selected chip; the cores gate only on the cfgs nxp-pac exports.

This crate is not meant to be used directly by applications: use the HAL's driver modules.
