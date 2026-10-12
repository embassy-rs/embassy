#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::{assert_eq, info, panic};
use defmt_rtt as _;
use embassy_embedded_hal::SetConfig;
use embassy_executor::Spawner;
use embassy_rp::clocks::{PllConfig, XoscConfig};
use embassy_rp::config::Config as rpConfig;
use embassy_rp::peripherals::{I2C0, I2C1};
use embassy_rp::time::Hertz;
use embassy_rp::{bind_interrupts, i2c, i2c_slave, mode};
use embedded_hal_1::i2c::Operation;
use panic_probe as _;

use crate::i2c::AbortReason;

fn dump_i2c(tag: &str) {
    use embassy_rp::pac;
    let v = pac::SIO.gpio_in(0).read();
    info!(
        "DIAG {}: GP18(sda1)={} GP19(scl1)={} GP20(sda0)={} GP21(scl0)={}",
        tag,
        (v >> 18) & 1,
        (v >> 19) & 1,
        (v >> 20) & 1,
        (v >> 21) & 1
    );
    macro_rules! regs {
        ($name:expr, $r:expr) => {{
            let r = $r;
            info!(
                "DIAG {} {}: con={:#x} tar={:#x} sar={:#x} enable={:#x} en_status={:#x} status={:#x} raw_intr={:#x} mask={:#x} abrt={:#x} txflr={} rxflr={} hcnt={} lcnt={}",
                tag,
                $name,
                r.ic_con().read().0,
                r.ic_tar().read().0,
                r.ic_sar().read().0,
                r.ic_enable().read().0,
                r.ic_enable_status().read().0,
                r.ic_status().read().0,
                r.ic_raw_intr_stat().read().0,
                r.ic_intr_mask().read().0,
                r.ic_tx_abrt_source().read().0,
                r.ic_txflr().read().0,
                r.ic_rxflr().read().0,
                r.ic_fs_scl_hcnt().read().0,
                r.ic_fs_scl_lcnt().read().0,
            );
        }};
    }
    regs!("I2C0(ctrl)", pac::I2C0);
    regs!("I2C1(slave)", pac::I2C1);
    use cortex_m::peripheral::NVIC;
    info!(
        "DIAG {} nvic: I2C0 enabled={} pending={} | I2C1 enabled={} pending={}",
        tag,
        NVIC::is_enabled(pac::Interrupt::I2C0_IRQ),
        NVIC::is_pending(pac::Interrupt::I2C0_IRQ),
        NVIC::is_enabled(pac::Interrupt::I2C1_IRQ),
        NVIC::is_pending(pac::Interrupt::I2C1_IRQ),
    );
}

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
    I2C1_IRQ => i2c::InterruptHandler<I2C1>;
});

const DEV_ADDR: u8 = 0x42;

