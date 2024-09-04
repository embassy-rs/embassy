#![no_std]
#![no_main]

use core::mem::MaybeUninit;
use core::net::IpAddr;
use core::ptr::addr_of_mut;
use core::str::FromStr;
use core::slice;

use defmt::{assert, info, warn, unwrap};
use heapless::Vec;
use embassy_executor::Spawner;
use embassy_net::{Ipv4Address, Ipv4Cidr, Stack, StackResources};
use embassy_net_nrf91::{Runner, State, context};
use embassy_nrf::buffered_uarte::{self, BufferedUarteTx};
use embassy_nrf::gpio::{AnyPin, Level, Output, OutputDrive, Pin};
use embassy_nrf::uarte::Baudrate;
use embassy_nrf::{bind_interrupts, interrupt, peripherals, uarte};
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[interrupt]
fn IPC() {
    embassy_net_nrf91::on_ipc_irq();
}

bind_interrupts!(struct Irqs {
    UARTE0_SPIM0_SPIS0_TWIM0_TWIS0 => buffered_uarte::InterruptHandler<peripherals::SERIAL0>;
});

// embassy-net-nrf91 only supports blocking trace write for now.
// We don't want to block packet processing with slow uart writes, so
// we make an adapter that writes whatever fits in the buffer and drops
// data if it's full.
struct TraceWriter(BufferedUarteTx<'static, peripherals::SERIAL0>);

impl embedded_io::ErrorType for TraceWriter {
    type Error = core::convert::Infallible;
}

impl embedded_io::Write for TraceWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let _ = self.0.try_write(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[embassy_executor::task]
async fn modem_task(runner: Runner<'static, TraceWriter>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<embassy_net_nrf91::NetDriver<'static>>) -> ! {
    stack.run().await
}

#[embassy_executor::task]
async fn blink_task(pin: AnyPin) {
    let mut led = Output::new(pin, Level::Low, OutputDrive::Standard);
    loop {
        led.set_high();
        Timer::after_millis(1000).await;
        led.set_low();
        Timer::after_millis(1000).await;
    }
}

extern "C" {
    static __start_ipc: u8;
    static __end_ipc: u8;
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    info!("Hello World!");

    unwrap!(spawner.spawn(blink_task(p.P0_02.degrade())));

    let ipc_mem = unsafe {
        let ipc_start = &__start_ipc as *const u8 as *mut MaybeUninit<u8>;
        let ipc_end = &__end_ipc as *const u8 as *mut MaybeUninit<u8>;
        let ipc_len = ipc_end.offset_from(ipc_start) as usize;
        slice::from_raw_parts_mut(ipc_start, ipc_len)
    };

    static mut TRACE_BUF: [u8; 4096] = [0u8; 4096];
    let mut config = uarte::Config::default();
    config.baudrate = Baudrate::BAUD1M;
    let trace_writer = TraceWriter(BufferedUarteTx::new(
        //let trace_uart = BufferedUarteTx::new(
        unsafe { peripherals::SERIAL0::steal() },
        Irqs,
        unsafe { peripherals::P0_01::steal() },
        //unsafe { peripherals::P0_14::steal() },
        config,
        unsafe { &mut *addr_of_mut!(TRACE_BUF) },
    ));

    static STATE: StaticCell<State> = StaticCell::new();
    let (device, control, runner) = embassy_net_nrf91::new(STATE.init(State::new()), ipc_mem, trace_writer).await;
    unwrap!(spawner.spawn(modem_task(runner)));

    let config = embassy_net::Config::default();

    // Generate "random" seed. nRF91 has no RNG, TODO figure out something...
    let seed = 123456;

    // Init network stack
    static RESOURCES: StaticCell<StackResources<2>> = StaticCell::new();
    static STACK: StaticCell<Stack<embassy_net_nrf91::NetDriver<'static>>> = StaticCell::new();
    let stack = &*STACK.init(Stack::new(
        device,
        config,
        RESOURCES.init(StackResources::<2>::new()),
        seed,
    ));

    unwrap!(spawner.spawn(net_task(stack)));

    let control = context::Control::new(control, 0).await;

    unwrap!(control.configure(context::Config {
        gateway: "iot.nat.es",
        auth_prot: context::AuthProt::Pap,
        auth: Some(("orange", "orange")),
    }).await);

    info!("waiting for attach...");

    let mut status = unwrap!(control.status().await);
    while !status.attached && status.ip.is_none() {
        Timer::after_millis(1000).await;
        status = unwrap!(control.status().await);
        info!("STATUS: {:?}", status);
    }

    let Some(IpAddr::V4(addr)) = status.ip else {
        panic!("Unexpected IP address");
    };
    let addr = Ipv4Address(addr.octets());

    let gateway = if let Some(IpAddr::V4(addr)) = status.gateway {
        Some(Ipv4Address(addr.octets()))
    } else {
        None
    };

    let mut dns_servers = Vec::new();
    for dns in status.dns {
        if let IpAddr::V4(ip) = dns {
            unwrap!(dns_servers.push(Ipv4Address(ip.octets())));
        }
    }

    stack.set_config_v4(embassy_net::ConfigV4::Static(embassy_net::StaticConfigV4 {
        address: Ipv4Cidr::new(addr, 32),
        gateway,
        dns_servers,
    }));

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    loop {
        let mut socket = embassy_net::tcp::TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        info!("Connecting...");
        let host_addr = embassy_net::Ipv4Address::from_str("45.79.112.203").unwrap();
        if let Err(e) = socket.connect((host_addr, 4242)).await {
            warn!("connect error: {:?}", e);
            Timer::after_secs(1).await;
            continue;
        }
        info!("Connected to {:?}", socket.remote_endpoint());

        let msg = b"Hello world!\n";
        for _ in 0..10 {
            if let Err(e) = socket.write_all(msg).await {
                warn!("write error: {:?}", e);
                break;
            }
            info!("txd: {}", core::str::from_utf8(msg).unwrap());
            Timer::after_secs(1).await;
        }
    }
}
