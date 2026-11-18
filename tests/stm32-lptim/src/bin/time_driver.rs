//! LPTIM1 time driver on LSE with 32768 Hz ticks: the 16-bit counter wraps every 2 s.
#![no_std]
#![no_main]

use defmt::{assert, info, panic};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rcc::LsConfig;
use embassy_time::{Duration, Instant, Timer};
use panic_probe as _;

#[cfg(feature = "stm32g491re")]
teleprobe_meta::target!(b"nucleo-stm32g491re");
#[cfg(feature = "stm32h563zi")]
teleprobe_meta::target!(b"nucleo-stm32h563zi");
#[cfg(feature = "stm32l073rz")]
teleprobe_meta::target!(b"nucleo-stm32l073rz");
#[cfg(feature = "stm32l496zg")]
teleprobe_meta::target!(b"nucleo-stm32l496zg");
#[cfg(feature = "stm32l4a6zg")]
teleprobe_meta::target!(b"nucleo-stm32l4a6zg");
#[cfg(feature = "stm32l4r5zi")]
teleprobe_meta::target!(b"nucleo-stm32l4r5zi");
#[cfg(feature = "stm32u083rc")]
teleprobe_meta::target!(b"nucleo-stm32u083rc");
#[cfg(feature = "stm32u585ai")]
teleprobe_meta::target!(b"iot-stm32u585ai");
#[cfg(feature = "stm32wba65ri")]
teleprobe_meta::target!(b"nucleo-stm32wba65ri");
#[cfg(feature = "stm32wl55jc")]
teleprobe_meta::target!(b"nucleo-stm32wl55jc");
teleprobe_meta::timeout!(60);

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.ls = LsConfig::default_lse();
    lptim1_on_lse(&mut config);
    #[cfg(feature = "stm32wl55jc")]
    let p = {
        // Not in shared memory, but the second core isn't running.
        static SHARED_DATA: core::mem::MaybeUninit<embassy_stm32::SharedData> = core::mem::MaybeUninit::uninit();
        embassy_stm32::init_primary(config, &SHARED_DATA)
    };
    #[cfg(not(feature = "stm32wl55jc"))]
    let p = embassy_stm32::init(config);
    let sys = embassy_stm32::rcc::clocks(&p.RCC).sys.to_hertz().unwrap().0;
    info!("sys clock {} Hz", sys);

    // now() across counter wraps: never backwards, never a jump.
    let start = Instant::now();
    let mut last = start;
    let mut wraps = 0;
    let mut max_step = 0;
    while last.duration_since(start) < Duration::from_millis(6_500) {
        let now = Instant::now();
        let (a, b) = (last.as_ticks(), now.as_ticks());
        if b < a {
            panic!("time went backwards: {} -> {}", a, b);
        }
        max_step = max_step.max(b - a);
        if (b >> 16) != (a >> 16) {
            wraps += 1;
        }
        last = now;
    }
    info!("now(): {} wraps, max step {} ticks", wraps, max_step);
    assert!(wraps >= 3);
    assert!(max_step < 100);

    // Timers fire at their deadline, including ones that cross counter wraps. With `lp`, the
    // executor pauses time and sleeps or enters Stop between them.
    for ms in [1u64, 10, 100, 1_500, 2_100, 4_300] {
        let t0 = Instant::now();
        Timer::after_millis(ms).await;
        let ticks = t0.elapsed().as_ticks();
        let want = ms * 32_768 / 1_000;
        info!("after {} ms: {} ticks (want >= {})", ms, ticks, want);
        assert!(ticks >= want);
        assert!(ticks <= want + 200);
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

fn lptim1_on_lse(config: &mut Config) {
    use embassy_stm32::rcc::mux;
    #[cfg(any(
        feature = "stm32u083rc",
        feature = "stm32u585ai",
        feature = "stm32wba65ri",
        feature = "stm32wl55jc",
    ))]
    if let Some(lse) = &mut config.rcc.ls.lse {
        lse.peripherals_clocked = true;
    }
    #[cfg(any(
        feature = "stm32l496zg",
        feature = "stm32l4a6zg",
        feature = "stm32l4r5zi",
        feature = "stm32wba65ri",
        feature = "stm32u083rc",
    ))]
    {
        config.rcc.mux.lptim1sel = mux::Lptim1sel::Lse;
    }
    #[cfg(not(any(
        feature = "stm32l496zg",
        feature = "stm32l4a6zg",
        feature = "stm32l4r5zi",
        feature = "stm32wba65ri",
        feature = "stm32u083rc",
    )))]
    {
        config.rcc.mux.lptim1sel = mux::Lptimsel::Lse;
    }
}