#[embassy_executor::task]
async fn device_task(mut dev: i2c_slave::I2cSlave<'static, mode::Async>) -> ! {
    info!("Device start");

    let mut count = 0xD0;

    loop {
        let mut buf = [0u8; 128];
        match dev.listen(&mut buf).await {
            Ok(i2c_slave::Command::GeneralCall(len)) => {
                assert_eq!(buf[..len], [0xCA, 0x11], "recieving the general call failed");
                info!("General Call - OK");
            }
            Ok(i2c_slave::Command::Read) => {
                loop {
                    match dev.respond_to_read(&[count]).await {
                        Ok(x) => match x {
                            i2c_slave::ReadStatus::Done => break,
                            i2c_slave::ReadStatus::NeedMoreBytes => count += 1,
                            i2c_slave::ReadStatus::LeftoverBytes(x) => panic!("tried to write {} extra bytes", x),
                        },
                        Err(e) => match e {
                            embassy_rp::i2c_slave::Error::Abort(AbortReason::Other(n)) => panic!("Other {:b}", n),
                            _ => panic!("{}", e),
                        },
                    }
                }
                count += 1;
            }
            Ok(i2c_slave::Command::Write(len)) => match len {
                1 => {
                    assert_eq!(buf[..len], [0xAA], "recieving a single byte failed");
                    info!("Single Byte Write - OK")
                }
                4 => {
                    assert_eq!(buf[..len], [0xAA, 0xBB, 0xCC, 0xDD], "recieving 4 bytes failed");
                    info!("4 Byte Write - OK")
                }
                32 => {
                    assert_eq!(
                        buf[..len],
                        [
                            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
                            25, 26, 27, 28, 29, 30, 31
                        ],
                        "recieving 32 bytes failed"
                    );
                    info!("32 Byte Write - OK")
                }
                _ => panic!("Invalid write length {}", len),
            },
            Ok(i2c_slave::Command::WriteRead(len)) => {
                info!("device received write read: {:x}", buf[..len]);
                match buf[0] {
                    0xC2 => {
                        let resp_buff = [0xD1, 0xD2, 0xD3, 0xD4];
                        dev.respond_to_read(&resp_buff).await.unwrap();
                    }
                    0xC8 => {
                        let mut resp_buff = [0u8; 32];
                        for i in 0..32 {
                            resp_buff[i] = i as u8;
                        }
                        dev.respond_to_read(&resp_buff).await.unwrap();
                        // reset count for next round of tests
                        count = 0xD0;
                    }
                    x => panic!("Invalid Write Read {:x}", x),
                }
            }
            Err(e) => match e {
                embassy_rp::i2c_slave::Error::Abort(AbortReason::Other(n)) => panic!("Other {:b}", n),
                _ => panic!("{}", e),
            },
        }
    }
}

// ---- temporary diagnostic battery ----

/// Pulse the reset of both I2C blocks so each experiment starts from a clean peripheral.
fn reset_i2c() {
    use embassy_rp::pac;
    pac::RESETS.reset().modify(|w| {
        w.set_i2c0(true);
        w.set_i2c1(true);
    });
    pac::RESETS.reset().modify(|w| {
        w.set_i2c0(false);
        w.set_i2c1(false);
    });
    while {
        let d = pac::RESETS.reset_done().read();
        !(d.i2c0() && d.i2c1())
    } {}
}

/// Reset one I2C block only, to see which one is holding the bus.
fn reset_block(i2c0: bool, i2c1: bool) {
    use embassy_rp::pac;
    pac::RESETS.reset().modify(|w| {
        if i2c0 {
            w.set_i2c0(true);
        }
        if i2c1 {
            w.set_i2c1(true);
        }
    });
    pac::RESETS.reset().modify(|w| {
        if i2c0 {
            w.set_i2c0(false);
        }
        if i2c1 {
            w.set_i2c1(false);
        }
    });
    while {
        let d = pac::RESETS.reset_done().read();
        (i2c0 && !d.i2c0()) || (i2c1 && !d.i2c1())
    } {}
}

/// bit0 = SDA0 (GP20), bit1 = SCL0 (GP21), bit2 = SDA1 (GP18), bit3 = SCL1 (GP19)
fn levels() -> u8 {
    let v = embassy_rp::pac::SIO.gpio_in(0).read();
    (((v >> 20) & 1) | (((v >> 21) & 1) << 1) | (((v >> 18) & 1) << 2) | (((v >> 19) & 1) << 3)) as u8
}

async fn slave_loop(dev: &mut i2c_slave::I2cSlave<'_, mode::Async>) -> core::convert::Infallible {
    loop {
        let mut buf = [0u8; 64];
        match dev.listen(&mut buf).await {
            Ok(i2c_slave::Command::Read) => {
                dev.respond_and_fill(&[0xD0], 0xD0).await.ok();
            }
            Ok(i2c_slave::Command::WriteRead(_)) => {
                dev.respond_and_fill(&[0xD1], 0xD1).await.ok();
            }
            Ok(_) => {}
            Err(_) => embassy_futures::yield_now().await,
        }
    }
}

