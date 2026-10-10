//! Interactive MCXA577 power-mode demonstration.
//!
//! This is an Embassy port of NXP's `power_mode_switch` SDK example for the
//! FRDM-MCXA577. The menu is available at 115200 baud through LPUART1 and the
//! MCU-Link USB CDC/VCOM port. Low-power modes can wake from either LPTMR0 or
//! the SW2/WAKEUP button.
//! Supported modes are Active, Sleep, and Deep Sleep.
//!
//! Run with:
//!
//! ```sh
//! cargo run --release --no-default-features --features=executor-platform --bin power-mode-switch
//! ```

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use hal::clocks::PoweredClock;
use hal::clocks::config::{CoreSleep, FlashSleep};
use hal::gpio::Pull;
use hal::interrupt::InterruptExt;
use hal::lpuart::{Blocking, Config as UartConfig, Lpuart, LpuartRx, LpuartTx};
use hal::pac::scg::{SirccsrLk, Sircvld};
use hal::pac::spc::{LpCfgBgmode, LpCfgCoreldoVddDs, LpCfgCoreldoVddLvl};
use hal::reset_reason::ResetReasonRaw;
use hal::wuu::{ConfiguredWake, Edge, Event, ExternalPinConfig, InternalModule, PinMode, WakeupPin, Wuu};
use hal::{bind_interrupts, interrupt};
use panic_probe as _;

const LPTMR0_BASE: usize = 0x400A_B000;
const LPTMR_CSR: usize = 0x00;
const LPTMR_PSR: usize = 0x04;
const LPTMR_CMR: usize = 0x08;
const LPTMR_CSR_TEN: u32 = 1 << 0;
const LPTMR_CSR_TIE: u32 = 1 << 6;
const LPTMR_CSR_TCF: u32 = 1 << 7;
const LPTMR_PSR_PCS_FRO16K: u32 = 1;
const LPTMR_PSR_BYPASS: u32 = 1 << 2;
const LPTMR_CLOCK_HZ: u32 = 16_384;
const DEBUG_RECOVERY_DELAY_CYCLES: u32 = 48_000_000;
const BUTTON_WAKE_EVD_ISOLATION_MASK: u32 = 0x16;
const EVDLPISO_SHIFT: u32 = 8;

bind_interrupts!(pub struct Irqs {
    LPTMR0 => LptmrInterruptHandler;
    WUU0 => hal::wuu::InterruptHandler;
});

struct LptmrInterruptHandler;

impl interrupt::typelevel::Handler<interrupt::typelevel::LPTMR0> for LptmrInterruptHandler {
    unsafe fn on_interrupt() {
        let csr = read_register(LPTMR0_BASE, LPTMR_CSR);
        if csr & LPTMR_CSR_TIE != 0 {
            write_register(
                LPTMR0_BASE,
                LPTMR_CSR,
                (csr & !(LPTMR_CSR_TIE | LPTMR_CSR_TEN)) | LPTMR_CSR_TCF,
            );
        }
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
        match uppercase(byte) {
            b'A' => Some(Self::Active),
            b'B' => Some(Self::Sleep),
            b'C' => Some(Self::DeepSleep),
            _ => None,
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Active => "Active: clocks remain running.\r\n",
            Self::Sleep => "Sleep: the core clock is gated.\r\n",
            Self::DeepSleep => "Deep Sleep: system clocks are gated and low-power mode is entered.\r\n",
        }
    }
}

#[derive(Clone, Copy)]
enum WakeSource {
    Timer,
    Button,
}

struct Console<'d> {
    tx: LpuartTx<'d, Blocking>,
    rx: LpuartRx<'d, Blocking>,
}

impl<'d> Console<'d> {
    fn write(&mut self, text: &str) {
        self.write_bytes(text.as_bytes());
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        if self.tx.blocking_write(bytes).is_err() {
            uart_failure();
        }
    }

