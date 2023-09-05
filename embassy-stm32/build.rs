use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::path::PathBuf;
use std::{env, fs};

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use stm32_metapac::metadata::{MemoryRegionKind, METADATA};

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
                    if r.version.starts_with("h5") || r.version.starts_with("h7") || r.version.starts_with("f4") {
                        singletons.push("MCO1".to_string());
                        singletons.push("MCO2".to_string());
                    }
                    if r.version.starts_with("l4") {
                        singletons.push("MCO".to_string());
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

    let time_driver_singleton = match time_driver.as_ref().map(|x| x.as_ref()) {
        None => "",
        Some("tim2") => "TIM2",
        Some("tim3") => "TIM3",
        Some("tim4") => "TIM4",
        Some("tim5") => "TIM5",
        Some("tim12") => "TIM12",
        Some("tim15") => "TIM15",
        Some("any") => {
            if singletons.contains(&"TIM2".to_string()) {
                "TIM2"
            } else if singletons.contains(&"TIM3".to_string()) {
                "TIM3"
            } else if singletons.contains(&"TIM4".to_string()) {
                "TIM4"
            } else if singletons.contains(&"TIM5".to_string()) {
                "TIM5"
            } else if singletons.contains(&"TIM12".to_string()) {
                "TIM12"
            } else if singletons.contains(&"TIM15".to_string()) {
                "TIM15"
            } else {
                panic!("time-driver-any requested, but the chip doesn't have TIM2, TIM3, TIM4, TIM5, TIM12 or TIM15.")
            }
        }
        _ => panic!("unknown time_driver {:?}", time_driver),
    };

    if !time_driver_singleton.is_empty() {
        println!("cargo:rustc-cfg=time_driver_{}", time_driver_singleton.to_lowercase());
    }

    // ========
    // Write singletons

    let mut g = TokenStream::new();

    let singleton_tokens: Vec<_> = singletons.iter().map(|s| format_ident!("{}", s)).collect();

    g.extend(quote! {
        embassy_hal_internal::peripherals_definition!(#(#singleton_tokens),*);
    });

    let singleton_tokens: Vec<_> = singletons
        .iter()
        .filter(|s| *s != &time_driver_singleton.to_string())
        .map(|s| format_ident!("{}", s))
        .collect();

    g.extend(quote! {
        embassy_hal_internal::peripherals_struct!(#(#singleton_tokens),*);
    });

    // ========
    // Generate interrupt declarations

    let mut irqs = Vec::new();
    for irq in METADATA.interrupts {
        irqs.push(format_ident!("{}", irq.name));
    }

    g.extend(quote! {
        embassy_hal_internal::interrupt_mod!(
            #(
                #irqs,
            )*
        );
    });

    // ========
    // Generate FLASH regions
    let mut flash_regions = TokenStream::new();
    let flash_memory_regions: Vec<_> = METADATA
        .memory
        .iter()
        .filter(|x| x.kind == MemoryRegionKind::Flash && x.settings.is_some())
        .collect();
    for region in flash_memory_regions.iter() {
        let region_name = format_ident!("{}", get_flash_region_name(region.name));
        let bank_variant = format_ident!(
            "{}",
            if region.name.starts_with("BANK_1") {
                "Bank1"
            } else if region.name.starts_with("BANK_2") {
                "Bank2"
            } else if region.name == "OTP" {
                "Otp"
            } else {
                continue;
            }
        );
        let base = region.address;
        let size = region.size;
        let settings = region.settings.as_ref().unwrap();
        let erase_size = settings.erase_size;
        let write_size = settings.write_size;
        let erase_value = settings.erase_value;

        flash_regions.extend(quote! {
            pub const #region_name: crate::flash::FlashRegion = crate::flash::FlashRegion {
                bank: crate::flash::FlashBank::#bank_variant,
                base: #base,
                size: #size,
                erase_size: #erase_size,
                write_size: #write_size,
                erase_value: #erase_value,
                _ensure_internal: (),
            };
        });

        let region_type = format_ident!("{}", get_flash_region_type_name(region.name));
        flash_regions.extend(quote! {
            #[cfg(flash)]
            pub struct #region_type<'d, MODE = crate::flash::Async>(pub &'static crate::flash::FlashRegion, pub(crate) embassy_hal_internal::PeripheralRef<'d, crate::peripherals::FLASH>, pub(crate) core::marker::PhantomData<MODE>);
        });
    }

    let (fields, (inits, region_names)): (Vec<TokenStream>, (Vec<TokenStream>, Vec<Ident>)) = flash_memory_regions
        .iter()
        .map(|f| {
            let region_name = get_flash_region_name(f.name);
            let field_name = format_ident!("{}", region_name.to_lowercase());
            let field_type = format_ident!("{}", get_flash_region_type_name(f.name));
            let field = quote! {
                pub #field_name: #field_type<'d, MODE>
            };
            let region_name = format_ident!("{}", region_name);
            let init = quote! {
                #field_name: #field_type(&#region_name, unsafe { p.clone_unchecked()}, core::marker::PhantomData)
            };

            (field, (init, region_name))
        })
        .unzip();

    let regions_len = flash_memory_regions.len();
    flash_regions.extend(quote! {
        #[cfg(flash)]
        pub struct FlashLayout<'d, MODE = crate::flash::Async> {
            #(#fields),*,
            _mode: core::marker::PhantomData<MODE>,
        }

        #[cfg(flash)]
        impl<'d, MODE> FlashLayout<'d, MODE> {
            pub(crate) fn new(p: embassy_hal_internal::PeripheralRef<'d, crate::peripherals::FLASH>) -> Self {
                Self {
                    #(#inits),*,
                    _mode: core::marker::PhantomData,
                }
            }
        }

        pub const FLASH_REGIONS: [&crate::flash::FlashRegion; #regions_len] = [
            #(&#region_names),*
        ];
    });

    let max_erase_size = flash_memory_regions
        .iter()
        .map(|region| region.settings.as_ref().unwrap().erase_size)
        .max()
        .unwrap();

    g.extend(quote! { pub const MAX_ERASE_SIZE: usize = #max_erase_size as usize; });

    g.extend(quote! { pub mod flash_regions { #flash_regions } });

    // ========
    // Generate DMA IRQs.

    let mut dma_irqs: HashMap<&str, Vec<(&str, &str, &str)>> = HashMap::new();

    for p in METADATA.peripherals {
        if let Some(r) = &p.registers {
            if r.kind == "dma" || r.kind == "bdma" || r.kind == "gpdma" {
                if p.name == "BDMA1" {
                    // BDMA1 in H7 doesn't use DMAMUX, which breaks
                    continue;
                }
                for irq in p.interrupts {
                    dma_irqs
                        .entry(irq.interrupt)
                        .or_default()
                        .push((r.kind, p.name, irq.signal));
                }
            }
        }
    }

    for (irq, channels) in dma_irqs {
        let irq = format_ident!("{}", irq);

        let xdma = format_ident!("{}", channels[0].0);
        let channels = channels.iter().map(|(_, dma, ch)| format_ident!("{}_{}", dma, ch));

        g.extend(quote! {
            #[cfg(feature = "rt")]
            #[crate::interrupt]
            unsafe fn #irq () {
                #(
                    <crate::peripherals::#channels as crate::dma::#xdma::sealed::Channel>::on_irq();
                )*
            }
        });
    }

    // ========
    // Generate RccPeripheral impls

    let refcounted_peripherals = HashSet::from(["usart", "adc"]);
    let mut refcount_statics = HashSet::new();

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
                        critical_section::with(|_| {
                            crate::pac::RCC.#rst_reg().modify(|w| w.#set_rst_field(true));
                            crate::pac::RCC.#rst_reg().modify(|w| w.#set_rst_field(false));
                        });
                    }
                }
                None => TokenStream::new(),
            };

            let after_enable = if chip_name.starts_with("stm32f2") {
                // Errata: ES0005 - 2.1.11 Delay after an RCC peripheral clock enabling
                quote! {
                    cortex_m::asm::dsb();
                }
            } else {
                TokenStream::new()
            };

            let ptype = if let Some(reg) = &p.registers { reg.kind } else { "" };
            let pname = format_ident!("{}", p.name);
            let clk = format_ident!("{}", rcc.clock.to_ascii_lowercase());
            let en_reg = format_ident!("{}", en.register.to_ascii_lowercase());
            let set_en_field = format_ident!("set_{}", en.field.to_ascii_lowercase());

            let (before_enable, before_disable) = if refcounted_peripherals.contains(ptype) {
                let refcount_static =
                    format_ident!("{}_{}", en.register.to_ascii_uppercase(), en.field.to_ascii_uppercase());

                refcount_statics.insert(refcount_static.clone());

                (
                    quote! {
                        unsafe { refcount_statics::#refcount_static += 1 };
                        if unsafe { refcount_statics::#refcount_static } > 1 {
                            return;
                        }
                    },
                    quote! {
                        unsafe { refcount_statics::#refcount_static -= 1 };
                        if unsafe { refcount_statics::#refcount_static } > 0  {
                            return;
                        }
                    },
                )
            } else {
                (TokenStream::new(), TokenStream::new())
            };

            g.extend(quote! {
                impl crate::rcc::sealed::RccPeripheral for peripherals::#pname {
                    fn frequency() -> crate::time::Hertz {
                        unsafe { crate::rcc::get_freqs().#clk }
                    }
                    fn enable() {
                        critical_section::with(|_| {
                            #before_enable
                            #[cfg(feature = "low-power")]
                            crate::rcc::clock_refcount_add();
                            crate::pac::RCC.#en_reg().modify(|w| w.#set_en_field(true));
                            #after_enable
                        })
                    }
                    fn disable() {
                        critical_section::with(|_| {
                            #before_disable
                            crate::pac::RCC.#en_reg().modify(|w| w.#set_en_field(false));
                            #[cfg(feature = "low-power")]
                            crate::rcc::clock_refcount_sub();
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

    let mut refcount_mod = TokenStream::new();
    for refcount_static in refcount_statics {
        refcount_mod.extend(quote! {
            pub(crate) static mut #refcount_static: u8 = 0;
        });
    }

    g.extend(quote! {
        mod refcount_statics {
            #refcount_mod
        }
    });

    // ========
    // Generate fns to enable GPIO, DMA in RCC

    for kind in ["dma", "bdma", "dmamux", "gpdma", "gpio"] {
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
        // (kind, signal) => trait
        (("usart", "TX"), quote!(crate::usart::TxPin)),
        (("usart", "RX"), quote!(crate::usart::RxPin)),
        (("usart", "CTS"), quote!(crate::usart::CtsPin)),
        (("usart", "RTS"), quote!(crate::usart::RtsPin)),
        (("usart", "CK"), quote!(crate::usart::CkPin)),
        (("usart", "DE"), quote!(crate::usart::DePin)),
        (("lpuart", "TX"), quote!(crate::usart::TxPin)),
        (("lpuart", "RX"), quote!(crate::usart::RxPin)),
        (("lpuart", "CTS"), quote!(crate::usart::CtsPin)),
        (("lpuart", "RTS"), quote!(crate::usart::RtsPin)),
        (("lpuart", "CK"), quote!(crate::usart::CkPin)),
        (("lpuart", "DE"), quote!(crate::usart::DePin)),
        (("spi", "SCK"), quote!(crate::spi::SckPin)),
        (("spi", "MOSI"), quote!(crate::spi::MosiPin)),
        (("spi", "MISO"), quote!(crate::spi::MisoPin)),
        (("spi", "NSS"), quote!(crate::spi::CsPin)),
        (("spi", "I2S_MCK"), quote!(crate::spi::MckPin)),
        (("spi", "I2S_CK"), quote!(crate::spi::CkPin)),
        (("spi", "I2S_WS"), quote!(crate::spi::WsPin)),
        (("i2c", "SDA"), quote!(crate::i2c::SdaPin)),
        (("i2c", "SCL"), quote!(crate::i2c::SclPin)),
        (("rcc", "MCO_1"), quote!(crate::rcc::McoPin)),
        (("rcc", "MCO_2"), quote!(crate::rcc::McoPin)),
        (("rcc", "MCO"), quote!(crate::rcc::McoPin)),
        (("dcmi", "D0"), quote!(crate::dcmi::D0Pin)),
        (("dcmi", "D1"), quote!(crate::dcmi::D1Pin)),
        (("dcmi", "D2"), quote!(crate::dcmi::D2Pin)),
        (("dcmi", "D3"), quote!(crate::dcmi::D3Pin)),
        (("dcmi", "D4"), quote!(crate::dcmi::D4Pin)),
        (("dcmi", "D5"), quote!(crate::dcmi::D5Pin)),
        (("dcmi", "D6"), quote!(crate::dcmi::D6Pin)),
        (("dcmi", "D7"), quote!(crate::dcmi::D7Pin)),
        (("dcmi", "D8"), quote!(crate::dcmi::D8Pin)),
        (("dcmi", "D9"), quote!(crate::dcmi::D9Pin)),
        (("dcmi", "D10"), quote!(crate::dcmi::D10Pin)),
        (("dcmi", "D11"), quote!(crate::dcmi::D11Pin)),
        (("dcmi", "D12"), quote!(crate::dcmi::D12Pin)),
        (("dcmi", "D13"), quote!(crate::dcmi::D13Pin)),
        (("dcmi", "HSYNC"), quote!(crate::dcmi::HSyncPin)),
        (("dcmi", "VSYNC"), quote!(crate::dcmi::VSyncPin)),
        (("dcmi", "PIXCLK"), quote!(crate::dcmi::PixClkPin)),
        (("usb", "DP"), quote!(crate::usb::DpPin)),
        (("usb", "DM"), quote!(crate::usb::DmPin)),
        (("otg", "DP"), quote!(crate::usb_otg::DpPin)),
        (("otg", "DM"), quote!(crate::usb_otg::DmPin)),
        (("otg", "ULPI_CK"), quote!(crate::usb_otg::UlpiClkPin)),
        (("otg", "ULPI_DIR"), quote!(crate::usb_otg::UlpiDirPin)),
        (("otg", "ULPI_NXT"), quote!(crate::usb_otg::UlpiNxtPin)),
        (("otg", "ULPI_STP"), quote!(crate::usb_otg::UlpiStpPin)),
        (("otg", "ULPI_D0"), quote!(crate::usb_otg::UlpiD0Pin)),
        (("otg", "ULPI_D1"), quote!(crate::usb_otg::UlpiD1Pin)),
        (("otg", "ULPI_D2"), quote!(crate::usb_otg::UlpiD2Pin)),
        (("otg", "ULPI_D3"), quote!(crate::usb_otg::UlpiD3Pin)),
        (("otg", "ULPI_D4"), quote!(crate::usb_otg::UlpiD4Pin)),
        (("otg", "ULPI_D5"), quote!(crate::usb_otg::UlpiD5Pin)),
        (("otg", "ULPI_D6"), quote!(crate::usb_otg::UlpiD6Pin)),
        (("otg", "ULPI_D7"), quote!(crate::usb_otg::UlpiD7Pin)),
        (("can", "TX"), quote!(crate::can::TxPin)),
        (("can", "RX"), quote!(crate::can::RxPin)),
        (("eth", "REF_CLK"), quote!(crate::eth::RefClkPin)),
        (("eth", "MDIO"), quote!(crate::eth::MDIOPin)),
        (("eth", "MDC"), quote!(crate::eth::MDCPin)),
        (("eth", "CRS_DV"), quote!(crate::eth::CRSPin)),
        (("eth", "RXD0"), quote!(crate::eth::RXD0Pin)),
        (("eth", "RXD1"), quote!(crate::eth::RXD1Pin)),
        (("eth", "TXD0"), quote!(crate::eth::TXD0Pin)),
        (("eth", "TXD1"), quote!(crate::eth::TXD1Pin)),
        (("eth", "TX_EN"), quote!(crate::eth::TXEnPin)),
        (("fmc", "A0"), quote!(crate::fmc::A0Pin)),
        (("fmc", "A1"), quote!(crate::fmc::A1Pin)),
        (("fmc", "A2"), quote!(crate::fmc::A2Pin)),
        (("fmc", "A3"), quote!(crate::fmc::A3Pin)),
        (("fmc", "A4"), quote!(crate::fmc::A4Pin)),
        (("fmc", "A5"), quote!(crate::fmc::A5Pin)),
        (("fmc", "A6"), quote!(crate::fmc::A6Pin)),
        (("fmc", "A7"), quote!(crate::fmc::A7Pin)),
        (("fmc", "A8"), quote!(crate::fmc::A8Pin)),
        (("fmc", "A9"), quote!(crate::fmc::A9Pin)),
        (("fmc", "A10"), quote!(crate::fmc::A10Pin)),
        (("fmc", "A11"), quote!(crate::fmc::A11Pin)),
        (("fmc", "A12"), quote!(crate::fmc::A12Pin)),
        (("fmc", "A13"), quote!(crate::fmc::A13Pin)),
        (("fmc", "A14"), quote!(crate::fmc::A14Pin)),
        (("fmc", "A15"), quote!(crate::fmc::A15Pin)),
        (("fmc", "A16"), quote!(crate::fmc::A16Pin)),
        (("fmc", "A17"), quote!(crate::fmc::A17Pin)),
        (("fmc", "A18"), quote!(crate::fmc::A18Pin)),
        (("fmc", "A19"), quote!(crate::fmc::A19Pin)),
        (("fmc", "A20"), quote!(crate::fmc::A20Pin)),
        (("fmc", "A21"), quote!(crate::fmc::A21Pin)),
        (("fmc", "A22"), quote!(crate::fmc::A22Pin)),
        (("fmc", "A23"), quote!(crate::fmc::A23Pin)),
        (("fmc", "A24"), quote!(crate::fmc::A24Pin)),
        (("fmc", "A25"), quote!(crate::fmc::A25Pin)),
        (("fmc", "D0"), quote!(crate::fmc::D0Pin)),
        (("fmc", "D1"), quote!(crate::fmc::D1Pin)),
        (("fmc", "D2"), quote!(crate::fmc::D2Pin)),
        (("fmc", "D3"), quote!(crate::fmc::D3Pin)),
        (("fmc", "D4"), quote!(crate::fmc::D4Pin)),
        (("fmc", "D5"), quote!(crate::fmc::D5Pin)),
        (("fmc", "D6"), quote!(crate::fmc::D6Pin)),
        (("fmc", "D7"), quote!(crate::fmc::D7Pin)),
        (("fmc", "D8"), quote!(crate::fmc::D8Pin)),
        (("fmc", "D9"), quote!(crate::fmc::D9Pin)),
        (("fmc", "D10"), quote!(crate::fmc::D10Pin)),
        (("fmc", "D11"), quote!(crate::fmc::D11Pin)),
        (("fmc", "D12"), quote!(crate::fmc::D12Pin)),
        (("fmc", "D13"), quote!(crate::fmc::D13Pin)),
        (("fmc", "D14"), quote!(crate::fmc::D14Pin)),
        (("fmc", "D15"), quote!(crate::fmc::D15Pin)),
        (("fmc", "D16"), quote!(crate::fmc::D16Pin)),
        (("fmc", "D17"), quote!(crate::fmc::D17Pin)),
        (("fmc", "D18"), quote!(crate::fmc::D18Pin)),
        (("fmc", "D19"), quote!(crate::fmc::D19Pin)),
        (("fmc", "D20"), quote!(crate::fmc::D20Pin)),
        (("fmc", "D21"), quote!(crate::fmc::D21Pin)),
        (("fmc", "D22"), quote!(crate::fmc::D22Pin)),
        (("fmc", "D23"), quote!(crate::fmc::D23Pin)),
        (("fmc", "D24"), quote!(crate::fmc::D24Pin)),
        (("fmc", "D25"), quote!(crate::fmc::D25Pin)),
        (("fmc", "D26"), quote!(crate::fmc::D26Pin)),
        (("fmc", "D27"), quote!(crate::fmc::D27Pin)),
        (("fmc", "D28"), quote!(crate::fmc::D28Pin)),
        (("fmc", "D29"), quote!(crate::fmc::D29Pin)),
        (("fmc", "D30"), quote!(crate::fmc::D30Pin)),
        (("fmc", "D31"), quote!(crate::fmc::D31Pin)),
        (("fmc", "DA0"), quote!(crate::fmc::DA0Pin)),
        (("fmc", "DA1"), quote!(crate::fmc::DA1Pin)),
        (("fmc", "DA2"), quote!(crate::fmc::DA2Pin)),
        (("fmc", "DA3"), quote!(crate::fmc::DA3Pin)),
        (("fmc", "DA4"), quote!(crate::fmc::DA4Pin)),
        (("fmc", "DA5"), quote!(crate::fmc::DA5Pin)),
        (("fmc", "DA6"), quote!(crate::fmc::DA6Pin)),
        (("fmc", "DA7"), quote!(crate::fmc::DA7Pin)),
        (("fmc", "DA8"), quote!(crate::fmc::DA8Pin)),
        (("fmc", "DA9"), quote!(crate::fmc::DA9Pin)),
        (("fmc", "DA10"), quote!(crate::fmc::DA10Pin)),
        (("fmc", "DA11"), quote!(crate::fmc::DA11Pin)),
        (("fmc", "DA12"), quote!(crate::fmc::DA12Pin)),
        (("fmc", "DA13"), quote!(crate::fmc::DA13Pin)),
        (("fmc", "DA14"), quote!(crate::fmc::DA14Pin)),
        (("fmc", "DA15"), quote!(crate::fmc::DA15Pin)),
        (("fmc", "SDNWE"), quote!(crate::fmc::SDNWEPin)),
        (("fmc", "SDNCAS"), quote!(crate::fmc::SDNCASPin)),
        (("fmc", "SDNRAS"), quote!(crate::fmc::SDNRASPin)),
        (("fmc", "SDNE0"), quote!(crate::fmc::SDNE0Pin)),
        (("fmc", "SDNE1"), quote!(crate::fmc::SDNE1Pin)),
        (("fmc", "SDCKE0"), quote!(crate::fmc::SDCKE0Pin)),
        (("fmc", "SDCKE1"), quote!(crate::fmc::SDCKE1Pin)),
        (("fmc", "SDCLK"), quote!(crate::fmc::SDCLKPin)),
        (("fmc", "NBL0"), quote!(crate::fmc::NBL0Pin)),
        (("fmc", "NBL1"), quote!(crate::fmc::NBL1Pin)),
        (("fmc", "NBL2"), quote!(crate::fmc::NBL2Pin)),
        (("fmc", "NBL3"), quote!(crate::fmc::NBL3Pin)),
        (("fmc", "INT"), quote!(crate::fmc::INTPin)),
        (("fmc", "NL"), quote!(crate::fmc::NLPin)),
        (("fmc", "NWAIT"), quote!(crate::fmc::NWaitPin)),
        (("fmc", "NE1"), quote!(crate::fmc::NE1Pin)),
        (("fmc", "NE2"), quote!(crate::fmc::NE2Pin)),
        (("fmc", "NE3"), quote!(crate::fmc::NE3Pin)),
        (("fmc", "NE4"), quote!(crate::fmc::NE4Pin)),
        (("fmc", "NCE"), quote!(crate::fmc::NCEPin)),
        (("fmc", "NOE"), quote!(crate::fmc::NOEPin)),
        (("fmc", "NWE"), quote!(crate::fmc::NWEPin)),
        (("fmc", "Clk"), quote!(crate::fmc::ClkPin)),
        (("fmc", "BA0"), quote!(crate::fmc::BA0Pin)),
        (("fmc", "BA1"), quote!(crate::fmc::BA1Pin)),
        (("timer", "CH1"), quote!(crate::timer::Channel1Pin)),
        (("timer", "CH1N"), quote!(crate::timer::Channel1ComplementaryPin)),
        (("timer", "CH2"), quote!(crate::timer::Channel2Pin)),
        (("timer", "CH2N"), quote!(crate::timer::Channel2ComplementaryPin)),
        (("timer", "CH3"), quote!(crate::timer::Channel3Pin)),
        (("timer", "CH3N"), quote!(crate::timer::Channel3ComplementaryPin)),
        (("timer", "CH4"), quote!(crate::timer::Channel4Pin)),
        (("timer", "CH4N"), quote!(crate::timer::Channel4ComplementaryPin)),
        (("timer", "ETR"), quote!(crate::timer::ExternalTriggerPin)),
        (("timer", "BKIN"), quote!(crate::timer::BreakInputPin)),
        (("timer", "BKIN_COMP1"), quote!(crate::timer::BreakInputComparator1Pin)),
        (("timer", "BKIN_COMP2"), quote!(crate::timer::BreakInputComparator2Pin)),
        (("timer", "BKIN2"), quote!(crate::timer::BreakInput2Pin)),
        (("timer", "BKIN2_COMP1"), quote!(crate::timer::BreakInput2Comparator1Pin)),
        (("timer", "BKIN2_COMP2"), quote!(crate::timer::BreakInput2Comparator2Pin)),
        (("hrtim", "CHA1"), quote!(crate::hrtim::ChannelAPin)),
        (("hrtim", "CHA2"), quote!(crate::hrtim::ChannelAComplementaryPin)),
        (("hrtim", "CHB1"), quote!(crate::hrtim::ChannelBPin)),
        (("hrtim", "CHB2"), quote!(crate::hrtim::ChannelBComplementaryPin)),
        (("hrtim", "CHC1"), quote!(crate::hrtim::ChannelCPin)),
        (("hrtim", "CHC2"), quote!(crate::hrtim::ChannelCComplementaryPin)),
        (("hrtim", "CHD1"), quote!(crate::hrtim::ChannelDPin)),
        (("hrtim", "CHD2"), quote!(crate::hrtim::ChannelDComplementaryPin)),
        (("hrtim", "CHE1"), quote!(crate::hrtim::ChannelEPin)),
        (("hrtim", "CHE2"), quote!(crate::hrtim::ChannelEComplementaryPin)),
        (("hrtim", "CHF1"), quote!(crate::hrtim::ChannelFPin)),
        (("hrtim", "CHF2"), quote!(crate::hrtim::ChannelFComplementaryPin)),
        (("sdmmc", "CK"), quote!(crate::sdmmc::CkPin)),
        (("sdmmc", "CMD"), quote!(crate::sdmmc::CmdPin)),
        (("sdmmc", "D0"), quote!(crate::sdmmc::D0Pin)),
        (("sdmmc", "D1"), quote!(crate::sdmmc::D1Pin)),
        (("sdmmc", "D2"), quote!(crate::sdmmc::D2Pin)),
        (("sdmmc", "D3"), quote!(crate::sdmmc::D3Pin)),
        (("sdmmc", "D4"), quote!(crate::sdmmc::D4Pin)),
        (("sdmmc", "D5"), quote!(crate::sdmmc::D5Pin)),
        (("sdmmc", "D6"), quote!(crate::sdmmc::D6Pin)),
        (("sdmmc", "D6"), quote!(crate::sdmmc::D7Pin)),
        (("sdmmc", "D8"), quote!(crate::sdmmc::D8Pin)),
        (("quadspi", "BK1_IO0"), quote!(crate::qspi::D0Pin)),
        (("quadspi", "BK1_IO1"), quote!(crate::qspi::D1Pin)),
        (("quadspi", "BK1_IO2"), quote!(crate::qspi::D2Pin)),
        (("quadspi", "BK1_IO3"), quote!(crate::qspi::D3Pin)),
        (("quadspi", "CLK"), quote!(crate::qspi::SckPin)),
        (("quadspi", "BK1_NCS"), quote!(crate::qspi::NSSPin)),
    ].into();

    for p in METADATA.peripherals {
        if let Some(regs) = &p.registers {
            for pin in p.pins {
                let key = (regs.kind, pin.signal);
                if let Some(tr) = signals.get(&key) {
                    let mut peri = format_ident!("{}", p.name);
                    let pin_name = format_ident!("{}", pin.pin);
                    let af = pin.af.unwrap_or(0);

                    // MCO is special
                    if pin.signal.starts_with("MCO_") {
                        // Supported in H7 only for now
                        if regs.version.starts_with("h5")
                            || regs.version.starts_with("h7")
                            || regs.version.starts_with("f4")
                        {
                            peri = format_ident!("{}", pin.signal.replace('_', ""));
                        } else {
                            continue;
                        }
                    }

                    if pin.signal == "MCO" {
                        // Supported in H7 only for now
                        if regs.version.starts_with("l4") {
                            peri = format_ident!("MCO");
                        } else {
                            continue;
                        }
                    }

                    g.extend(quote! {
                        pin_trait_impl!(#tr, #peri, #pin_name, #af);
                    })
                }

                // ADC is special
                if regs.kind == "adc" {
                    if p.rcc.is_none() {
                        continue;
                    }

                    let peri = format_ident!("{}", p.name);
                    let pin_name = format_ident!("{}", pin.pin);

                    // H7 has differential voltage measurements
                    let ch: Option<u8> = if pin.signal.starts_with("INP") {
                        Some(pin.signal.strip_prefix("INP").unwrap().parse().unwrap())
                    } else if pin.signal.starts_with("INN") {
                        // TODO handle in the future when embassy supports differential measurements
                        None
                    } else if pin.signal.starts_with("IN") {
                        Some(pin.signal.strip_prefix("IN").unwrap().parse().unwrap())
                    } else {
                        None
                    };
                    if let Some(ch) = ch {
                        g.extend(quote! {
                            impl_adc_pin!( #peri, #pin_name, #ch);
                        })
                    }
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
        (("lpuart", "RX"), quote!(crate::usart::RxDma)),
        (("lpuart", "TX"), quote!(crate::usart::TxDma)),
        (("spi", "RX"), quote!(crate::spi::RxDma)),
        (("spi", "TX"), quote!(crate::spi::TxDma)),
        (("i2c", "RX"), quote!(crate::i2c::RxDma)),
        (("i2c", "TX"), quote!(crate::i2c::TxDma)),
        (("dcmi", "DCMI"), quote!(crate::dcmi::FrameDma)),
        (("dcmi", "PSSI"), quote!(crate::dcmi::FrameDma)),
        // SDMMCv1 uses the same channel for both directions, so just implement for RX
        (("sdmmc", "RX"), quote!(crate::sdmmc::SdmmcDma)),
        (("quadspi", "QUADSPI"), quote!(crate::qspi::QuadDma)),
        (("dac", "CH1"), quote!(crate::dac::DmaCh1)),
        (("dac", "CH2"), quote!(crate::dac::DmaCh2)),
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
                        // Chip with DMA/BDMA, without DMAMUX
                        let channel = format_ident!("{}", channel);
                        quote!({channel: #channel})
                    } else if let Some(dmamux) = &ch.dmamux {
                        // Chip with DMAMUX
                        let dmamux = format_ident!("{}", dmamux);
                        quote!({dmamux: #dmamux})
                    } else if let Some(dma) = &ch.dma {
                        // Chip with GPDMA
                        let dma = format_ident!("{}", dma);
                        quote!({dma: #dma})
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

    let mut flash_regions_table: Vec<Vec<String>> = Vec::new();
    let mut interrupts_table: Vec<Vec<String>> = Vec::new();
    let mut peripherals_table: Vec<Vec<String>> = Vec::new();
    let mut pins_table: Vec<Vec<String>> = Vec::new();
    let mut dma_channels_table: Vec<Vec<String>> = Vec::new();

    for m in METADATA
        .memory
        .iter()
        .filter(|m| m.kind == MemoryRegionKind::Flash && m.settings.is_some())
    {
        let settings = m.settings.as_ref().unwrap();
        let row = vec![
            get_flash_region_type_name(m.name),
            settings.write_size.to_string(),
            settings.erase_size.to_string(),
        ];
        flash_regions_table.push(row);
    }

    let gpio_base = METADATA.peripherals.iter().find(|p| p.name == "GPIOA").unwrap().address as u32;
    let gpio_stride = 0x400;

    for p in METADATA.peripherals {
        if let Some(regs) = &p.registers {
            if regs.kind == "gpio" {
                let port_letter = p.name.chars().nth(4).unwrap();
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
                let row = vec![
                    p.name.to_string(),
                    regs.kind.to_string(),
                    regs.block.to_string(),
                    irq.signal.to_string(),
                    irq.interrupt.to_ascii_uppercase(),
                ];
                interrupts_table.push(row)
            }

            let row = vec![regs.kind.to_string(), p.name.to_string()];
            peripherals_table.push(row);
        }
    }

    let mut dma_channel_count: usize = 0;
    let mut bdma_channel_count: usize = 0;
    let mut gpdma_channel_count: usize = 0;

    for ch in METADATA.dma_channels {
        let mut row = Vec::new();
        let dma_peri = METADATA.peripherals.iter().find(|p| p.name == ch.dma).unwrap();
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
            "gpdma" => {
                num = gpdma_channel_count;
                gpdma_channel_count += 1;
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
            row.push(format!("{{dmamux: {}, dmamux_channel: {}}}", dmamux, dmamux_channel));
        } else {
            row.push("{}".to_string());
        }

        dma_channels_table.push(row);
    }

    g.extend(quote! {
        pub(crate) const DMA_CHANNEL_COUNT: usize = #dma_channel_count;
        pub(crate) const BDMA_CHANNEL_COUNT: usize = #bdma_channel_count;
        pub(crate) const GPDMA_CHANNEL_COUNT: usize = #gpdma_channel_count;
    });

    for irq in METADATA.interrupts {
        let name = irq.name.to_ascii_uppercase();
        interrupts_table.push(vec![name.clone()]);
        if name.contains("EXTI") {
            interrupts_table.push(vec!["EXTI".to_string(), name.clone()]);
        }
    }

    let mut m = String::new();

    make_table(&mut m, "foreach_flash_region", &flash_regions_table);
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
        println!("cargo:rustc-cfg={}_{}", &chip_name[..chip_name.len() - 2], core);
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
        writeln!(out, "        __{}_inner!(({}));", name, row.join(",")).unwrap();
    }

    write!(
        out,
        "    }};
}}"
    )
    .unwrap();
}

fn get_flash_region_name(name: &str) -> String {
    let name = name.replace("BANK_", "BANK").replace("REGION_", "REGION");
    if name.contains("REGION") {
        name
    } else {
        name + "_REGION"
    }
}

fn get_flash_region_type_name(name: &str) -> String {
    get_flash_region_name(name)
        .replace("BANK", "Bank")
        .replace("REGION", "Region")
        .replace('_', "")
}