/// Three bounded operations. Returns true if all completed.
async fn run_exp(name: &str, ctrl: &mut i2c::I2c<'_, mode::Async>) -> bool {
    use embassy_futures::select::{Either, select};
    use embassy_time::{Duration, Instant, Timer, with_timeout};

    let mut ok = true;
    info!("EXP {} start levels={:x}", name, levels());

    // 1. general call, with the bus sampled every millisecond while it runs
    let mut samples = [0xFFu8; 40];
    let t = Instant::now();
    let r = {
        let op = with_timeout(Duration::from_millis(500), ctrl.write(0u8, &[0xCA, 0x11]));
        let sampler = async {
            for s in samples.iter_mut() {
                *s = levels();
                Timer::after_micros(1000).await;
            }
            core::future::pending::<()>().await
        };
        match select(op, sampler).await {
            Either::First(r) => r,
            Either::Second(_) => unreachable!(),
        }
    };
    let us = t.elapsed().as_micros();
    info!(
        "EXP {} bus samples @1ms (bit0 sda0, bit1 scl0, bit2 sda1, bit3 scl1): {:x}",
        name, samples
    );
    match r {
        Ok(Ok(())) => info!("RESULT {} general-call: OK {}us", name, us),
        Ok(Err(e)) => {
            ok = false;
            info!("RESULT {} general-call: ERR {:?} {}us", name, e, us);
        }
        Err(_) => {
            info!("RESULT {} general-call: HANG {}us", name, us);
            dump_i2c(name);
            // Who holds the bus? Reset the blocks one at a time and look at the lines.
            info!("RELEASE {} before: levels={:x}", name, levels());
            reset_block(false, true);
            Timer::after_millis(2).await;
            info!(
                "RELEASE {} after resetting slave block I2C1: levels={:x}",
                name,
                levels()
            );
            reset_block(true, false);
            Timer::after_millis(2).await;
            info!(
                "RELEASE {} after also resetting controller block I2C0: levels={:x}",
                name,
                levels()
            );
            info!(
                "EXP {} end levels={:x} ok=false (skipped ops 2 and 3 after hang)",
                name,
                levels()
            );
            return false;
        }
    }

    // 2. one-byte write to the slave
    let t = Instant::now();
    match with_timeout(Duration::from_millis(500), ctrl.write(DEV_ADDR, &[0xAA])).await {
        Ok(Ok(())) => info!("RESULT {} write: OK {}us", name, t.elapsed().as_micros()),
        Ok(Err(e)) => {
            ok = false;
            info!("RESULT {} write: ERR {:?} {}us", name, e, t.elapsed().as_micros());
        }
        Err(_) => {
            ok = false;
            info!("RESULT {} write: HANG {}us", name, t.elapsed().as_micros());
        }
    }

    // 3. one-byte read from the slave
    let t = Instant::now();
    let mut b = [0u8; 1];
    match with_timeout(Duration::from_millis(500), ctrl.read(DEV_ADDR, &mut b)).await {
        Ok(Ok(())) => info!("RESULT {} read: OK {:x} {}us", name, b, t.elapsed().as_micros()),
        Ok(Err(e)) => {
            ok = false;
            info!("RESULT {} read: ERR {:?} {}us", name, e, t.elapsed().as_micros());
        }
        Err(_) => {
            ok = false;
            info!("RESULT {} read: HANG {}us", name, t.elapsed().as_micros());
        }
    }

    info!("EXP {} end levels={:x} ok={}", name, levels(), ok);
    ok
}