    fn flush(&mut self) {
        if self.tx.blocking_flush().is_err() {
            uart_failure();
        }
    }

    fn suspend_for_low_power(&mut self) {
        self.flush();
        hal::pac::LPUART1.ctrl().modify(|w| {
            w.set_te(false);
            w.set_re(false);
        });
    }

    fn resume_after_low_power(&mut self) {
        hal::pac::LPUART1.ctrl().modify(|w| {
            w.set_te(true);
            w.set_re(true);
        });
    }

    fn read_byte(&mut self) -> u8 {
        let mut byte = [0];
        if self.rx.blocking_read(&mut byte).is_err() {
            uart_failure();
        }
        byte.first().copied().unwrap_or_default()
    }

    fn read_choice(&mut self) -> u8 {
        let byte = uppercase(self.read_byte());
        self.write_bytes(&[byte]);
        self.write("\r\n");
        byte
    }
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
    config.clock_cfg.sirc.fro_lf_div = Some(hal::clocks::config::Div8::no_div());
    config.clock_cfg.sirc.power = PoweredClock::AlwaysEnabled;
    config.clock_cfg.vdd_power.core_sleep = CoreSleep::DeepSleep;
    config.clock_cfg.vdd_power.flash_sleep = FlashSleep::FlashDoze;

    let p = hal::init(config);

    let uart_config = UartConfig {
        baudrate_bps: 115_200,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        ..Default::default()
    };
    let uart = match Lpuart::new_blocking(p.LPUART1, p.P1_9, p.P1_8, uart_config) {
        Ok(uart) => uart,
        Err(_) => uart_failure(),
    };
    let (tx, rx) = uart.split();
    let mut console = Console { tx, rx };

    let mut wakeup_button = WakeupPin::new(p.P3_17, Pull::Up);
    let mut wuu = match Wuu::new(p.WUU0, Irqs) {
        Ok(wuu) => wuu,
        Err(_) => uart_failure(),
    };
    let _lptmr = p.LPTMR0;

    console.write("\r\n");
    write_reset_reason(&mut console, reset_reason);

    loop {
        console.write("\r\n###################  Rust    Power Mode Switch Demo    ###########################\r\n");
        console.write("    Core Clock = 48000000Hz\r\n");
        console.write("    Power mode: Active\r\n");

        let power_mode = select_power_mode(&mut console);
        console.write("\r\n");
        console.write(power_mode.description());

        if power_mode != PowerMode::Active {
            clear_stale_event();
            let wake_source = select_wake_source(&mut console);
            let configured_wake =
                configure_wake_source(&mut console, power_mode, wake_source, &mut wuu, &mut wakeup_button);
            let saved_lp_config = if power_mode == PowerMode::DeepSleep {
                Some(suspend_sirc_consumers(&mut console))
            } else {
                console.flush();
                None
            };
            let entry_result = enter_power_mode(power_mode, &configured_wake);
            if let Some(saved_lp_config) = saved_lp_config {
                resume_sirc_consumers(&mut console, saved_lp_config);
            }
            match entry_result {
                Ok(Some(status)) => write_sleep_status(&mut console, status),
                Ok(None) => {}
                Err(error) => {
                    defmt::warn!("Power mode entry failed: {:?}", error);
                    console.write("Power mode entry failed: ");
                    console.write(power_mode_error_message(error));
                    console.write(".\r\n");
                }
            }
        }

        console.write("\r\nNext loop.\r\n");
    }
}

fn select_power_mode(console: &mut Console<'_>) -> PowerMode {
    loop {
        console.write("\r\nSelect the desired operation:\r\n\r\n");
        console.write("\tPress A to enter: Active mode\r\n");
        console.write("\tPress B to enter: Sleep mode\r\n");
        console.write("\tPress C to enter: Deep Sleep mode\r\n");
        console.write("\r\nWaiting for power mode select...\r\n\r\n");

        if let Some(mode) = PowerMode::from_byte(console.read_choice()) {
            return mode;
        }

        console.write("Wrong input!\r\n");
    }
}

