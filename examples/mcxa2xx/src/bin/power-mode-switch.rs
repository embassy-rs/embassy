//! Interactive FRDM-MCXA266 power-mode switch demonstration.
//!
//! Ports the supplied NXP `power_mode_switch.c`, `board/app.h`, and pin setup.
//! LPUART2 TX=P2_2/RX=P2_3 provides a 115200-baud, 8-N-1 menu through MCU-Link
//! VCOM. SW2 uses P1_7/WUU input 9; timed wake uses WAKETIMER0.
//! UART byte writes, blocking reads, flushing, and command echo follow the
//! MCXA5xx example. RTT carries only binary defmt diagnostics.
//! Active mode uses the 180 MHz PLL. Sleep and Deep Sleep use the checked HAL
//! entry/recovery routines; Power Down and Deep Power Down are not supported.
//!
//! Build and program from `examples\mcxa2xx`:
//!
//! ```powershell
//! cargo build --release --no-default-features --features executor-platform --bin power-mode-switch
//! probe-rs run --chip MCXA266 target\thumbv8m.main-none-eabihf\release\power-mode-switch
//! ```
//!
//! The explicit chip overrides this package's configured MCXA276 runner.
//! Disconnect the debugger for current measurements. A full power cycle may
//! be required if earlier firmware locked FRO16K off.
//!
//! No tasks or Embassy timers are awaited while peripherals are suspended.
//! UART and OSTIMER interrupts are quiesced before entry and restored after
//! clock recovery, including rejected entry attempts.

#![no_std]
#![no_main]

use core::fmt::{self, Write};

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use hal::clocks::PoweredClock;
use hal::clocks::config::{
    CoreSleep, Div8, FlashSleep, MainClockConfig, MainClockSource, SpllConfig, SpllMode, SpllSource, VddDriveStrength,
    VddLevel,
};
use hal::gpio::Pull;
use hal::interrupt::InterruptExt;
use hal::lpuart::{Blocking, Config as UartConfig, Lpuart, LpuartRx, LpuartTx};
use hal::pac::scg::{SirccsrLk, Sircerr, Sircvld};
use hal::wuu::{Edge, Event, ExternalPinConfig, InternalModule, PinMode, WakeupPin, Wuu};
use hal::{bind_interrupts, interrupt};
use panic_probe as _;

const DEBUG_RECOVERY_DELAY_CYCLES: u32 = 90_000_000;
const USB_ISOLATION_MASK: u8 = 0x2;
const SDK_LOW_POWER_WAKE_DELAY: u16 = 0x7d;

// WAKETIMER0 is described by SDK PERI_WAKETIMER.h and RM chapter 30.
// It has no register API in the current nxp-pac.
const WAKE_TIMER_CONTROL: *mut u32 = 0x400A_E000 as *mut u32;
const WAKE_TIMER_COUNT: *mut u32 = 0x400A_E00C as *mut u32;
const WAKE_FLAG: u32 = 1 << 1;
const CLEAR_WAKE_TIMER: u32 = 1 << 2;
const OSC_DIVIDE_ENABLE: u32 = 1 << 4;
const WAKE_INTERRUPT_ENABLE: u32 = 1 << 5;
// The mandatory four-bit divider turns the HAL's 16.384 kHz FRO into 1024 Hz.
const WAKE_TIMER_TICKS_PER_SECOND: u32 = 16_384 / 16;

bind_interrupts!(pub struct Irqs {
    WAKETIMER0 => WakeTimerInterruptHandler;
    WUU0 => hal::wuu::InterruptHandler;
});

struct WakeTimerInterruptHandler;

impl interrupt::typelevel::Handler<interrupt::typelevel::WAKETIMER0> for WakeTimerInterruptHandler {
    unsafe fn on_interrupt() {
        // One-shot wake: acknowledge WAKE_FLAG and disable further interrupts.
        write_wake_control(OSC_DIVIDE_ENABLE | WAKE_FLAG);
    }
}

#[derive(Clone, Copy, PartialEq)]
enum PowerMode {
    Active,
    Sleep,
    DeepSleep,
}

impl PowerMode {
    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            b'A' => Some(Self::Active),
            b'B' => Some(Self::Sleep),
            b'C' => Some(Self::DeepSleep),
            _ => None,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Active => "Active: core and system clocks remain running.\r\n",
            Self::Sleep => "Sleep: the core clock is gated; system and bus clocks remain on.\r\n",
            Self::DeepSleep => "Deep Sleep: core, system, and bus clocks are gated.\r\n",
        }
    }
}

#[derive(Clone, Copy)]
enum WakeSource {
    Timer(u8),
    Button,
}