/// What is actually connected: idle levels, a continuity matrix and pull-up rise times.
async fn gpio_survey(p: &mut embassy_rp::Peripherals) {
    use embassy_rp::gpio::{Flex, Pull};
    use embassy_time::{Instant, Timer};

    let mut f = [
        Flex::new(p.PIN_18.reborrow()),
        Flex::new(p.PIN_19.reborrow()),
        Flex::new(p.PIN_20.reborrow()),
        Flex::new(p.PIN_21.reborrow()),
    ];
    for x in f.iter_mut() {
        x.set_pull(Pull::None);
        x.set_as_input();
    }
    Timer::after_millis(2).await;
    info!(
        "SURVEY idle, no internal pull: GP18={} GP19={} GP20={} GP21={}",
        f[0].is_high(),
        f[1].is_high(),
        f[2].is_high(),
        f[3].is_high()
    );

    // Continuity: internal pull-ups on the pins not being driven, so a bridged pin follows the driven one down.
    for d in 0..4 {
        for x in f.iter_mut() {
            x.set_as_input();
            x.set_pull(Pull::Up);
        }
        Timer::after_millis(1).await;
        f[d].set_low();
        f[d].set_as_output();
        Timer::after_micros(500).await;
        info!(
            "SURVEY continuity drive GP{} low (others pulled up): GP18={} GP19={} GP20={} GP21={}",
            18 + d,
            f[0].is_high(),
            f[1].is_high(),
            f[2].is_high(),
            f[3].is_high()
        );
        f[d].set_as_input();
        Timer::after_millis(2).await;
    }

    // Rise time after release, first with no internal pull (external only), then with the internal pull-up.
    for (pull, label) in [(Pull::None, "no-internal"), (Pull::Up, "internal-up")] {
        for d in 0..4 {
            for x in f.iter_mut() {
                x.set_as_input();
                x.set_pull(pull);
            }
            f[d].set_low();
            f[d].set_as_output();
            Timer::after_micros(500).await;
            f[d].set_as_input();
            let t = Instant::now();
            while f[d].is_low() && t.elapsed().as_micros() < 5000 {}
            info!(
                "SURVEY rise GP{} {}: {}us (5001+ = never)",
                18 + d,
                label,
                t.elapsed().as_micros()
            );
            Timer::after_millis(2).await;
        }
    }

    for x in f.iter_mut() {
        x.set_pull(Pull::Down);
    }
    Timer::after_millis(2).await;
    info!(
        "SURVEY internal pull-down only (high = external pull-up wins): GP18={} GP19={} GP20={} GP21={}",
        f[0].is_high(),
        f[1].is_high(),
        f[2].is_high(),
        f[3].is_high()
    );
}