fn select_wake_source(console: &mut Console<'_>) -> WakeSource {
    loop {
        console.write("Please select wakeup source:\r\n");
        console.write("\tPress A to select TIMER as wakeup source;\r\n");
        console.write("\tPress B to select WAKE-UP-BUTTON as wakeup source;\r\n");
        console.write("Waiting for wakeup source select...\r\n");

        match console.read_choice() {
            b'A' => return WakeSource::Timer,
            b'B' => return WakeSource::Button,
            _ => console.write("Wrong input!\r\n"),
        }
    }
}

fn get_wakeup_timeout(console: &mut Console<'_>) -> u8 {
    loop {
        console.write("Select the wake up timeout in seconds.\r\n");
        console.write("The allowed range is 1s ~ 9s.\r\n");
        console.write("For example, enter 5 to wake up in 5 seconds.\r\n");
        console.write("\r\nWaiting for input timeout value...\r\n\r\n");

        let byte = console.read_choice();
        if let b'1'..=b'9' = byte {
            return byte - b'0';
        }

        console.write("Wrong value!\r\n");
    }
}

fn configure_wake_source<'a, 'd>(
    console: &mut Console<'_>,
    power_mode: PowerMode,
    wake_source: WakeSource,
    wuu: &'a mut Wuu<'d>,
    wakeup_button: &'a mut WakeupPin<'d>,
) -> ConfiguredWake<'a, 'd> {
    disable_timer_wakeup();
    hal::pac::SPC0.evd_cfg().write(|w| w.0 = 0);

    match wake_source {
        WakeSource::Timer => {
            console.write("Timer selected as wakeup source.\r\n");
            let timeout = get_wakeup_timeout(console);
            console.write("The timer will wake the device in ");
            console.write_bytes(&[b'0' + timeout]);
            console.write(" seconds.\r\n");

            let configured_wake = wuu.enable_internal_module(InternalModule::Lptmr0);
            configure_timer_wakeup(timeout);
            configured_wake
        }
        WakeSource::Button => {
            console.write("Wakeup button selected as wakeup source.\r\n");
            let configured_wake = wuu.enable_external_pin(
                wakeup_button,
                ExternalPinConfig {
                    edge: Edge::Falling,
                    event: Event::Interrupt,
                    mode: PinMode::AnyPower,
                },
            );
            console.write("Please press SW2/WAKEUP to wake the device.\r\n");
            if power_mode == PowerMode::DeepSleep {
                hal::pac::SPC0.evd_cfg().write(|w| {
                    w.0 = BUTTON_WAKE_EVD_ISOLATION_MASK | (BUTTON_WAKE_EVD_ISOLATION_MASK << EVDLPISO_SHIFT);
                });
                console.write("Isolate power domains: VDD_USB, VDD_P2/ANA, VDD_P4.\r\n");
            }
            configured_wake
        }
    }
}

fn disable_timer_wakeup() {
    interrupt::LPTMR0.disable();
    interrupt::LPTMR0.unpend();

    write_register(LPTMR0_BASE, LPTMR_CSR, LPTMR_CSR_TCF);
}

fn configure_timer_wakeup(timeout_seconds: u8) {
    write_register(LPTMR0_BASE, LPTMR_CSR, 0);
    write_register(LPTMR0_BASE, LPTMR_PSR, LPTMR_PSR_PCS_FRO16K | LPTMR_PSR_BYPASS);
    write_register(LPTMR0_BASE, LPTMR_CMR, LPTMR_CLOCK_HZ * u32::from(timeout_seconds) - 1);
    write_register(LPTMR0_BASE, LPTMR_CSR, LPTMR_CSR_TCF);

    interrupt::LPTMR0.unpend();
    // SAFETY: LPTMR0 is bound to LptmrInterruptHandler above, and its status
    // flag is cleared before the interrupt is enabled.
    unsafe { interrupt::LPTMR0.enable() };
    write_register(LPTMR0_BASE, LPTMR_CSR, LPTMR_CSR_TIE | LPTMR_CSR_TEN);
}

