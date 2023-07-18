# embassy-usb

TODO crate description

## Configuration

`embassy-usb` has some configuration settings that are set at compile time, affecting sizes
and counts of buffers.

They can be set in two ways:

- Via Cargo features: enable a feature like `<name>-<value>`. `name` must be in lowercase and
use dashes instead of underscores. For example. `max-interface-count-3`. Only a selection of values
is available, check `Cargo.toml` for the list.
- Via environment variables at build time: set the variable named `EMBASSY_USB_<value>`. For example 
`EMBASSY_USB_MAX_INTERFACE_COUNT=3 cargo build`. You can also set them in the `[env]` section of `.cargo/config.toml`. 
Any value can be set, unlike with Cargo features.

Environment variables take precedence over Cargo features. If two Cargo features are enabled for the same setting
with different values, compilation fails.

### `MAX_INTERFACE_COUNT`

Max amount of interfaces that can be created in one device. Default: 4.


## Interoperability

This crate can run on any executor.

## Minimum supported Rust version (MSRV)

This crate requires nightly Rust, due to using "async fn in trait" support.

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

