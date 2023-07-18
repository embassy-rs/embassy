# embassy-net-driver

This crate contains the driver trait necessary for adding [`embassy-net`](https://crates.io/crates/embassy-net) support
for a new hardware platform.

If you want to *use* `embassy-net` with already made drivers, you should depend on the main `embassy-net` crate, not on this crate.

If you are writing a driver, you  should depend only on this crate, not on the main `embassy-net` crate.
This will allow your driver to continue working for newer `embassy-net` major versions, without needing an update,
if the driver trait has not had breaking changes.

See also [`embassy-net-driver-channel`](https://crates.io/crates/embassy-net-driver-channel), which provides a higer-level API
to construct a driver that processes packets in its own background task and communicates with the `embassy-net` task via
packet queues for RX and TX.

## Interoperability

This crate can run on any executor.

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