fn suspend_sirc_consumers(console: &mut Console<'_>) -> (u32, u32) {
    console.suspend_for_low_power();

    interrupt::OS_EVENT.disable();
    hal::pac::OSTIMER0.osevent_ctrl().modify(|w| {
        w.set_ostimer_intena(false);
        w.set_ostimer_intrflag(true);
    });
    interrupt::OS_EVENT.unpend();

    let scg = hal::pac::SCG0;
    scg.sirccsr().modify(|w| w.set_lk(SirccsrLk::WriteEnabled));
    scg.sirccsr().modify(|w| w.set_sircsten(false));
    scg.sirccsr().modify(|w| w.set_lk(SirccsrLk::WriteDisabled));

    configure_low_power_regulator()
}

fn configure_low_power_regulator() -> (u32, u32) {
    let spc = hal::pac::SPC0;
    while spc.sc().read().busy() {}

    let saved_lp_config = spc.lp_cfg().read().0;
    let saved_lp_config1 = spc.lp_cfg1().read().0;
    spc.lp_cfg1().write(|w| w.0 = 0);
    spc.lp_cfg().modify(|w| {
        w.set_sramldo_dpd_on(false);
        w.set_core_lvde(false);
        w.set_sys_lvde(false);
        w.set_sys_hvde(false);
        w.set_lp_irefen(false);
    });
    spc.lp_cfg().modify(|w| w.set_coreldo_vdd_ds(LpCfgCoreldoVddDs::Low));
    spc.lp_cfg().modify(|w| w.set_coreldo_vdd_lvl(LpCfgCoreldoVddLvl::Mid));
    spc.lp_cfg().modify(|w| w.set_bgmode(LpCfgBgmode::Bgmode0));

    while spc.sc().read().busy() {}
    (saved_lp_config, saved_lp_config1)
}

fn resume_sirc_consumers(console: &mut Console<'_>, saved_lp_config: (u32, u32)) {
    let scg = hal::pac::SCG0;
    scg.sirccsr().modify(|w| w.set_lk(SirccsrLk::WriteEnabled));
    scg.sirccsr().modify(|w| {
        w.set_sirc_clk_periph_en(true);
        w.set_sircsten(true);
    });
    scg.sirccsr().modify(|w| w.set_lk(SirccsrLk::WriteDisabled));
    while scg.sirccsr().read().sircvld() != Sircvld::EnabledAndValid {}

    let spc = hal::pac::SPC0;
    spc.lp_cfg().modify(|w| w.set_bgmode(LpCfgBgmode::Bgmode01));
    spc.lp_cfg().write(|w| w.0 = saved_lp_config.0);
    spc.lp_cfg1().write(|w| w.0 = saved_lp_config.1);
    while spc.sc().read().busy() {}

    console.resume_after_low_power();
    interrupt::OS_EVENT.unpend();
    // SAFETY: HAL initialization installed the OSTIMER time-driver handler.
    unsafe { interrupt::OS_EVENT.enable() };
}

fn enter_power_mode(
    power_mode: PowerMode,
    _wake_source: &ConfiguredWake<'_, '_>,
) -> Result<Option<hal::clocks::SleepStatus>, hal::clocks::PowerModeError> {
    let result = critical_section::with(|cs| {
        match power_mode {
            PowerMode::Active => Ok(None),
            PowerMode::Sleep => {
                // SAFETY: clock initialization completed, the selected wake
                // source is armed, UART transmission is complete, and the
                // critical section remains held throughout entry and recovery.
                unsafe { hal::clocks::go_to_sleep_with_status(&cs) }.map(Some)
            }
            PowerMode::DeepSleep => {
                // SAFETY: the clock configuration permits Deep Sleep, the
                // selected wake source is armed, and the critical section
                // remains held throughout entry and recovery.
                unsafe { hal::clocks::go_to_deep_sleep_with_status(&cs) }.map(Some)
            }
        }
    });
    cortex_m::asm::isb();
    result
}