struct Console<'d> {
    tx: LpuartTx<'d, Blocking>,
    rx: LpuartRx<'d, Blocking>,
}

impl Write for Console<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.write_bytes(text.as_bytes());
        Ok(())
    }
}

impl<'d> Console<'d> {
    fn print(&mut self, args: fmt::Arguments<'_>) {
        if self.write_fmt(args).is_err() {
            fatal("Console formatting/write failed");
        }
    }

    fn write(&mut self, text: &str) {
        self.write_bytes(text.as_bytes());
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        if let Err(error) = self.tx.blocking_write(bytes) {
            defmt::error!("Console write failed: {:?}", error);
            fatal("Console write failed");
        }
    }

    fn flush(&mut self) {
        if self.tx.blocking_flush().is_err() {
            fatal("Console flush failed");
        }
    }

    fn read_byte(&mut self) -> u8 {
        let mut buffer = [0];
        if let Err(error) = self.rx.blocking_read(&mut buffer) {
            defmt::error!("Console read failed: {:?}", error);
            fatal("Console read failed");
        }
        let [byte] = buffer;
        byte
    }

    fn read_choice(&mut self) -> u8 {
        loop {
            let byte = self.read_byte();
            if byte == b'\r' || byte == b'\n' {
                continue;
            }
            let choice = byte.to_ascii_uppercase();
            self.write_bytes(&[choice]);
            self.write("\r\n");
            return choice;
        }
    }
}

struct SuspendedPeripherals {
    uart_transmit_enabled: bool,
    uart_receive_enabled: bool,
    ostimer_interrupt_enabled: bool,
    ostimer_nvic_enabled: bool,
    sirc_stop: Option<(bool, SirccsrLk)>,
}

#[cfg_attr(
    feature = "executor-platform",
    embassy_executor::main(executor = "embassy_mcxa::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "executor-platform"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    cortex_m::asm::delay(DEBUG_RECOVERY_DELAY_CYCLES);
    let reset_reason = hal::reset_reason::reset_reason();
    let mut config = hal::config::Config::default();
    config.clock_cfg.sirc.fro_12m_enabled = true;
    config.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    config.clock_cfg.sirc.power = PoweredClock::AlwaysEnabled;
    config.clock_cfg.spll = Some(SpllConfig {
        source: SpllSource::Firc,
        mode: SpllMode::Mode1b {
            m_mult: 8,
            p_div: 1,
            bypass_p2_div: false,
        },
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        pll1_clk_div: None,
    });
    config.clock_cfg.main_clock = MainClockConfig {
        source: MainClockSource::SPll1,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        ahb_clk_div: Div8::no_div(),
    };
    config.clock_cfg.vdd_power.active_mode.level = VddLevel::OverDriveMode;
    config.clock_cfg.vdd_power.active_mode.drive = VddDriveStrength::Normal;
    config.clock_cfg.vdd_power.low_power_mode.level = VddLevel::MidDriveMode;
    config.clock_cfg.vdd_power.low_power_mode.drive = VddDriveStrength::Low { enable_bandgap: false };
    config.clock_cfg.vdd_power.core_sleep = CoreSleep::DeepSleep;
    config.clock_cfg.vdd_power.flash_sleep = FlashSleep::FlashDozeWithFlashWake;
    let p = hal::init(config);
    let core_clock_hz = match hal::clocks::with_clocks(|clocks| {
        clocks.cpu_system_clk.as_ref().map(|clock| clock.frequency)
    })
    .flatten()
    {
        Some(frequency) => frequency,
        None => fatal("System clock descriptor unavailable"),
    };
    hal::pac::SPC0
        .lpwkup_delay()
        .write(|w| w.set_lpwkup_delay(SDK_LOW_POWER_WAKE_DELAY));

    let uart_config = UartConfig {
        baudrate_bps: 115_200,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        ..Default::default()
    };
    let uart = match Lpuart::new_blocking(p.LPUART2, p.P2_2, p.P2_3, uart_config) {
        Ok(uart) => uart,
        Err(_) => fatal("LPUART2 initialization failed"),
    };
    let (tx, rx) = uart.split();
    let mut console = Console { tx, rx };
    let mut button = WakeupPin::new(p.P1_7, Pull::Up);
    let mut wuu = match Wuu::new(p.WUU0, Irqs) {
        Ok(wuu) => wuu,
        Err(_) => fatal("WUU initialization failed"),
    };
    let _wake_timer = p.WAKETIMER0;
    disable_wake_timer();
    console.write("\r\nNormal boot. Reset cause:");
    for reason in reset_reason {
        console.print(format_args!(" {:?}", reason));
    }
    console.write("\r\n");

    loop {
        console.print(format_args!(
            "\r\n################ Power Mode Switch Demo (Rust, MCXA266) ################\r\n\
             Core clock = {} Hz\r\nPower mode: Active\r\n",
            core_clock_hz
        ));
        let mode = select_power_mode(&mut console);
        console.write(mode.description());
        if mode == PowerMode::Active {
            continue;
        }
        let wake_source = select_wake_source(&mut console);
        disable_wake_timer();
        clear_stale_event();
        let configured_wake = match wake_source {
            WakeSource::Timer(_) => wuu.enable_internal_module(InternalModule::WakeTimer),
            WakeSource::Button => wuu.enable_external_pin(
                &mut button,
                ExternalPinConfig {
                    edge: Edge::Falling,
                    event: Event::Interrupt,
                    mode: PinMode::AnyPower,
                },
            ),
        };
        let isolation = if mode == PowerMode::DeepSleep {
            USB_ISOLATION_MASK
        } else {
            0
        };
        hal::pac::SPC0.evd_cfg().write(|w| {
            w.set_evdiso(isolation);
            w.set_evdlpiso(isolation);
        });
        if isolation != 0 {
            console.write("Isolate power domain: VDD_USB.\r\n");
        }
        match wake_source {
            WakeSource::Timer(seconds) => console.print(format_args!("Wake timer selected: {} seconds.\r\n", seconds)),
            WakeSource::Button => console.write("Press SW2 to wake the device.\r\n"),
        }
        console.write("Entering selected mode...\r\n");
        let suspended = suspend_peripherals(&mut console, mode);
        if let WakeSource::Timer(seconds) = wake_source {
            arm_wake_timer(seconds);
        }
        let result = enter_power_mode(mode);
        disable_wake_timer();
        resume_peripherals(suspended);
        hal::pac::SPC0.evd_cfg().write(|w| {
            w.set_evdiso(0);
            w.set_evdlpiso(0);
        });
        drop(configured_wake);

        match result {
            Ok(status) => {
                console.write("Returned to Active mode.\r\n");
                print_sleep_status(&mut console, status);
                if !status.core_clock_was_gated() {
                    console
                        .write("No clock gating recorded: a pending event or debugger may have prevented entry.\r\n");
                }
            }
            Err(error) => {
                defmt::error!("Power mode entry/recovery failed: {:?}", error);
                console.print(format_args!("Power mode entry/recovery failed: {:?}\r\n", error));
            }
        }
    }
}

