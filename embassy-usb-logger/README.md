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
