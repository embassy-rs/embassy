# embassy-usb-host-driver

Driver traits for USB host support in Embassy.

This crate contains the traits that USB host hardware drivers implement. The traits are used by `embassy-usb-host` to provide a hardware-independent USB host stack.

## Traits

- `HostDriver` — entry point, produces a `HostBus`
- `HostBus` — port management (connect/disconnect/reset) and channel allocation
- `HostChannel` — transfer operations (control, bulk IN, bulk OUT)
