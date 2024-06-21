#![no_std]
#![no_main]

use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;
use core::str::FromStr;
use core::{slice, str};

use defmt::{assert, *};
use embassy_executor::Spawner;
use embassy_net::{Ipv4Address, Ipv4Cidr, Stack, StackResources};
use embassy_net_nrf91::{Runner, State};
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
        Timer::after_millis(100).await;
        led.set_low();
        Timer::after_millis(100).await;
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

    control.wait_init().await;
    info!("INIT OK");

    let mut buf = [0u8; 256];

    let n = control.at_command(b"AT+CFUN?", &mut buf).await;
    info!("AT resp: '{}'", unsafe { str::from_utf8_unchecked(&buf[..n]) });

    let n = control
        .at_command(b"AT+CGDCONT=0,\"IP\",\"iot.nat.es\"", &mut buf)
        .await;
    info!("AT resp: '{}'", unsafe { str::from_utf8_unchecked(&buf[..n]) });
    let n = control
        .at_command(b"AT+CGAUTH=0,1,\"orange\",\"orange\"", &mut buf)
        .await;
    info!("AT resp: '{}'", unsafe { str::from_utf8_unchecked(&buf[..n]) });

    let n = control.at_command(b"AT+CFUN=1", &mut buf).await;
    info!("AT resp: '{}'", unsafe { str::from_utf8_unchecked(&buf[..n]) });

    info!("waiting for attach...");
    loop {
        Timer::after_millis(500).await;
        let n = control.at_command(b"AT+CGATT?", &mut buf).await;
        let mut res = &buf[..n];
        pop_prefix(&mut res, b"+CGATT: ");
        let res = split_field(&mut res);
        info!("AT resp field: '{}'", unsafe { str::from_utf8_unchecked(res) });
        if res == b"1" {
            break;
        }
    }

    let n = control.at_command(b"AT+CGPADDR=0", &mut buf).await;
    let mut res = &buf[..n];
    pop_prefix(&mut res, b"+CGPADDR: 0,");
    let ip = split_field(&mut res);
    let ip = Ipv4Address::from_str(unsafe { str::from_utf8_unchecked(ip) }).unwrap();
    info!("IP: '{}'", ip);

    info!("============== OPENING SOCKET");
    control.open_raw_socket().await;

    stack.set_config_v4(embassy_net::ConfigV4::Static(embassy_net::StaticConfigV4 {
        address: Ipv4Cidr::new(ip, 32),
        gateway: None,
        dns_servers: Default::default(),
    }));

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    loop {
        let mut socket = embassy_net::tcp::TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        info!("Connecting...");
        let host_addr = embassy_net::Ipv4Address::from_str("83.51.182.206").unwrap();
        if let Err(e) = socket.connect((host_addr, 8000)).await {
            warn!("connect error: {:?}", e);
            continue;
        }
        info!("Connected to {:?}", socket.remote_endpoint());

        let msg = b"Hello world!\n";
        loop {
            if let Err(e) = socket.write_all(msg).await {
                warn!("write error: {:?}", e);
                break;
            }
            info!("txd: {}", core::str::from_utf8(msg).unwrap());
            Timer::after_secs(1).await;
        }
    }
}

fn is_whitespace(char: u8) -> bool {
    match char {
        b'\r' | b'\n' | b' ' => true,
        _ => false,
    }
}

fn is_separator(char: u8) -> bool {
    match char {
        b',' | b'\r' | b'\n' | b' ' => true,
        _ => false,
    }
}

fn split_field<'a>(data: &mut &'a [u8]) -> &'a [u8] {
    while !data.is_empty() && is_whitespace(data[0]) {
        *data = &data[1..];
    }

    if data.is_empty() {
        return &[];
    }

    if data[0] == b'"' {
        let data2 = &data[1..];
        let end = data2.iter().position(|&x| x == b'"').unwrap_or(data2.len());
        let field = &data2[..end];
        let mut rest = &data2[data2.len().min(end + 1)..];
        if rest.first() == Some(&b'\"') {
            rest = &rest[1..];
        }
        while !rest.is_empty() && is_separator(rest[0]) {
            rest = &rest[1..];
        }
        *data = rest;
        field
    } else {
        let end = data.iter().position(|&x| is_separator(x)).unwrap_or(data.len());
        let field = &data[0..end];
        let rest = &data[data.len().min(end + 1)..];
        *data = rest;
        field
    }
}

fn pop_prefix(data: &mut &[u8], prefix: &[u8]) {
    assert!(data.len() >= prefix.len());
    assert!(&data[..prefix.len()] == prefix);
    *data = &data[prefix.len()..];
}