async fn controller_task(con: &mut i2c::I2c<'static, mode::Async>) {
    info!("Controller start");

    {
        let buf = [0xCA, 0x11];
        dump_i2c("before first op");
        match embassy_time::with_timeout(embassy_time::Duration::from_secs(2), con.write(0u8, &buf)).await {
            Ok(r) => r.unwrap(),
            Err(_) => {
                dump_i2c("first op timed out");
                embassy_time::Timer::after_millis(100).await;
                dump_i2c("100ms later");
                panic!("DIAG: first general call never completed");
            }
        }
        info!("Controler general call write");
        embassy_futures::yield_now().await;
    }

    {
        let mut buf = [0u8];
        con.read(DEV_ADDR, &mut buf).await.unwrap();
        assert_eq!(buf, [0xD0], "single byte read failed");
        info!("single byte read - OK");
        embassy_futures::yield_now().await;
    }

    {
        let mut buf = [0u8; 4];
        con.read(DEV_ADDR, &mut buf).await.unwrap();
        assert_eq!(buf, [0xD1, 0xD2, 0xD3, 0xD4], "single byte read failed");
        info!("4 byte read - OK");
        embassy_futures::yield_now().await;
    }

    {
        let buf = [0xAA];
        con.write(DEV_ADDR, &buf).await.unwrap();
        info!("Controler single byte write");
        embassy_futures::yield_now().await;
    }

    {
        let buf = [0xAA, 0xBB, 0xCC, 0xDD];
        con.write(DEV_ADDR, &buf).await.unwrap();
        info!("Controler 4 byte write");
        embassy_futures::yield_now().await;
    }

    {
        let mut buf = [0u8; 32];
        for i in 0..32 {
            buf[i] = i as u8;
        }
        con.write(DEV_ADDR, &buf).await.unwrap();
        info!("Controler 32 byte write");
        embassy_futures::yield_now().await;
    }

    {
        let mut buf = [0u8; 4];
        let mut ops = [Operation::Write(&[0xC2]), Operation::Read(&mut buf)];
        con.transaction(DEV_ADDR, &mut ops).await.unwrap();
        assert_eq!(buf, [0xD1, 0xD2, 0xD3, 0xD4], "write_read failed");
        info!("write_read - OK");
        embassy_futures::yield_now().await;
    }

    {
        let mut buf = [0u8; 32];
        let mut ops = [Operation::Write(&[0xC8]), Operation::Read(&mut buf)];
        con.transaction(DEV_ADDR, &mut ops).await.unwrap();
        assert_eq!(
            buf,
            [
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
                28, 29, 30, 31
            ],
            "write_read of 32 bytes failed"
        );
        info!("large write_read - OK")
    }

    #[embassy_executor::main]
    async fn main(spawner: Spawner) {
        let mut config = rpConfig::default();
        // Configure clk_sys to 48MHz to support 1kHz scl.
        // In theory it can go lower, but we won't bother to test below 1kHz.
        config.clocks.xosc = Some(XoscConfig {
            hz: 12_000_000,
            delay_multiplier: 128,
            sys_pll: Some(PllConfig {
                refdiv: 1,
                fbdiv: 120,
                post_div1: 6,
                post_div2: 5,
            }),
            usb_pll: Some(PllConfig {
                refdiv: 1,
                fbdiv: 120,
                post_div1: 6,
                post_div2: 5,
            }),
        });

        let mut p = embassy_rp::init(config);
        info!("Hello World!");
        info!(
            "DIAG clk_sys={}Hz clk_peri={}Hz",
            embassy_rp::clocks::clk_sys_freq(),
            embassy_rp::clocks::clk_peri_freq()
        );

        gpio_survey(&mut p).await;

        let mut failed = 0u32;
        macro_rules! exp_tail {
            ($name:expr, $dev:ident, $ctrl:ident, $freq:expr, $pu:expr, $via:expr) => {{
                if $via {
                    let mut c = i2c::Config::default();
                    c.frequency = Hertz($freq);
                    c.sda_pullup = $pu;
                    c.scl_pullup = $pu;
                    $ctrl.set_config(&c).unwrap();
                }
                let ok = match $dev.as_mut() {
                    Some(d) => match embassy_futures::select::select(slave_loop(d), run_exp($name, &mut $ctrl)).await {
                        embassy_futures::select::Either::Second(ok) => ok,
                        embassy_futures::select::Either::First(never) => match never {},
                    },
                    None => run_exp($name, &mut $ctrl).await,
                };
                drop($ctrl);
                drop($dev);
                if !ok {
                    failed += 1;
                }
            }};
        }
        macro_rules! exp {
            ($name:expr, swap = false, slave = $slave:expr, freq = $freq:expr, pullups = $pu:expr, viacfg = $via:expr) => {{
                reset_i2c();
                let mut cfg = i2c::Config::default();
                cfg.sda_pullup = $pu;
                cfg.scl_pullup = $pu;
                if !$via {
                    cfg.frequency = Hertz($freq);
                }
                let mut scfg = i2c_slave::Config::default();
                scfg.addr = DEV_ADDR as u16;
                let mut dev = if $slave {
                    Some(i2c_slave::I2cSlave::new(
                        p.I2C1.reborrow(),
                        p.PIN_19.reborrow(),
                        p.PIN_18.reborrow(),
                        Irqs,
                        scfg,
                    ))
                } else {
                    None
                };
                let mut ctrl = i2c::I2c::new(
                    p.I2C0.reborrow(),
                    p.PIN_21.reborrow(),
                    p.PIN_20.reborrow(),
                    Irqs,
                    cfg,
                );
                exp_tail!($name, dev, ctrl, $freq, $pu, $via);
            }};
            ($name:expr, swap = true, slave = $slave:expr, freq = $freq:expr, pullups = $pu:expr, viacfg = $via:expr) => {{
                reset_i2c();
                let mut cfg = i2c::Config::default();
                cfg.sda_pullup = $pu;
                cfg.scl_pullup = $pu;
                if !$via {
                    cfg.frequency = Hertz($freq);
                }
                let mut scfg = i2c_slave::Config::default();
                scfg.addr = DEV_ADDR as u16;
                let mut dev = if $slave {
                    Some(i2c_slave::I2cSlave::new(
                        p.I2C0.reborrow(),
                        p.PIN_21.reborrow(),
                        p.PIN_20.reborrow(),
                        Irqs,
                        scfg,
                    ))
                } else {
                    None
                };
                let mut ctrl = i2c::I2c::new(
                    p.I2C1.reborrow(),
                    p.PIN_19.reborrow(),
                    p.PIN_18.reborrow(),
                    Irqs,
                    cfg,
                );
                exp_tail!($name, dev, ctrl, $freq, $pu, $via);
            }};
        }

        // The configuration CI uses: slave first, controller built at the default frequency, then set_config.
        exp!(
            "A 1k slave via-set_config pullups-on",
            swap = false,
            slave = true,
            freq = 1000,
            pullups = true,
            viacfg = true
        );
        exp!(
            "B 1k slave direct-config pullups-on",
            swap = false,
            slave = true,
            freq = 1000,
            pullups = true,
            viacfg = false
        );
        exp!(
            "C 1k NO-slave via-set_config",
            swap = false,
            slave = false,
            freq = 1000,
            pullups = true,
            viacfg = true
        );
        exp!(
            "D 1k slave via-set_config pullups-off(external only)",
            swap = false,
            slave = true,
            freq = 1000,
            pullups = false,
            viacfg = true
        );
        exp!(
            "E 1k slave swapped-roles via-set_config",
            swap = true,
            slave = true,
            freq = 1000,
            pullups = true,
            viacfg = true
        );
        exp!(
            "F 10k slave via-set_config",
            swap = false,
            slave = true,
            freq = 10_000,
            pullups = true,
            viacfg = true
        );
        exp!(
            "G 100k slave via-set_config",
            swap = false,
            slave = true,
            freq = 100_000,
            pullups = true,
            viacfg = true
        );
        exp!(
            "H 400k slave via-set_config",
            swap = false,
            slave = true,
            freq = 400_000,
            pullups = true,
            viacfg = true
        );
        exp!(
            "I 1M slave via-set_config",
            swap = false,
            slave = true,
            freq = 1_000_000,
            pullups = true,
            viacfg = true
        );
        exp!(
            "J 100k NO-slave via-set_config",
            swap = false,
            slave = false,
            freq = 100_000,
            pullups = true,
            viacfg = true
        );
        exp!(
            "K 1k slave via-set_config pullups-on (repeat)",
            swap = false,
            slave = true,
            freq = 1000,
            pullups = true,
            viacfg = true
        );
        info!("DIAG SUMMARY: {} of 11 experiments had a failed operation", failed);
        reset_i2c();

        let d_sda = p.PIN_19;
        let d_scl = p.PIN_18;
        let mut config = i2c_slave::Config::default();
        config.addr = DEV_ADDR as u16;
        let device = i2c_slave::I2cSlave::new(p.I2C1, d_sda, d_scl, Irqs, config);

        spawner.spawn(device_task(device).unwrap());

        let c_sda = p.PIN_21;
        let c_scl = p.PIN_20;
        let mut controller = i2c::I2c::new(p.I2C0, c_sda, c_scl, Irqs, Default::default());

        for freq in [1000, 100_000, 400_000, 1_000_000] {
            info!("testing at {}hz", freq);
            let mut config = i2c::Config::default();
            config.frequency = Hertz(freq);
            controller.set_config(&config).unwrap();
            controller_task(&mut controller).await;
        }

        info!("Test OK");
        cortex_m::asm::bkpt();
    }
}
