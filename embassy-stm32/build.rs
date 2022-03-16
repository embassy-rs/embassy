use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;
use stm32_metapac::metadata::METADATA;

fn main() {
    let chip_name = match env::vars()
        .map(|(a, _)| a)
        .filter(|x| x.starts_with("CARGO_FEATURE_STM32"))
        .get_one()
    {
        Ok(x) => x,
        Err(GetOneError::None) => panic!("No stm32xx Cargo feature enabled"),
        Err(GetOneError::Multiple) => panic!("Multiple stm32xx Cargo features enabled"),
    }
    .strip_prefix("CARGO_FEATURE_")
    .unwrap()
    .to_ascii_lowercase();

    for p in METADATA.peripherals {
        if let Some(r) = &p.registers {
            println!("cargo:rustc-cfg={}", r.kind);
            println!("cargo:rustc-cfg={}_{}", r.kind, r.version);
        }
    }

    // ========
    // Generate singletons

    let mut singletons: Vec<String> = Vec::new();
    for p in METADATA.peripherals {
        if let Some(r) = &p.registers {
            match r.kind {
                // Generate singletons per pin, not per port
                "gpio" => {
                    println!("{}", p.name);
                    let port_letter = p.name.strip_prefix("GPIO").unwrap();
                    for pin_num in 0..16 {
                        singletons.push(format!("P{}{}", port_letter, pin_num));
                    }
                }

                // No singleton for these, the HAL handles them specially.
                "exti" => {}

                // We *shouldn't* have singletons for these, but the HAL currently requires
                // singletons, for using with RccPeripheral to enable/disable clocks to them.
                "rcc" => {
                    if r.version.starts_with("h7") {
                        singletons.push("MCO1".to_string());
                        singletons.push("MCO2".to_string());
                    }
                    singletons.push(p.name.to_string());
                }
                //"dbgmcu" => {}
                //"syscfg" => {}
                //"dma" => {}
                //"bdma" => {}
                //"dmamux" => {}

                // For other peripherals, one singleton per peri
                _ => singletons.push(p.name.to_string()),
            }
        }
    }

    // One singleton per EXTI line
    for pin_num in 0..16 {
        singletons.push(format!("EXTI{}", pin_num));
    }

    // One singleton per DMA channel
    for c in METADATA.dma_channels {
        singletons.push(c.name.to_string());
    }

    let mut g = TokenStream::new();

    let singleton_tokens: Vec<_> = singletons.iter().map(|s| format_ident!("{}", s)).collect();
    g.extend(quote! {
        embassy_hal_common::peripherals!(#(#singleton_tokens),*);
    });

    // ========
    // Generate interrupt declarations

    let mut irqs = Vec::new();
    for irq in METADATA.interrupts {
        irqs.push(format_ident!("{}", irq.name));
    }

    g.extend(quote! {
        pub mod interrupt {
            use crate::pac::Interrupt as InterruptEnum;
            #(
                embassy::interrupt::declare!(#irqs);
            )*
        }
    });

    // ========
    // Generate DMA IRQs.

    let mut dma_irqs: HashMap<&str, Vec<(&str, &str)>> = HashMap::new();

    for p in METADATA.peripherals {
        if let Some(r) = &p.registers {
            if r.kind == "dma" || r.kind == "bdma" {
                if p.name == "BDMA1" {
                    // BDMA1 in H7 doesn't use DMAMUX, which breaks
                    continue;
                }
                for irq in p.interrupts {
                    dma_irqs
                        .entry(irq.interrupt)
                        .or_default()
                        .push((p.name, irq.signal));
                }
            }
        }
    }

    for (irq, channels) in dma_irqs {
        let irq = format_ident!("{}", irq);

        let channels = channels
            .iter()
            .map(|(dma, ch)| format_ident!("{}_{}", dma, ch));

        g.extend(quote! {
            #[crate::interrupt]
            unsafe fn #irq () {
                #(
                    <crate::peripherals::#channels as crate::dma::sealed::Channel>::on_irq();
                )*
            }
        });
    }

    // ========
    // Generate RccPeripheral impls

    for p in METADATA.peripherals {
        if !singletons.contains(&p.name.to_string()) {
            continue;
        }

        if let Some(rcc) = &p.rcc {
            let en = rcc.enable.as_ref().unwrap();

            let rst = match &rcc.reset {
                Some(rst) => {
                    let rst_reg = format_ident!("{}", rst.register.to_ascii_lowercase());
                    let set_rst_field = format_ident!("set_{}", rst.field.to_ascii_lowercase());
                    quote! {
                        critical_section::with(|_| unsafe {
                            crate::pac::RCC.#rst_reg().modify(|w| w.#set_rst_field(true));
                            crate::pac::RCC.#rst_reg().modify(|w| w.#set_rst_field(false));
                        });
                    }
                }
                None => TokenStream::new(),
            };

            let pname = format_ident!("{}", p.name);
            let clk = format_ident!("{}", rcc.clock.to_ascii_lowercase());
            let en_reg = format_ident!("{}", en.register.to_ascii_lowercase());
            let set_en_field = format_ident!("set_{}", en.field.to_ascii_lowercase());

            g.extend(quote! {
                impl crate::rcc::sealed::RccPeripheral for peripherals::#pname {
                    fn frequency() -> crate::time::Hertz {
                        critical_section::with(|_| unsafe {
                            crate::rcc::get_freqs().#clk
                        })
                    }
                    fn enable() {
                        critical_section::with(|_| unsafe {
                            crate::pac::RCC.#en_reg().modify(|w| w.#set_en_field(true))
                        })
                    }
                    fn disable() {
                        critical_section::with(|_| unsafe {
                            crate::pac::RCC.#en_reg().modify(|w| w.#set_en_field(false));
                        })
                    }
                    fn reset() {
                        #rst
                    }
                }

                impl crate::rcc::RccPeripheral for peripherals::#pname {}
            });
        }
    }

    // ========
    // Generate fns to enable GPIO, DMA in RCC

    for kind in ["dma", "bdma", "dmamux", "gpio"] {
        let mut gg = TokenStream::new();

        for p in METADATA.peripherals {
            if p.registers.is_some() && p.registers.as_ref().unwrap().kind == kind {
                if let Some(rcc) = &p.rcc {
                    let en = rcc.enable.as_ref().unwrap();
                    let en_reg = format_ident!("{}", en.register.to_ascii_lowercase());
                    let set_en_field = format_ident!("set_{}", en.field.to_ascii_lowercase());

                    gg.extend(quote! {
                        crate::pac::RCC.#en_reg().modify(|w| w.#set_en_field(true));
                    })
                }
            }
        }

        let fname = format_ident!("init_{}", kind);
        g.extend(quote! {
            pub unsafe fn #fname(){
                #gg
            }
        })
    }

    // ========
    // Generate pin_trait_impl!

    #[rustfmt::skip]
    let signals: HashMap<_, _> = [
        // (kind, signal) => (trait, cfgs)
        (("usart", "TX"), (quote!(crate::usart::TxPin), quote!())),
        (("usart", "RX"), (quote!(crate::usart::RxPin), quote!())),
        (("usart", "CTS"), (quote!(crate::usart::CtsPin), quote!())),
        (("usart", "RTS"), (quote!(crate::usart::RtsPin), quote!())),
        (("usart", "CK"), (quote!(crate::usart::CkPin), quote!())),
        (("usart", "TX"), (quote!(crate::usart::TxPin), quote!())),
        (("usart", "RX"), (quote!(crate::usart::RxPin), quote!())),
        (("usart", "CTS"), (quote!(crate::usart::CtsPin), quote!())),
        (("usart", "RTS"), (quote!(crate::usart::RtsPin), quote!())),
        (("usart", "CK"), (quote!(crate::usart::CkPin), quote!())),
        (("spi", "SCK"), (quote!(crate::spi::SckPin), quote!())),
        (("spi", "MOSI"), (quote!(crate::spi::MosiPin), quote!())),
        (("spi", "MISO"), (quote!(crate::spi::MisoPin), quote!())),
        (("i2c", "SDA"), (quote!(crate::i2c::SdaPin), quote!())),
        (("i2c", "SCL"), (quote!(crate::i2c::SclPin), quote!())),
        (("rcc", "MCO_1"), (quote!(crate::rcc::McoPin), quote!())),
        (("rcc", "MCO_2"), (quote!(crate::rcc::McoPin), quote!())),
        (("dcmi", "D0"), (quote!(crate::dcmi::D0Pin), quote!())),
        (("dcmi", "D1"), (quote!(crate::dcmi::D1Pin), quote!())),
        (("dcmi", "D2"), (quote!(crate::dcmi::D2Pin), quote!())),
        (("dcmi", "D3"), (quote!(crate::dcmi::D3Pin), quote!())),
        (("dcmi", "D4"), (quote!(crate::dcmi::D4Pin), quote!())),
        (("dcmi", "D5"), (quote!(crate::dcmi::D5Pin), quote!())),
        (("dcmi", "D6"), (quote!(crate::dcmi::D6Pin), quote!())),
        (("dcmi", "D7"), (quote!(crate::dcmi::D7Pin), quote!())),
        (("dcmi", "D8"), (quote!(crate::dcmi::D8Pin), quote!())),
        (("dcmi", "D9"), (quote!(crate::dcmi::D9Pin), quote!())),
        (("dcmi", "D10"), (quote!(crate::dcmi::D10Pin), quote!())),
        (("dcmi", "D11"), (quote!(crate::dcmi::D11Pin), quote!())),
        (("dcmi", "D12"), (quote!(crate::dcmi::D12Pin), quote!())),
        (("dcmi", "D13"), (quote!(crate::dcmi::D13Pin), quote!())),
        (("dcmi", "HSYNC"), (quote!(crate::dcmi::HSyncPin), quote!())),
        (("dcmi", "VSYNC"), (quote!(crate::dcmi::VSyncPin), quote!())),
        (("dcmi", "PIXCLK"), (quote!(crate::dcmi::PixClkPin), quote!())),
        (("otgfs", "DP"), (quote!(crate::usb_otg::DpPin), quote!(#[cfg(feature="usb-otg")]))),
        (("otgfs", "DM"), (quote!(crate::usb_otg::DmPin), quote!(#[cfg(feature="usb-otg")]))),
        (("otghs", "DP"), (quote!(crate::usb_otg::DpPin), quote!(#[cfg(feature="usb-otg")]))),
        (("otghs", "DM"), (quote!(crate::usb_otg::DmPin), quote!(#[cfg(feature="usb-otg")]))),
        (("otghs", "ULPI_CK"), (quote!(crate::usb_otg::UlpiClkPin), quote!(#[cfg(feature="usb-otg")]))),
        (("otghs", "ULPI_DIR"), (quote!(crate::usb_otg::UlpiDirPin), quote!(#[cfg(feature="usb-otg")]))),
        (("otghs", "ULPI_NXT"), (quote!(crate::usb_otg::UlpiNxtPin), quote!(#[cfg(feature="usb-otg")]))),
        (("otghs", "ULPI_STP"), (quote!(crate::usb_otg::UlpiStpPin), quote!(#[cfg(feature="usb-otg")]))),
        (("otghs", "ULPI_D0"), (quote!(crate::usb_otg::UlpiD0Pin), quote!(#[cfg(feature="usb-otg")]))),
        (("otghs", "ULPI_D1"), (quote!(crate::usb_otg::UlpiD1Pin), quote!(#[cfg(feature="usb-otg")]))),
        (("otghs", "ULPI_D2"), (quote!(crate::usb_otg::UlpiD2Pin), quote!(#[cfg(feature="usb-otg")]))),
        (("otghs", "ULPI_D3"), (quote!(crate::usb_otg::UlpiD3Pin), quote!(#[cfg(feature="usb-otg")]))),
        (("otghs", "ULPI_D4"), (quote!(crate::usb_otg::UlpiD4Pin), quote!(#[cfg(feature="usb-otg")]))),
        (("otghs", "ULPI_D5"), (quote!(crate::usb_otg::UlpiD5Pin), quote!(#[cfg(feature="usb-otg")]))),
        (("otghs", "ULPI_D6"), (quote!(crate::usb_otg::UlpiD6Pin), quote!(#[cfg(feature="usb-otg")]))),
        (("otghs", "ULPI_D7"), (quote!(crate::usb_otg::UlpiD7Pin), quote!(#[cfg(feature="usb-otg")]))),
        (("can", "TX"), (quote!(crate::can::TxPin), quote!())),
        (("can", "RX"), (quote!(crate::can::RxPin), quote!())),
        (("eth", "REF_CLK"), (quote!(crate::eth::RefClkPin), quote!(#[cfg(feature="net")]))),
        (("eth", "MDIO"), (quote!(crate::eth::MDIOPin), quote!(#[cfg(feature="net")]))),
        (("eth", "MDC"), (quote!(crate::eth::MDCPin), quote!(#[cfg(feature="net")]))),
        (("eth", "CRS_DV"), (quote!(crate::eth::CRSPin), quote!(#[cfg(feature="net")]))),
        (("eth", "RXD0"), (quote!(crate::eth::RXD0Pin), quote!(#[cfg(feature="net")]))),
        (("eth", "RXD1"), (quote!(crate::eth::RXD1Pin), quote!(#[cfg(feature="net")]))),
        (("eth", "TXD0"), (quote!(crate::eth::TXD0Pin), quote!(#[cfg(feature="net")]))),
        (("eth", "TXD1"), (quote!(crate::eth::TXD1Pin), quote!(#[cfg(feature="net")]))),
        (("eth", "TX_EN"), (quote!(crate::eth::TXEnPin), quote!(#[cfg(feature="net")]))),
        (("fmc", "A0"), (quote!(crate::fmc::A0Pin), quote!())),
        (("fmc", "A1"), (quote!(crate::fmc::A1Pin), quote!())),
        (("fmc", "A2"), (quote!(crate::fmc::A2Pin), quote!())),
        (("fmc", "A3"), (quote!(crate::fmc::A3Pin), quote!())),
        (("fmc", "A4"), (quote!(crate::fmc::A4Pin), quote!())),
        (("fmc", "A5"), (quote!(crate::fmc::A5Pin), quote!())),
        (("fmc", "A6"), (quote!(crate::fmc::A6Pin), quote!())),
        (("fmc", "A7"), (quote!(crate::fmc::A7Pin), quote!())),
        (("fmc", "A8"), (quote!(crate::fmc::A8Pin), quote!())),
        (("fmc", "A9"), (quote!(crate::fmc::A9Pin), quote!())),
        (("fmc", "A10"), (quote!(crate::fmc::A10Pin), quote!())),
        (("fmc", "A11"), (quote!(crate::fmc::A11Pin), quote!())),
        (("fmc", "A12"), (quote!(crate::fmc::A12Pin), quote!())),
        (("fmc", "A13"), (quote!(crate::fmc::A13Pin), quote!())),
        (("fmc", "A14"), (quote!(crate::fmc::A14Pin), quote!())),
        (("fmc", "A15"), (quote!(crate::fmc::A15Pin), quote!())),
        (("fmc", "A16"), (quote!(crate::fmc::A16Pin), quote!())),
        (("fmc", "A17"), (quote!(crate::fmc::A17Pin), quote!())),
        (("fmc", "A18"), (quote!(crate::fmc::A18Pin), quote!())),
        (("fmc", "A19"), (quote!(crate::fmc::A19Pin), quote!())),
        (("fmc", "A20"), (quote!(crate::fmc::A20Pin), quote!())),
        (("fmc", "A21"), (quote!(crate::fmc::A21Pin), quote!())),
        (("fmc", "A22"), (quote!(crate::fmc::A22Pin), quote!())),
        (("fmc", "A23"), (quote!(crate::fmc::A23Pin), quote!())),
        (("fmc", "A24"), (quote!(crate::fmc::A24Pin), quote!())),
        (("fmc", "A25"), (quote!(crate::fmc::A25Pin), quote!())),
        (("fmc", "D0"), (quote!(crate::fmc::D0Pin), quote!())),
        (("fmc", "D1"), (quote!(crate::fmc::D1Pin), quote!())),
        (("fmc", "D2"), (quote!(crate::fmc::D2Pin), quote!())),
        (("fmc", "D3"), (quote!(crate::fmc::D3Pin), quote!())),
        (("fmc", "D4"), (quote!(crate::fmc::D4Pin), quote!())),
        (("fmc", "D5"), (quote!(crate::fmc::D5Pin), quote!())),
        (("fmc", "D6"), (quote!(crate::fmc::D6Pin), quote!())),
        (("fmc", "D7"), (quote!(crate::fmc::D7Pin), quote!())),
        (("fmc", "D8"), (quote!(crate::fmc::D8Pin), quote!())),
        (("fmc", "D9"), (quote!(crate::fmc::D9Pin), quote!())),
        (("fmc", "D10"), (quote!(crate::fmc::D10Pin), quote!())),
        (("fmc", "D11"), (quote!(crate::fmc::D11Pin), quote!())),
        (("fmc", "D12"), (quote!(crate::fmc::D12Pin), quote!())),
        (("fmc", "D13"), (quote!(crate::fmc::D13Pin), quote!())),
        (("fmc", "D14"), (quote!(crate::fmc::D14Pin), quote!())),
        (("fmc", "D15"), (quote!(crate::fmc::D15Pin), quote!())),
        (("fmc", "D16"), (quote!(crate::fmc::D16Pin), quote!())),
        (("fmc", "D17"), (quote!(crate::fmc::D17Pin), quote!())),
        (("fmc", "D18"), (quote!(crate::fmc::D18Pin), quote!())),
        (("fmc", "D19"), (quote!(crate::fmc::D19Pin), quote!())),
        (("fmc", "D20"), (quote!(crate::fmc::D20Pin), quote!())),
        (("fmc", "D21"), (quote!(crate::fmc::D21Pin), quote!())),
        (("fmc", "D22"), (quote!(crate::fmc::D22Pin), quote!())),
        (("fmc", "D23"), (quote!(crate::fmc::D23Pin), quote!())),
        (("fmc", "D24"), (quote!(crate::fmc::D24Pin), quote!())),
        (("fmc", "D25"), (quote!(crate::fmc::D25Pin), quote!())),
        (("fmc", "D26"), (quote!(crate::fmc::D26Pin), quote!())),
        (("fmc", "D27"), (quote!(crate::fmc::D27Pin), quote!())),
        (("fmc", "D28"), (quote!(crate::fmc::D28Pin), quote!())),
        (("fmc", "D29"), (quote!(crate::fmc::D29Pin), quote!())),
        (("fmc", "D30"), (quote!(crate::fmc::D30Pin), quote!())),
        (("fmc", "D31"), (quote!(crate::fmc::D31Pin), quote!())),
        (("fmc", "DA0"), (quote!(crate::fmc::DA0Pin), quote!())),
        (("fmc", "DA1"), (quote!(crate::fmc::DA1Pin), quote!())),
        (("fmc", "DA2"), (quote!(crate::fmc::DA2Pin), quote!())),
        (("fmc", "DA3"), (quote!(crate::fmc::DA3Pin), quote!())),
        (("fmc", "DA4"), (quote!(crate::fmc::DA4Pin), quote!())),
        (("fmc", "DA5"), (quote!(crate::fmc::DA5Pin), quote!())),
        (("fmc", "DA6"), (quote!(crate::fmc::DA6Pin), quote!())),
        (("fmc", "DA7"), (quote!(crate::fmc::DA7Pin), quote!())),
        (("fmc", "DA8"), (quote!(crate::fmc::DA8Pin), quote!())),
        (("fmc", "DA9"), (quote!(crate::fmc::DA9Pin), quote!())),
        (("fmc", "DA10"), (quote!(crate::fmc::DA10Pin), quote!())),
        (("fmc", "DA11"), (quote!(crate::fmc::DA11Pin), quote!())),
        (("fmc", "DA12"), (quote!(crate::fmc::DA12Pin), quote!())),
        (("fmc", "DA13"), (quote!(crate::fmc::DA13Pin), quote!())),
        (("fmc", "DA14"), (quote!(crate::fmc::DA14Pin), quote!())),
        (("fmc", "DA15"), (quote!(crate::fmc::DA15Pin), quote!())),
        (("fmc", "SDNWE"), (quote!(crate::fmc::SDNWEPin), quote!())),
        (("fmc", "SDNCAS"), (quote!(crate::fmc::SDNCASPin), quote!())),
        (("fmc", "SDNRAS"), (quote!(crate::fmc::SDNRASPin), quote!())),
        (("fmc", "SDNE0"), (quote!(crate::fmc::SDNE0Pin), quote!())),
        (("fmc", "SDNE1"), (quote!(crate::fmc::SDNE1Pin), quote!())),
        (("fmc", "SDCKE0"), (quote!(crate::fmc::SDCKE0Pin), quote!())),
        (("fmc", "SDCKE1"), (quote!(crate::fmc::SDCKE1Pin), quote!())),
        (("fmc", "SDCLK"), (quote!(crate::fmc::SDCLKPin), quote!())),
        (("fmc", "NBL0"), (quote!(crate::fmc::NBL0Pin), quote!())),
        (("fmc", "NBL1"), (quote!(crate::fmc::NBL1Pin), quote!())),
        (("fmc", "NBL2"), (quote!(crate::fmc::NBL2Pin), quote!())),
        (("fmc", "NBL3"), (quote!(crate::fmc::NBL3Pin), quote!())),
        (("fmc", "INT"), (quote!(crate::fmc::INTPin), quote!())),
        (("fmc", "NL"), (quote!(crate::fmc::NLPin), quote!())),
        (("fmc", "NWAIT"), (quote!(crate::fmc::NWaitPin), quote!())),
        (("fmc", "NE1"), (quote!(crate::fmc::NE1Pin), quote!())),
        (("fmc", "NE2"), (quote!(crate::fmc::NE2Pin), quote!())),
        (("fmc", "NE3"), (quote!(crate::fmc::NE3Pin), quote!())),
        (("fmc", "NE4"), (quote!(crate::fmc::NE4Pin), quote!())),
        (("fmc", "NCE"), (quote!(crate::fmc::NCEPin), quote!())),
        (("fmc", "NOE"), (quote!(crate::fmc::NOEPin), quote!())),
        (("fmc", "NWE"), (quote!(crate::fmc::NWEPin), quote!())),
        (("fmc", "Clk"), (quote!(crate::fmc::ClkPin), quote!())),
        (("fmc", "BA0"), (quote!(crate::fmc::BA0Pin), quote!())),
        (("fmc", "BA1"), (quote!(crate::fmc::BA1Pin), quote!())),
        (("timer", "CH1"), (quote!(crate::pwm::Channel1Pin), quote!())),
        (("timer", "CH1N"), (quote!(crate::pwm::Channel1ComplementaryPin), quote!())),
        (("timer", "CH2"), (quote!(crate::pwm::Channel2Pin), quote!())),
        (("timer", "CH2N"), (quote!(crate::pwm::Channel2ComplementaryPin), quote!())),
        (("timer", "CH3"), (quote!(crate::pwm::Channel3Pin), quote!())),
        (("timer", "CH3N"), (quote!(crate::pwm::Channel3ComplementaryPin), quote!())),
        (("timer", "CH4"), (quote!(crate::pwm::Channel4Pin), quote!())),
        (("timer", "CH4N"), (quote!(crate::pwm::Channel4ComplementaryPin), quote!())),
        (("timer", "ETR"), (quote!(crate::pwm::ExternalTriggerPin), quote!())),
        (("timer", "BKIN"), (quote!(crate::pwm::BreakInputPin), quote!())),
        (("timer", "BKIN_COMP1"), (quote!(crate::pwm::BreakInputComparator1Pin), quote!())),
        (("timer", "BKIN_COMP2"), (quote!(crate::pwm::BreakInputComparator2Pin), quote!())),
        (("timer", "BKIN2"), (quote!(crate::pwm::BreakInput2Pin), quote!())),
        (("timer", "BKIN2_COMP1"), (quote!(crate::pwm::BreakInput2Comparator1Pin), quote!())),
        (("timer", "BKIN2_COMP2"), (quote!(crate::pwm::BreakInput2Comparator2Pin), quote!())),
        (("sdmmc", "CK"), (quote!(crate::sdmmc::CkPin), quote!())),
        (("sdmmc", "CMD"), (quote!(crate::sdmmc::CmdPin), quote!())),
        (("sdmmc", "D0"), (quote!(crate::sdmmc::D0Pin), quote!())),
        (("sdmmc", "D1"), (quote!(crate::sdmmc::D1Pin), quote!())),
        (("sdmmc", "D2"), (quote!(crate::sdmmc::D2Pin), quote!())),
        (("sdmmc", "D3"), (quote!(crate::sdmmc::D3Pin), quote!())),
        (("sdmmc", "D4"), (quote!(crate::sdmmc::D4Pin), quote!())),
        (("sdmmc", "D5"), (quote!(crate::sdmmc::D5Pin), quote!())),
        (("sdmmc", "D6"), (quote!(crate::sdmmc::D6Pin), quote!())),
        (("sdmmc", "D6"), (quote!(crate::sdmmc::D7Pin), quote!())),
        (("sdmmc", "D8"), (quote!(crate::sdmmc::D8Pin), quote!())),
    ].into();

    for p in METADATA.peripherals {
        if let Some(regs) = &p.registers {
            for pin in p.pins {
                let key = (regs.kind, pin.signal);
                if let Some((tr, cfgs)) = signals.get(&key) {
                    let mut peri = format_ident!("{}", p.name);
                    let pin_name = format_ident!("{}", pin.pin);
                    let af = pin.af.unwrap_or(0);

                    // MCO is special
                    if pin.signal.starts_with("MCO_") {
                        // Supported in H7 only for now
                        if regs.version.starts_with("h7") {
                            peri = format_ident!("{}", pin.signal.replace("_", ""));
                        } else {
                            continue;
                        }
                    }

                    g.extend(quote! {
                        #cfgs
                        pin_trait_impl!(#tr, #peri, #pin_name, #af);
                    })
                }

                // ADC is special
                if regs.kind == "adc" {
                    let peri = format_ident!("{}", p.name);
                    let pin_name = format_ident!("{}", pin.pin);
                    let ch: u8 = pin.signal.strip_prefix("IN").unwrap().parse().unwrap();

                    g.extend(quote! {
                        impl_adc_pin!( #peri, #pin_name, #ch);
                    })
                }

                // DAC is special
                if regs.kind == "dac" {
                    let peri = format_ident!("{}", p.name);
                    let pin_name = format_ident!("{}", pin.pin);
                    let ch: u8 = pin.signal.strip_prefix("OUT").unwrap().parse().unwrap();

                    g.extend(quote! {
                        impl_dac_pin!( #peri, #pin_name, #ch);
                    })
                }
            }
        }
    }

    // ========
    // Generate dma_trait_impl!

    let signals: HashMap<_, _> = [
        // (kind, signal) => trait
        (("usart", "RX"), quote!(crate::usart::RxDma)),
        (("usart", "TX"), quote!(crate::usart::TxDma)),
        (("spi", "RX"), quote!(crate::spi::RxDma)),
        (("spi", "TX"), quote!(crate::spi::TxDma)),
        (("i2c", "RX"), quote!(crate::i2c::RxDma)),
        (("i2c", "TX"), quote!(crate::i2c::TxDma)),
        (("dcmi", "DCMI"), quote!(crate::dcmi::FrameDma)),
        (("dcmi", "PSSI"), quote!(crate::dcmi::FrameDma)),
        // SDMMCv1 uses the same channel for both directions, so just implement for RX
        (("sdmmc", "RX"), quote!(crate::sdmmc::SdmmcDma)),
    ]
    .into();

    for p in METADATA.peripherals {
        if let Some(regs) = &p.registers {
            let mut dupe = HashSet::new();
            for ch in p.dma_channels {
                // Some chips have multiple request numbers for the same (peri, signal, channel) combos.
                // Ignore the dupes, picking the first one. Otherwise this causes conflicting trait impls
                let key = (ch.signal, ch.channel);
                if !dupe.insert(key) {
                    continue;
                }

                if let Some(tr) = signals.get(&(regs.kind, ch.signal)) {
                    let peri = format_ident!("{}", p.name);

                    let channel = if let Some(channel) = &ch.channel {
                        let channel = format_ident!("{}", channel);
                        quote!({channel: #channel})
                    } else if let Some(dmamux) = &ch.dmamux {
                        let dmamux = format_ident!("{}", dmamux);
                        quote!({dmamux: #dmamux})
                    } else {
                        unreachable!();
                    };

                    let request = if let Some(request) = ch.request {
                        let request = request as u8;
                        quote!(#request)
                    } else {
                        quote!(())
                    };

                    g.extend(quote! {
                        dma_trait_impl!(#tr, #peri, #channel, #request);
                    });
                }
            }
        }
    }

    // ========
    // Write foreach_foo! macrotables

    let mut interrupts_table: Vec<Vec<String>> = Vec::new();
    let mut peripherals_table: Vec<Vec<String>> = Vec::new();
    let mut pins_table: Vec<Vec<String>> = Vec::new();
    let mut dma_channels_table: Vec<Vec<String>> = Vec::new();

    let gpio_base = METADATA
        .peripherals
        .iter()
        .find(|p| p.name == "GPIOA")
        .unwrap()
        .address as u32;
    let gpio_stride = 0x400;

    for p in METADATA.peripherals {
        if let Some(regs) = &p.registers {
            if regs.kind == "gpio" {
                let port_letter = p.name.chars().skip(4).next().unwrap();
                assert_eq!(0, (p.address as u32 - gpio_base) % gpio_stride);
                let port_num = (p.address as u32 - gpio_base) / gpio_stride;

                for pin_num in 0u32..16 {
                    let pin_name = format!("P{}{}", port_letter, pin_num);
                    pins_table.push(vec![
                        pin_name,
                        p.name.to_string(),
                        port_num.to_string(),
                        pin_num.to_string(),
                        format!("EXTI{}", pin_num),
                    ]);
                }
            }

            for irq in p.interrupts {
                let mut row = Vec::new();
                row.push(p.name.to_string());
                row.push(regs.kind.to_string());
                row.push(regs.block.to_string());
                row.push(irq.signal.to_string());
                row.push(irq.interrupt.to_ascii_uppercase());
                interrupts_table.push(row)
            }

            let mut row = Vec::new();
            row.push(regs.kind.to_string());
            row.push(p.name.to_string());
            peripherals_table.push(row);
        }
    }

    let mut dma_channel_count: usize = 0;
    let mut bdma_channel_count: usize = 0;

    for ch in METADATA.dma_channels {
        let mut row = Vec::new();
        let dma_peri = METADATA
            .peripherals
            .iter()
            .find(|p| p.name == ch.dma)
            .unwrap();
        let bi = dma_peri.registers.as_ref().unwrap();

        let num;
        match bi.kind {
            "dma" => {
                num = dma_channel_count;
                dma_channel_count += 1;
            }
            "bdma" => {
                num = bdma_channel_count;
                bdma_channel_count += 1;
            }
            _ => panic!("bad dma channel kind {}", bi.kind),
        }

        row.push(ch.name.to_string());
        row.push(ch.dma.to_string());
        row.push(bi.kind.to_string());
        row.push(ch.channel.to_string());
        row.push(num.to_string());
        if let Some(dmamux) = &ch.dmamux {
            let dmamux_channel = ch.dmamux_channel.unwrap();
            row.push(format!(
                "{{dmamux: {}, dmamux_channel: {}}}",
                dmamux, dmamux_channel
            ));
        } else {
            row.push("{}".to_string());
        }

        dma_channels_table.push(row);
    }

    g.extend(quote! {
        pub(crate) const DMA_CHANNEL_COUNT: usize = #dma_channel_count;
        pub(crate) const BDMA_CHANNEL_COUNT: usize = #bdma_channel_count;
    });

    for irq in METADATA.interrupts {
        let name = irq.name.to_ascii_uppercase();
        interrupts_table.push(vec![name.clone()]);
        if name.contains("EXTI") {
            interrupts_table.push(vec!["EXTI".to_string(), name.clone()]);
        }
    }

    let mut m = String::new();

    make_table(&mut m, "foreach_interrupt", &interrupts_table);
    make_table(&mut m, "foreach_peripheral", &peripherals_table);
    make_table(&mut m, "foreach_pin", &pins_table);
    make_table(&mut m, "foreach_dma_channel", &dma_channels_table);

    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("_macros.rs").to_string_lossy().to_string();
    fs::write(out_file, m).unwrap();

    // ========
    // Write generated.rs

    let out_file = out_dir.join("_generated.rs").to_string_lossy().to_string();
    fs::write(out_file, g.to_string()).unwrap();

    // ========
    // Multicore

    let mut s = chip_name.split('_');
    let mut chip_name: String = s.next().unwrap().to_string();
    let core_name = if let Some(c) = s.next() {
        if !c.starts_with("CM") {
            chip_name.push('_');
            chip_name.push_str(c);
            None
        } else {
            Some(c)
        }
    } else {
        None
    };

    if let Some(core) = core_name {
        println!(
            "cargo:rustc-cfg={}_{}",
            &chip_name[..chip_name.len() - 2],
            core
        );
    } else {
        println!("cargo:rustc-cfg={}", &chip_name[..chip_name.len() - 2]);
    }

    // ========
    // stm32f3 wildcard features used in RCC

    if chip_name.starts_with("stm32f3") {
        println!("cargo:rustc-cfg={}x{}", &chip_name[..9], &chip_name[10..11]);
    }

    // =======
    // Features for targeting groups of chips

    println!("cargo:rustc-cfg={}", &chip_name[..7]); // stm32f4
    println!("cargo:rustc-cfg={}", &chip_name[..9]); // stm32f429
    println!("cargo:rustc-cfg={}x", &chip_name[..8]); // stm32f42x
    println!("cargo:rustc-cfg={}x{}", &chip_name[..7], &chip_name[8..9]); // stm32f4x9

    // ========
    // Handle time-driver-XXXX features.

    let time_driver = match env::vars()
        .map(|(a, _)| a)
        .filter(|x| x.starts_with("CARGO_FEATURE_TIME_DRIVER_"))
        .get_one()
    {
        Ok(x) => Some(
            x.strip_prefix("CARGO_FEATURE_TIME_DRIVER_")
                .unwrap()
                .to_ascii_lowercase(),
        ),
        Err(GetOneError::None) => None,
        Err(GetOneError::Multiple) => panic!("Multiple stm32xx Cargo features enabled"),
    };

    match time_driver.as_ref().map(|x| x.as_ref()) {
        None => {}
        Some("tim2") => println!("cargo:rustc-cfg=time_driver_tim2"),
        Some("tim3") => println!("cargo:rustc-cfg=time_driver_tim3"),
        Some("tim4") => println!("cargo:rustc-cfg=time_driver_tim4"),
        Some("tim5") => println!("cargo:rustc-cfg=time_driver_tim5"),
        Some("tim12") => println!("cargo:rustc-cfg=time_driver_tim12"),
        Some("tim15") => println!("cargo:rustc-cfg=time_driver_tim15"),
        Some("any") => {
            if singletons.contains(&"TIM2".to_string()) {
                println!("cargo:rustc-cfg=time_driver_tim2");
            } else if singletons.contains(&"TIM3".to_string()) {
                println!("cargo:rustc-cfg=time_driver_tim3");
            } else if singletons.contains(&"TIM4".to_string()) {
                println!("cargo:rustc-cfg=time_driver_tim4");
            } else if singletons.contains(&"TIM5".to_string()) {
                println!("cargo:rustc-cfg=time_driver_tim5");
            } else if singletons.contains(&"TIM12".to_string()) {
                println!("cargo:rustc-cfg=time_driver_tim12");
            } else if singletons.contains(&"TIM15".to_string()) {
                println!("cargo:rustc-cfg=time_driver_tim15");
            } else {
                panic!("time-driver-any requested, but the chip doesn't have TIM2, TIM3, TIM4, TIM5, TIM12 or TIM15.")
            }
        }
        _ => panic!("unknown time_driver {:?}", time_driver),
    }

    // Handle time-driver-XXXX features.
    if env::var("CARGO_FEATURE_TIME_DRIVER_ANY").is_ok() {}
    println!("cargo:rustc-cfg={}", &chip_name[..chip_name.len() - 2]);

    println!("cargo:rerun-if-changed=build.rs");
}

enum GetOneError {
    None,
    Multiple,
}

trait IteratorExt: Iterator {
    fn get_one(self) -> Result<Self::Item, GetOneError>;
}

impl<T: Iterator> IteratorExt for T {
    fn get_one(mut self) -> Result<Self::Item, GetOneError> {
        match self.next() {
            None => Err(GetOneError::None),
            Some(res) => match self.next() {
                Some(_) => Err(GetOneError::Multiple),
                None => Ok(res),
            },
        }
    }
}

fn make_table(out: &mut String, name: &str, data: &Vec<Vec<String>>) {
    write!(
        out,
        "#[allow(unused)]
macro_rules! {} {{
    ($($pat:tt => $code:tt;)*) => {{
        macro_rules! __{}_inner {{
            $(($pat) => $code;)*
            ($_:tt) => {{}}
        }}
",
        name, name
    )
    .unwrap();

    for row in data {
        write!(out, "        __{}_inner!(({}));\n", name, row.join(",")).unwrap();
    }

    write!(
        out,
        "    }};
}}"
    )
    .unwrap();
}
