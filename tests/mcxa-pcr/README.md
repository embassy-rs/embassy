# MCXA PCR register host tests

From this directory, run:

```text
cargo test
```

Tests the HAL's PCR adapter using MCXA156 PAC registers backed by RAM. Checks
all eight PCR layouts for bit preservation, mux widths, and rejection of
invalid mux selections before a write. These tests are not run by CI.

A separate crate avoids the HAL's Cortex-M executor dev-dependency, which
cannot run on the host. Keep its PAC revision aligned with the HAL's two
`nxp-pac` dependencies.
