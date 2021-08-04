#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../serial_port.rs"]
mod serial_port;

use async_io::Async;
use embassy::io::AsyncBufReadExt;
use embassy::util::Forever;
use embassy_std::Executor;
use log::*;
use nix::sys::termios;

use self::serial_port::SerialPort;

#[embassy::task]
async fn run() {
    // Open the serial port.
    let baudrate = termios::BaudRate::B115200;
    let port = SerialPort::new("/dev/ttyACM0", baudrate).unwrap();
    //let port = Spy::new(port);

    // Use async_io's reactor for async IO.
    // This demonstrates how embassy's executor can drive futures from another IO library.
    // Essentially, async_io::Async converts from AsRawFd+Read+Write to futures's AsyncRead+AsyncWrite
    let port = Async::new(port).unwrap();

    // This implements futures's AsyncBufRead based on futures's AsyncRead
    let port = futures::io::BufReader::new(port);

    // We can then use FromStdIo to convert from futures's AsyncBufRead+AsyncWrite
    // to embassy's AsyncBufRead+AsyncWrite
    let mut port = embassy::io::FromStdIo::new(port);

    info!("Serial opened!");

    loop {
        let mut buf = [0u8; 256];
        let n = port.read(&mut buf).await.unwrap();
        info!("read {:?}", &buf[..n]);
    }
}

static EXECUTOR: Forever<Executor> = Forever::new();

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("async_io", log::LevelFilter::Info)
        .format_timestamp_nanos()
        .init();

    let executor = EXECUTOR.put(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(run()).unwrap();
    });
}