fn select_power_mode(console: &mut Console<'_>) -> PowerMode {
    loop {
        console.write(
            "\r\nSelect operation:\r\n\
             \tA: Active\r\n\tB: Sleep\r\n\tC: Deep Sleep\r\n\
             Waiting for power mode selection...\r\n",
        );
        if let Some(mode) = PowerMode::from_byte(console.read_choice()) {
            return mode;
        }
        console.write("Wrong input. Power Down modes are not supported.\r\n");
    }
}

fn select_wake_source(console: &mut Console<'_>) -> WakeSource {
    loop {
        console.write("\r\nSelect wake source:\r\n\tA: Wake timer\r\n\tB: SW2 button\r\n");
        match console.read_choice() {
            b'A' => loop {
                console.write("Wake timeout in seconds (1-9):\r\n");
                match console.read_choice() {
                    byte @ b'1'..=b'9' => return WakeSource::Timer(byte - b'0'),
                    _ => console.write("Wrong timeout.\r\n"),
                }
            },
            b'B' => return WakeSource::Button,
            _ => console.write("Wrong wake source.\r\n"),
        }
    }
}

fn suspend_peripherals(console: &mut Console<'_>, mode: PowerMode) -> SuspendedPeripherals {
    console.flush();
    let uart_control = hal::pac::LPUART2.ctrl().read();
    hal::pac::LPUART2.ctrl().modify(|w| {
        w.set_te(false);
        w.set_re(false);
    });
    let ostimer_interrupt_enabled = hal::pac::OSTIMER0.osevent_ctrl().read().ostimer_intena();
    let ostimer_nvic_enabled = interrupt::OS_EVENT.is_enabled();
    interrupt::OS_EVENT.disable();
    hal::pac::OSTIMER0.osevent_ctrl().modify(|w| {
        w.set_ostimer_intena(false);
        w.set_ostimer_intrflag(true);
    });
    interrupt::OS_EVENT.unpend();
    let sirc_stop = if mode == PowerMode::DeepSleep {
        let sirc = hal::pac::SCG0.sirccsr().read();
        set_sirc_stop(false, sirc.lk());
        Some((sirc.sircsten(), sirc.lk()))
    } else {
        None
    };
    SuspendedPeripherals {
        uart_transmit_enabled: uart_control.te(),
        uart_receive_enabled: uart_control.re(),
        ostimer_interrupt_enabled,
        ostimer_nvic_enabled,
        sirc_stop,
    }
}

