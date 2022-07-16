#![no_std]
#![no_main]
#![feature(generic_associated_types, type_alias_impl_trait)]

use core::convert::Infallible;
use core::future::Future;

use defmt::{assert, assert_eq, panic, *};
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Ipv4Address, Ipv4Cidr, Stack, StackResources};
use embassy_rp::gpio::{Flex, Level, Output, Pin};
use embassy_rp::peripherals::{PIN_23, PIN_24, PIN_25, PIN_29};
use embassy_rp::Peripherals;
use embedded_hal_1::spi::ErrorType;
use embedded_hal_async::spi::{ExclusiveDevice, SpiBusFlush, SpiBusRead, SpiBusWrite};
use embedded_io::asynch::{Read, Write};
use {defmt_rtt as _, panic_probe as _};

macro_rules! forever {
    ($val:expr) => {{
        type T = impl Sized;
        static FOREVER: Forever<T> = Forever::new();
        FOREVER.put_with(move || $val)
    }};
}

#[embassy::task]
async fn wifi_task(
    runner: cyw43::Runner<'static, Output<'static, PIN_23>, ExclusiveDevice<MySpi, Output<'static, PIN_25>>>,
) -> ! {
    runner.run().await
}

#[embassy::task]
async fn net_task(stack: &'static Stack<cyw43::NetDevice<'static>>) -> ! {
    stack.run().await
}

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    info!("Hello World!");

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let clk = Output::new(p.PIN_29, Level::Low);
    let mut dio = Flex::new(p.PIN_24);
    dio.set_low();
    dio.set_as_output();

    let bus = MySpi { clk, dio };
    let spi = ExclusiveDevice::new(bus, cs);

    let state = forever!(cyw43::State::new());
    let (mut control, runner) = cyw43::new(state, pwr, spi).await;

    spawner.spawn(wifi_task(runner)).unwrap();

    let net_device = control.init().await;

    //control.join_open("MikroTik-951589").await;
    control.join_wpa2("DirbaioWifi", "HelloWorld").await;

    let config = embassy_net::ConfigStrategy::Dhcp;
    //let config = embassy_net::ConfigStrategy::Static(embassy_net::Config {
    //    address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 69, 2), 24),
    //    dns_servers: Vec::new(),
    //    gateway: Some(Ipv4Address::new(192, 168, 69, 1)),
    //});

    // Generate random seed
    let seed = 0x0123_4567_89ab_cdef; // chosen by fair dice roll. guarenteed to be random.

    // Init network stack
    let stack = &*forever!(Stack::new(
        net_device,
        config,
        forever!(StackResources::<1, 2, 8>::new()),
        seed
    ));

    unwrap!(spawner.spawn(net_task(stack)));

    // And now we can use it!

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(embassy_net::SmolDuration::from_secs(10)));

        info!("Listening on TCP:1234...");
        if let Err(e) = socket.accept(1234).await {
            warn!("accept error: {:?}", e);
            continue;
        }

        info!("Received connection from {:?}", socket.remote_endpoint());

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    warn!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    warn!("read error: {:?}", e);
                    break;
                }
            };

            info!("rxd {:02x}", &buf[..n]);

            match socket.write_all(&buf[..n]).await {
                Ok(()) => {}
                Err(e) => {
                    warn!("write error: {:?}", e);
                    break;
                }
            };
        }
    }
}

struct MySpi {
    /// SPI clock
    clk: Output<'static, PIN_29>,

    /// 4 signals, all in one!!
    /// - SPI MISO
    /// - SPI MOSI
    /// - IRQ
    /// - strap to set to gSPI mode on boot.
    dio: Flex<'static, PIN_24>,
}

impl ErrorType for MySpi {
    type Error = Infallible;
}

impl SpiBusFlush for MySpi {
    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        async move { Ok(()) }
    }
}

impl SpiBusRead<u32> for MySpi {
    type ReadFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn read<'a>(&'a mut self, words: &'a mut [u32]) -> Self::ReadFuture<'a> {
        async move {
            self.dio.set_as_input();
            for word in words {
                let mut w = 0;
                for _ in 0..32 {
                    w = w << 1;

                    // rising edge, sample data
                    if self.dio.is_high() {
                        w |= 0x01;
                    }
                    self.clk.set_high();

                    // falling edge
                    self.clk.set_low();
                }
                *word = w
            }

            Ok(())
        }
    }
}

impl SpiBusWrite<u32> for MySpi {
    type WriteFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn write<'a>(&'a mut self, words: &'a [u32]) -> Self::WriteFuture<'a> {
        async move {
            self.dio.set_as_output();
            for word in words {
                let mut word = *word;
                for _ in 0..32 {
                    // falling edge, setup data
                    self.clk.set_low();
                    if word & 0x8000_0000 == 0 {
                        self.dio.set_low();
                    } else {
                        self.dio.set_high();
                    }

                    // rising edge
                    self.clk.set_high();

                    word = word << 1;
                }
            }
            self.clk.set_low();

            self.dio.set_as_input();
            Ok(())
        }
    }
}
