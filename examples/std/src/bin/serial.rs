#![feature(type_alias_impl_trait)]

#[path = "../serial_port.rs"]
mod serial_port;

use async_io::Async;
use embassy_executor::Executor;
use embedded_io_async::Read;
use log::*;
use nix::sys::termios;
use static_cell::StaticCell;

use self::serial_port::SerialPort;

#[embassy_executor::task]
async fn run() {
    // Open the serial port.
    let baudrate = termios::BaudRate::B115200;
    let port = SerialPort::new("/dev/ttyACM0", baudrate).unwrap();
    //let port = Spy::new(port);

    // Use async_io's reactor for async IO.
    // This demonstrates how embassy's executor can drive futures from another IO library.
    // Essentially, async_io::Async converts from AsRawFd+Read+Write to futures's AsyncRead+AsyncWrite
    let port = Async::new(port).unwrap();

    // We can then use FromStdIo to convert from futures's AsyncRead+AsyncWrite
    // to embedded_io's async Read+Write.
    //
    // This is not really needed, you could write the code below using futures::io directly.
    // It's useful if you want to have portable code across embedded and std.
    let mut port = embedded_io_adapters::futures_03::FromFutures::new(port);

    info!("Serial opened!");

    loop {
        let mut buf = [0u8; 256];
        let n = port.read(&mut buf).await.unwrap();
        info!("read {:?}", &buf[..n]);
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("async_io", log::LevelFilter::Info)
        .format_timestamp_nanos()
        .init();

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(run()).unwrap();
    });
}
