#![no_std]
#![feature(generic_associated_types)]
#![feature(asm)]
#![feature(type_alias_impl_trait)]
#![feature(never_type)]
#![allow(incomplete_features)]

pub use rp2040_pac2 as pac;

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

pub mod interrupt;

pub mod dma;
pub mod gpio;
pub mod pll;
pub mod resets;
pub mod uart;

embassy_extras::peripherals! {
    PIN_0,
    PIN_1,
    PIN_2,
    PIN_3,
    PIN_4,
    PIN_5,
    PIN_6,
    PIN_7,
    PIN_8,
    PIN_9,
    PIN_10,
    PIN_11,
    PIN_12,
    PIN_13,
    PIN_14,
    PIN_15,
    PIN_16,
    PIN_17,
    PIN_18,
    PIN_19,
    PIN_20,
    PIN_21,
    PIN_22,
    PIN_23,
    PIN_24,
    PIN_25,
    PIN_26,
    PIN_27,
    PIN_28,
    PIN_29,
    PIN_QSPI_SCLK,
    PIN_QSPI_SS,
    PIN_QSPI_SD0,
    PIN_QSPI_SD1,
    PIN_QSPI_SD2,
    PIN_QSPI_SD3,

    UART0,
    UART1,

    DMA_CH0,
    DMA_CH1,
    DMA_CH2,
    DMA_CH3,
    DMA_CH4,
    DMA_CH5,
    DMA_CH6,
    DMA_CH7,
    DMA_CH8,
    DMA_CH9,
    DMA_CH10,
    DMA_CH11,
}

#[link_section = ".boot2"]
#[used]
static BOOT2: [u8; 256] = *include_bytes!("boot2.bin");

pub mod config {
    #[non_exhaustive]
    pub struct Config {}

    impl Default for Config {
        fn default() -> Self {
            Self {}
        }
    }
}

pub fn init(_config: config::Config) -> Peripherals {
    // Do this first, so that it panics if user is calling `init` a second time
    // before doing anything important.
    let peripherals = Peripherals::take();

    // Now reset all the peripherals, except QSPI and XIP (we're using those
    // to execute from external flash!)

    let resets = resets::Resets::new();

    // Reset everything except:
    // - QSPI (we're using it to run this code!)
    // - PLLs (it may be suicide if that's what's clocking us)
    let mut peris = resets::ALL_PERIPHERALS;
    peris.set_io_qspi(false);
    peris.set_pads_qspi(false);
    peris.set_pll_sys(false);
    peris.set_pll_usb(false);
    resets.reset(peris);

    let mut peris = resets::ALL_PERIPHERALS;
    peris.set_adc(false);
    peris.set_rtc(false);
    peris.set_spi0(false);
    peris.set_spi1(false);
    peris.set_uart0(false);
    peris.set_uart1(false);
    peris.set_usbctrl(false);
    resets.unreset_wait(peris);

    unsafe {
        // xosc 12 mhz
        pac::WATCHDOG.tick().write(|w| {
            w.set_cycles(XOSC_MHZ as u16);
            w.set_enable(true);
        });

        pac::CLOCKS
            .clk_sys_resus_ctrl()
            .write_value(pac::clocks::regs::ClkSysResusCtrl(0));

        // Enable XOSC
        // TODO extract to HAL module
        const XOSC_MHZ: u32 = 12;
        pac::XOSC
            .ctrl()
            .write(|w| w.set_freq_range(pac::xosc::vals::CtrlFreqRange::_1_15MHZ));

        let startup_delay = (((XOSC_MHZ * 1_000_000) / 1000) + 128) / 256;
        pac::XOSC
            .startup()
            .write(|w| w.set_delay(startup_delay as u16));
        pac::XOSC.ctrl().write(|w| {
            w.set_freq_range(pac::xosc::vals::CtrlFreqRange::_1_15MHZ);
            w.set_enable(pac::xosc::vals::CtrlEnable::ENABLE);
        });
        while !pac::XOSC.status().read().stable() {}

        // Before we touch PLLs, switch sys and ref cleanly away from their aux sources.
        pac::CLOCKS
            .clk_sys_ctrl()
            .modify(|w| w.set_src(pac::clocks::vals::ClkSysCtrlSrc::CLK_REF));
        while pac::CLOCKS.clk_sys_selected().read() != 1 {}
        pac::CLOCKS
            .clk_ref_ctrl()
            .modify(|w| w.set_src(pac::clocks::vals::ClkRefCtrlSrc::ROSC_CLKSRC_PH));
        while pac::CLOCKS.clk_ref_selected().read() != 1 {}

        let mut peris = resets::Peripherals(0);
        peris.set_pll_sys(true);
        peris.set_pll_usb(true);
        resets.reset(peris);
        resets.unreset_wait(peris);

        pll::PLL::new(pll::PllSys).configure(1, 1500_000_000, 6, 2);
        pll::PLL::new(pll::PllUsb).configure(1, 480_000_000, 5, 2);

        // Activate peripheral clock and take external oscillator as input
        pac::CLOCKS.clk_peri_ctrl().write(|w| {
            w.set_enable(true);
            w.set_auxsrc(pac::clocks::vals::ClkPeriCtrlAuxsrc::XOSC_CLKSRC);
        });
    }

    peripherals
}