fn write_sleep_status(console: &mut Console<'_>, status: hal::clocks::SleepStatus) {
    console.write("Sleep attempt status before recovery (requested mode is not proof of entry):\r\n");
    write_hex_register(console, "  requested MAIN mode: ", status.requested_main_mode);
    write_hex_register(console, "  CMC.CKSTAT:         ", status.ckstat);
    write_hex_register(console, "  wake MAIN mode:     ", status.wake_main_mode);
    write_hex_register(console, "  SPC.PD_STATUS0:     ", status.pd_status0);
    write_hex_register(console, "  SPC.SC:             ", status.spc_sc);
}

fn write_hex_register(console: &mut Console<'_>, label: &str, value: u32) {
    const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

    let mut encoded = *b"0x00000000\r\n";
    for position in 0..8 {
        let shift = (7 - position) * 4;
        let digit = usize::try_from((value >> shift) & 0xf).unwrap_or_default();
        if let (Some(destination), Some(source)) = (encoded.get_mut(position + 2), HEX_DIGITS.get(digit)) {
            *destination = *source;
        }
    }

    console.write(label);
    console.write_bytes(&encoded);
}

fn clear_stale_event() {
    cortex_m::asm::sev();
    cortex_m::asm::wfe();
}

fn uppercase(byte: u8) -> u8 {
    if byte.is_ascii_lowercase() {
        byte - (b'a' - b'A')
    } else {
        byte
    }
}

fn write_reset_reason(console: &mut Console<'_>, reason: ResetReasonRaw) {
    console.write("Reset cause:");
    if reason.is_wakeup() {
        console.write(" wake-up");
    }
    if reason.is_por() {
        console.write(" power-on");
    }
    if reason.is_voltage_detect() {
        console.write(" voltage-detect");
    }
    if reason.is_pin() {
        console.write(" reset-pin");
    }
    if reason.is_dap() {
        console.write(" debug-access-port");
    }
    if reason.is_jtag() {
        console.write(" JTAG");
    }
    if reason.is_low_power_ack_timeout() {
        console.write(" low-power-ack-timeout");
    }
    if reason.is_system_clock_generation() {
        console.write(" system-clock");
    }
    if reason.is_watchdog0() || reason.is_watchdog1() {
        console.write(" watchdog");
    }
    if reason.is_software() {
        console.write(" software");
    }
    if reason.is_lockup() {
        console.write(" lockup");
    }
    console.write("\r\n");
}

fn power_mode_error_message(error: hal::clocks::PowerModeError) -> &'static str {
    match error {
        hal::clocks::PowerModeError::ClockNotInitialized => "clocks have not been initialized",
        hal::clocks::PowerModeError::ClockControlLocked => "CKCTRL is locked in an incompatible mode",
        hal::clocks::PowerModeError::PowerModeProtectionLocked => {
            "PMPROT is locked without permitting the requested mode"
        }
        hal::clocks::PowerModeError::ConfigurationRejected => "clock/power configuration rejected",
        hal::clocks::PowerModeError::RecoveryRejected => "CMC rejected idle-mode recovery",
    }
}

fn read_register(base: usize, offset: usize) -> u32 {
    // SAFETY: the caller supplies an aligned register offset within the fixed
    // MCXA577 peripheral register blocks used by this example.
    unsafe { core::ptr::read_volatile((base + offset) as *const u32) }
}

fn write_register(base: usize, offset: usize, value: u32) {
    // SAFETY: the caller supplies an aligned register offset within the fixed
    // MCXA577 peripheral register blocks used by this example.
    unsafe { core::ptr::write_volatile((base + offset) as *mut u32, value) };
}

fn uart_failure() -> ! {
    loop {
        cortex_m::asm::wfi();
    }
}