fn resume_peripherals(state: SuspendedPeripherals) {
    if let Some((enabled, lock)) = state.sirc_stop {
        set_sirc_stop(enabled, lock);
    }
    while hal::pac::SCG0.sirccsr().read().sircvld() != Sircvld::EnabledAndValid {}
    hal::pac::LPUART2.ctrl().modify(|w| {
        w.set_te(state.uart_transmit_enabled);
        w.set_re(state.uart_receive_enabled);
    });
    interrupt::OS_EVENT.unpend();
    hal::pac::OSTIMER0.osevent_ctrl().modify(|w| {
        w.set_ostimer_intrflag(true);
        w.set_ostimer_intena(state.ostimer_interrupt_enabled);
    });
    if state.ostimer_nvic_enabled {
        // SAFETY: HAL initialization installed the OSTIMER time-driver handler.
        unsafe { interrupt::OS_EVENT.enable() };
    }
}

fn set_sirc_stop(enabled: bool, lock: SirccsrLk) {
    hal::pac::SCG0.sirccsr().modify(|w| {
        w.set_lk(SirccsrLk::WriteEnabled);
        w.set_sircerr(Sircerr::ErrorNotDetected);
    });
    hal::pac::SCG0.sirccsr().modify(|w| {
        w.set_sircsten(enabled);
        w.set_lk(lock);
        w.set_sircerr(Sircerr::ErrorNotDetected);
    });
}

fn enter_power_mode(mode: PowerMode) -> Result<hal::clocks::SleepStatus, hal::clocks::PowerModeError> {
    let result = critical_section::with(|cs| {
        match mode {
            PowerMode::Active => Err(hal::clocks::PowerModeError::ConfigurationRejected),
            PowerMode::Sleep => {
                // SAFETY: UART/time consumers are suspended, the wake source is
                // armed, and the critical section covers entry and clock recovery.
                unsafe { hal::clocks::go_to_sleep_with_status(&cs) }
            }
            PowerMode::DeepSleep => {
                // SAFETY: consumers are suspended, the wake source is armed, and
                // the critical section covers temporary SIRC/PLL clock switching.
                unsafe { hal::clocks::go_to_deep_sleep_with_status(&cs) }
            }
        }
    });
    cortex_m::asm::isb();
    result
}

fn print_sleep_status(console: &mut Console<'_>, status: hal::clocks::SleepStatus) {
    console.print(format_args!(
        "Status before recovery:\r\n\
         Requested MAIN: {:#010X}\r\nCMC.CKSTAT: {:#010X}\r\n\
         Wake MAIN: {:#010X}\r\nSPC.PD_STATUS0: {:#010X}\r\nSPC.SC: {:#010X}\r\n",
        status.requested_main_mode, status.ckstat, status.wake_main_mode, status.pd_status0, status.spc_sc
    ));
}

fn disable_wake_timer() {
    interrupt::WAKETIMER0.disable();
    write_wake_control(OSC_DIVIDE_ENABLE | CLEAR_WAKE_TIMER | WAKE_FLAG);
    interrupt::WAKETIMER0.unpend();
}

fn arm_wake_timer(seconds: u8) {
    disable_wake_timer();
    write_wake_control(OSC_DIVIDE_ENABLE | WAKE_INTERRUPT_ENABLE | WAKE_FLAG);
    // SAFETY: WAKETIMER0 is exclusively owned by this example. The timer was
    // halted before loading the count, which starts the one-shot countdown.
    unsafe { core::ptr::write_volatile(WAKE_TIMER_COUNT, u32::from(seconds) * WAKE_TIMER_TICKS_PER_SECOND) };
    // SAFETY: Irqs installs WakeTimerInterruptHandler and stale flags were cleared.
    unsafe { interrupt::WAKETIMER0.enable() };
}

fn write_wake_control(value: u32) {
    // SAFETY: WAKE_TIMER_CONTROL is the aligned MCXA2xx WAKETIMER0 control
    // register; writes contain only its documented control/W1C bits.
    unsafe {
        core::ptr::write_volatile(WAKE_TIMER_CONTROL, value);
        let _ = core::ptr::read_volatile(WAKE_TIMER_CONTROL);
    }
}

fn clear_stale_event() {
    cortex_m::asm::sev();
    cortex_m::asm::wfe();
}

fn fatal(message: &str) -> ! {
    defmt::error!("{}", message);
    loop {
        cortex_m::asm::nop();
    }
}
