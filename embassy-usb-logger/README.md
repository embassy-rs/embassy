# embassy-usb-logger

USB implementation of the `log` crate. This logger can be used by any device that implements `embassy-usb`. When running,
it will output all logging done through the `log` facade to the USB serial peripheral.

## Usage

Add the following embassy task to your application. The `Driver` type is different depending on which HAL you use.

 ```rust
#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}
```

## Minimum supported Rust version (MSRV)

Embassy is guaranteed to compile on the latest stable Rust version at the time of release. It might compile with older versions but that may change in any new patch release.

## License

This work is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
