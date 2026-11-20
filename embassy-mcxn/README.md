# Embassy NXP MCX-N MCUs HAL

A Hardware Abstraction Layer (HAL) for the NXP MCX-N family of
microcontrollers using the Embassy async framework.

**Status: early bring-up.** Supported chip: MCXN947.

- `init` checks that the core clock is on its reset default (`fro_hf`, FIRC at
  48 MHz) and records the `main_clk`/`system_clk` frequencies; it does not
  switch the main clock source.
- Peripheral clock gating through SYSCON (`clocks::Gate`).
- Blocking GPIO: `Flex`, `Input`, `Output` and `OutputOpenDrain`, type-erased
  through `AnyPin`, with `embedded-hal` 0.2 and 1.0 implementations. There is
  no async `wait_for_*` yet.

No other peripheral has a driver yet. Examples for the FRDM-MCXN947 board are
in `examples/mcxn947`.
