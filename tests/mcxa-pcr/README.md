# MCXA PCR register host tests

From this directory, run:

```text
cargo test
```

This native-host command is a manual test lane; adding it to shared upstream
CI is a separate follow-up. Current CI builds/checks do not execute these
regression tests. The harness compiles
the HAL's actual PCR adapter against the same pinned upstream MCXA156 PAC,
with register storage backed by aligned RAM. It checks all eight PCR types:
optional/reserved bit preservation, mux selector widths, and rejection of
out-of-range or PORT4-reserved mux selections before a write. It does not
validate board wiring, electrical behavior, or hardware locking semantics.

The separate harness avoids `embassy-mcxa`'s Cortex-M executor dev-dependency,
which contains ARM assembly and cannot run on the host. Keep this PAC revision
aligned with both `embassy-mcxa` dependency entries when updating it.
