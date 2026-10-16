# MCXA PCR register host tests

From this directory, run:

```text
cargo test --target x86_64-pc-windows-msvc
```

Use the host's Rust target on other platforms. This manual host lane compiles
the HAL's actual PCR adapter against the same pinned upstream MCXA156 PAC,
with register storage backed by aligned RAM. It checks all eight PCR types:
optional/reserved bit preservation, mux selector widths, and rejection of
out-of-range or PORT4-reserved mux selections before a write. It does not
validate board wiring, electrical behavior, or hardware locking semantics.

The separate harness avoids `embassy-mcxa`'s Cortex-M executor dev-dependency,
which contains ARM assembly and cannot run on the host. Keep this PAC revision
aligned with both `embassy-mcxa` dependency entries when updating it.
