#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

use core::pin::Pin;
use core::sync::atomic::{AtomicUsize, Ordering};

use cortex_m_rt::entry;
use defmt::{info, unwrap};
use defmt_rtt as _; // global logger
use embassy::executor::{Executor, Spawner};
use embassy::io::AsyncWriteExt;
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_macros::interrupt_take;
use embassy_net::{
    Config as NetConfig, Ipv4Address, Ipv4Cidr, StackResources, StaticConfigurator, TcpSocket,
};
use embassy_stm32::clock::{Alarm, Clock};
use embassy_stm32::eth::lan8742a::LAN8742A;
use embassy_stm32::eth::Ethernet;
use embassy_stm32::rcc::{Config as RccConfig, Rcc};
use embassy_stm32::rng::Random;
use embassy_stm32::time::Hertz;
use embassy_stm32::{interrupt, peripherals, Config};
use heapless::Vec;
use panic_probe as _;
use peripherals::{RNG, TIM2};

defmt::timestamp! {"{=u64}", {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        // NOTE(no-CAS) `timestamps` runs with interrupts disabled
        let n = COUNT.load(Ordering::Relaxed);
        COUNT.store(n + 1, Ordering::Relaxed);
        n as u64
    }
}

#[embassy::task]
async fn main_task(
    device: &'static mut Pin<&'static mut Ethernet<'static, LAN8742A, 4, 4>>,
    config: &'static mut StaticConfigurator,
    spawner: Spawner,
) {
    let net_resources = NET_RESOURCES.put(StackResources::new());

    // Init network stack
    embassy_net::init(device, config, net_resources);

    // Launch network task
    unwrap!(spawner.spawn(net_task()));

    info!("Network task initialized");

    // Then we can use it!
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];
    let mut socket = TcpSocket::new(&mut rx_buffer, &mut tx_buffer);

    socket.set_timeout(Some(embassy_net::SmolDuration::from_secs(10)));

    let remote_endpoint = (Ipv4Address::new(192, 168, 0, 10), 8000);
    let r = socket.connect(remote_endpoint).await;
    if let Err(e) = r {
        info!("connect error: {:?}", e);
        return;
    }
    info!("connected!");
    loop {
        let r = socket.write_all(b"Hello\n").await;
        if let Err(e) = r {
            info!("write error: {:?}", e);
            return;
        }
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy::task]
async fn net_task() {
    embassy_net::run().await
}

#[no_mangle]
fn _embassy_rand(buf: &mut [u8]) {
    use rand_core::RngCore;

    critical_section::with(|_| unsafe {
        unwrap!(RNG_INST.as_mut()).fill_bytes(buf);
    });
}

static mut RNG_INST: Option<Random<RNG>> = None;

static EXECUTOR: Forever<Executor> = Forever::new();
static TIMER_RTC: Forever<Clock<TIM2>> = Forever::new();
static ALARM: Forever<Alarm<TIM2>> = Forever::new();
static ETH: Forever<Ethernet<'static, LAN8742A, 4, 4>> = Forever::new();
static DEVICE: Forever<Pin<&'static mut Ethernet<'static, LAN8742A, 4, 4>>> = Forever::new();
static CONFIG: Forever<StaticConfigurator> = Forever::new();
static NET_RESOURCES: Forever<StackResources<1, 2, 8>> = Forever::new();

#[entry]
fn main() -> ! {
    use stm32_metapac::RCC;

    info!("Hello World!");

    info!("Setup RCC...");
    let mut rcc_config = RccConfig::default();
    rcc_config.sys_ck = Some(Hertz(400_000_000));
    rcc_config.pll1.q_ck = Some(Hertz(100_000_000));
    let config = Config::default().rcc(rcc_config);

    let mut p = embassy_stm32::init(config);

    // Constrain and Freeze clock

    let mut rcc = Rcc::new(&mut p.RCC, RccConfig::default());
    rcc.enable_debug_wfe(&mut p.DBGMCU, true);

    unsafe {
        RCC.ahb4enr().modify(|w| {
            w.set_gpioaen(true);
            w.set_gpioben(true);
            w.set_gpiocen(true);
            w.set_gpioden(true);
            w.set_gpioien(true);
        });
    }

    let rtc_int = interrupt_take!(TIM2);
    let rtc = TIMER_RTC.put(Clock::new(p.TIM2, rtc_int));
    rtc.start();
    let alarm = ALARM.put(rtc.alarm1());

    unsafe { embassy::time::set_clock(rtc) };

    let rng = Random::new(p.RNG);
    unsafe {
        RNG_INST.replace(rng);
    }

    let eth_int = interrupt_take!(ETH);
    let mac_addr = [0x10; 6];
    let eth = ETH.put(Ethernet::new(
        p.ETH, eth_int, p.PA1, p.PA2, p.PC1, p.PA7, p.PC4, p.PC5, p.PB12, p.PB13, p.PB11, LAN8742A,
        mac_addr, 1,
    ));

    // NOTE(unsafe) This thing is a &'static
    let net_device = DEVICE.put(unsafe { Pin::new_unchecked(eth) });
    net_device.as_mut().init();

    let config = StaticConfigurator::new(NetConfig {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 0, 61), 24),
        dns_servers: Vec::new(),
        gateway: Some(Ipv4Address::new(192, 168, 0, 1)),
    });

    let config = CONFIG.put(config);

    let executor = EXECUTOR.put(Executor::new());
    executor.set_alarm(alarm);

    executor.run(move |spawner| {
        unwrap!(spawner.spawn(main_task(net_device, config, spawner)));
    })
}
