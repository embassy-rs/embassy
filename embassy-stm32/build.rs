use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::PathBuf;
use std::{env, fs};

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use stm32_metapac::metadata::ir::{BlockItemInner, Enum, FieldSet};
use stm32_metapac::metadata::{MemoryRegionKind, PeripheralRccRegister, StopMode, METADATA};

fn main() {
    let target = env::var("TARGET").unwrap();

    if target.starts_with("thumbv6m-") {
        println!("cargo:rustc-cfg=cortex_m");
        println!("cargo:rustc-cfg=armv6m");
    } else if target.starts_with("thumbv7m-") {
        println!("cargo:rustc-cfg=cortex_m");
        println!("cargo:rustc-cfg=armv7m");
    } else if target.starts_with("thumbv7em-") {
        println!("cargo:rustc-cfg=cortex_m");
        println!("cargo:rustc-cfg=armv7m");
        println!("cargo:rustc-cfg=armv7em"); // (not currently used)
    } else if target.starts_with("thumbv8m.base") {
        println!("cargo:rustc-cfg=cortex_m");
        println!("cargo:rustc-cfg=armv8m");
        println!("cargo:rustc-cfg=armv8m_base");
    } else if target.starts_with("thumbv8m.main") {
        println!("cargo:rustc-cfg=cortex_m");
        println!("cargo:rustc-cfg=armv8m");
        println!("cargo:rustc-cfg=armv8m_main");
    }

    if target.ends_with("-eabihf") {
        println!("cargo:rustc-cfg=has_fpu");
    }

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
            println!("cargo:rustc-cfg=peri_{}", p.name.to_ascii_lowercase());
            match r.kind {
                // Generate singletons per pin, not per port
                "gpio" => {
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
                    for pin in p.pins {
                        if pin.signal.starts_with("MCO") {
                            let name = pin.signal.replace('_', "").to_string();
                            if !singletons.contains(&name) {
                                println!("cargo:rustc-cfg={}", name.to_ascii_lowercase());
                                singletons.push(name);
                            }
                        }
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

    let mut pin_set = std::collections::HashSet::new();
    for p in METADATA.peripherals {
        for pin in p.pins {
            pin_set.insert(pin.pin);
        }
    }

    struct SplitFeature {
        feature_name: String,
        pin_name_with_c: String,
        #[cfg(feature = "_split-pins-enabled")]
        pin_name_without_c: String,
    }

    // Extra analog switch pins available on most H7 chips
    let split_features: Vec<SplitFeature> = vec![
        #[cfg(feature = "split-pa0")]
        SplitFeature {
            feature_name: "split-pa0".to_string(),
            pin_name_with_c: "PA0_C".to_string(),
            pin_name_without_c: "PA0".to_string(),
        },
        #[cfg(feature = "split-pa1")]
        SplitFeature {
            feature_name: "split-pa1".to_string(),
            pin_name_with_c: "PA1_C".to_string(),
            pin_name_without_c: "PA1".to_string(),
        },
        #[cfg(feature = "split-pc2")]
        SplitFeature {
            feature_name: "split-pc2".to_string(),
            pin_name_with_c: "PC2_C".to_string(),
            pin_name_without_c: "PC2".to_string(),
        },
        #[cfg(feature = "split-pc3")]
        SplitFeature {
            feature_name: "split-pc3".to_string(),
            pin_name_with_c: "PC3_C".to_string(),
            pin_name_without_c: "PC3".to_string(),
        },
    ];

    for split_feature in &split_features {
        if pin_set.contains(split_feature.pin_name_with_c.as_str()) {
            singletons.push(split_feature.pin_name_with_c.clone());
        } else {
            panic!(
                "'{}' feature invalid for this chip! No pin '{}' found.\n
                Found pins: {:#?}",
                split_feature.feature_name, split_feature.pin_name_with_c, pin_set
            )
        }
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
        Some("tim9") => "TIM9",
        Some("tim11") => "TIM11",
        Some("tim12") => "TIM12",
        Some("tim15") => "TIM15",
        Some("tim21") => "TIM21",
        Some("tim22") => "TIM22",
        Some("any") => {
            if singletons.contains(&"TIM2".to_string()) {
                "TIM2"
            } else if singletons.contains(&"TIM3".to_string()) {
                "TIM3"
            } else if singletons.contains(&"TIM4".to_string()) {
                "TIM4"
            } else if singletons.contains(&"TIM5".to_string()) {
                "TIM5"
            } else if singletons.contains(&"TIM9".to_string()) {
                "TIM9"
            } else if singletons.contains(&"TIM11".to_string()) {
                "TIM11"
            } else if singletons.contains(&"TIM12".to_string()) {
                "TIM12"
            } else if singletons.contains(&"TIM15".to_string()) {
                "TIM15"
            } else if singletons.contains(&"TIM21".to_string()) {
                "TIM21"
            } else if singletons.contains(&"TIM22".to_string()) {
                "TIM22"
            } else {
                panic!("time-driver-any requested, but the chip doesn't have TIM2, TIM3, TIM4, TIM5, TIM9, TIM11, TIM12 or TIM15.")
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

    let mut dma_irqs: BTreeMap<&str, Vec<(&str, &str, &str)>> = BTreeMap::new();

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

    let dma_irqs: TokenStream = dma_irqs
        .iter()
        .map(|(irq, channels)| {
            let irq = format_ident!("{}", irq);

            let xdma = format_ident!("{}", channels[0].0);
            let channels = channels.iter().map(|(_, dma, ch)| format_ident!("{}_{}", dma, ch));

            quote! {
                #[cfg(feature = "rt")]
                #[crate::interrupt]
                unsafe fn #irq () {
                    #(
                        <crate::peripherals::#channels as crate::dma::#xdma::sealed::Channel>::on_irq();
                    )*
                }
            }
        })
        .collect();

    g.extend(dma_irqs);

    // ========
    // Extract the rcc registers
    let rcc_registers = METADATA
        .peripherals
        .iter()
        .filter_map(|p| p.registers.as_ref())
        .find(|r| r.kind == "rcc")
        .unwrap();

    // ========
    // Generate rcc fieldset and enum maps
    let rcc_enum_map: HashMap<&str, HashMap<&str, &Enum>> = {
        let rcc_blocks = rcc_registers.ir.blocks.iter().find(|b| b.name == "Rcc").unwrap().items;
        let rcc_fieldsets: HashMap<&str, &FieldSet> = rcc_registers.ir.fieldsets.iter().map(|f| (f.name, f)).collect();
        let rcc_enums: HashMap<&str, &Enum> = rcc_registers.ir.enums.iter().map(|e| (e.name, e)).collect();

        rcc_blocks
            .iter()
            .filter_map(|b| match &b.inner {
                BlockItemInner::Register(register) => register.fieldset.map(|f| (b.name, f)),
                _ => None,
            })
            .filter_map(|(b, f)| {
                rcc_fieldsets.get(f).map(|f| {
                    (
                        b,
                        f.fields
                            .iter()
                            .filter_map(|f| {
                                let enumm = f.enumm?;
                                let enumm = rcc_enums.get(enumm)?;

                                Some((f.name, *enumm))
                            })
                            .collect(),
                    )
                })
            })
            .collect()
    };

    // ========
    // Generate RccPeripheral impls

    // count how many times each xxENR field is used, to enable refcounting if used more than once.
    let mut rcc_field_count: HashMap<_, usize> = HashMap::new();
    for p in METADATA.peripherals {
        if let Some(rcc) = &p.rcc {
            let en = rcc.enable.as_ref().unwrap();
            *rcc_field_count.entry((en.register, en.field)).or_insert(0) += 1;
        }
    }

    let force_refcount = HashSet::from(["usart"]);
    let mut refcount_statics = BTreeSet::new();

    let mut clock_names = BTreeSet::new();

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
                        crate::pac::RCC.#rst_reg().modify(|w| w.#set_rst_field(true));
                        crate::pac::RCC.#rst_reg().modify(|w| w.#set_rst_field(false));
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
            let en_reg = format_ident!("{}", en.register);
            let set_en_field = format_ident!("set_{}", en.field);

            let refcount =
                force_refcount.contains(ptype) || *rcc_field_count.get(&(en.register, en.field)).unwrap() > 1;
            let (before_enable, before_disable) = if refcount {
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

            let mux_for = |mux: Option<&'static PeripheralRccRegister>| {
                let mux = mux?;
                let fieldset = rcc_enum_map.get(mux.register)?;
                let enumm = fieldset.get(mux.field)?;

                Some((mux, *enumm))
            };

            let clock_frequency = match mux_for(rcc.mux.as_ref()) {
                Some((mux, rcc_enumm)) => {
                    let fieldset_name = format_ident!("{}", mux.register);
                    let field_name = format_ident!("{}", mux.field);
                    let enum_name = format_ident!("{}", rcc_enumm.name);

                    let match_arms: TokenStream = rcc_enumm
                        .variants
                        .iter()
                        .filter(|v| v.name != "DISABLE")
                        .map(|v| {
                            let variant_name = format_ident!("{}", v.name);
                            let clock_name = format_ident!("{}", v.name.to_ascii_lowercase());
                            clock_names.insert(v.name.to_ascii_lowercase());
                            quote! {
                                #enum_name::#variant_name => unsafe { crate::rcc::get_freqs().#clock_name.unwrap() },
                            }
                        })
                        .collect();

                    quote! {
                        use crate::pac::rcc::vals::#enum_name;

                        #[allow(unreachable_patterns)]
                        match crate::pac::RCC.#fieldset_name().read().#field_name() {
                            #match_arms
                            _ => unreachable!(),
                        }
                    }
                }
                None => {
                    let clock_name = format_ident!("{}", rcc.clock);
                    clock_names.insert(rcc.clock.to_string());
                    quote! {
                        unsafe { crate::rcc::get_freqs().#clock_name.unwrap() }
                    }
                }
            };

            /*
                A refcount leak can result if the same field is shared by peripherals with different stop modes
                This condition should be checked in stm32-data
            */
            let stop_refcount = match rcc.stop_mode {
                StopMode::Standby => None,
                StopMode::Stop2 => Some(quote! { REFCOUNT_STOP2 }),
                StopMode::Stop1 => Some(quote! { REFCOUNT_STOP1 }),
            };

            let (incr_stop_refcount, decr_stop_refcount) = match stop_refcount {
                Some(stop_refcount) => (
                    quote! {
                        #[cfg(feature = "low-power")]
                        unsafe { crate::rcc::#stop_refcount += 1 };
                    },
                    quote! {
                        #[cfg(feature = "low-power")]
                        unsafe { crate::rcc::#stop_refcount -= 1 };
                    },
                ),
                None => (TokenStream::new(), TokenStream::new()),
            };

            g.extend(quote! {
                impl crate::rcc::sealed::RccPeripheral for peripherals::#pname {
                    fn frequency() -> crate::time::Hertz {
                        #clock_frequency
                    }
                    fn enable_and_reset_with_cs(_cs: critical_section::CriticalSection) {
                        #before_enable
                        #incr_stop_refcount
                        crate::pac::RCC.#en_reg().modify(|w| w.#set_en_field(true));
                        #after_enable
                        #rst
                    }
                    fn disable_with_cs(_cs: critical_section::CriticalSection) {
                        #before_disable
                        crate::pac::RCC.#en_reg().modify(|w| w.#set_en_field(false));
                        #decr_stop_refcount
                    }
                }

                impl crate::rcc::RccPeripheral for peripherals::#pname {}
            });
        }
    }

    // Generate RCC
    clock_names.insert("sys".to_string());
    clock_names.insert("rtc".to_string());
    let clock_idents: Vec<_> = clock_names.iter().map(|n| format_ident!("{}", n)).collect();
    g.extend(quote! {
        #[derive(Clone, Copy, Debug)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        pub struct Clocks {
            #(
                pub #clock_idents: Option<crate::time::Hertz>,
            )*
        }
    });

    let clocks_macro = quote!(
        macro_rules! set_clocks {
            ($($(#[$m:meta])* $k:ident: $v:expr,)*) => {
                {
                    #[allow(unused)]
                    struct Temp {
                        $($(#[$m])* $k: Option<crate::time::Hertz>,)*
                    }
                    let all = Temp {
                        $($(#[$m])* $k: $v,)*
                    };
                    crate::rcc::set_freqs(crate::rcc::Clocks {
                        #( #clock_idents: all.#clock_idents, )*
                    });
                }
            };
        }
    );

    let refcount_mod: TokenStream = refcount_statics
        .iter()
        .map(|refcount_static| {
            quote! {
                pub(crate) static mut #refcount_static: u8 = 0;
            }
        })
        .collect();

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
        (("sai", "SCK_A"), quote!(crate::sai::SckPin<A>)),
        (("sai", "SCK_B"), quote!(crate::sai::SckPin<B>)),
        (("sai", "FS_A"), quote!(crate::sai::FsPin<A>)),
        (("sai", "FS_B"), quote!(crate::sai::FsPin<B>)),
        (("sai", "SD_A"), quote!(crate::sai::SdPin<A>)),
        (("sai", "SD_B"), quote!(crate::sai::SdPin<B>)),
        (("sai", "MCLK_A"), quote!(crate::sai::MclkPin<A>)),
        (("sai", "MCLK_B"), quote!(crate::sai::MclkPin<B>)),
        (("sai", "WS"), quote!(crate::sai::WsPin)),
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
        (("eth", "RX_CLK"), quote!(crate::eth::RXClkPin)),
        (("eth", "TX_CLK"), quote!(crate::eth::TXClkPin)),
        (("eth", "MDIO"), quote!(crate::eth::MDIOPin)),
        (("eth", "MDC"), quote!(crate::eth::MDCPin)),
        (("eth", "CRS_DV"), quote!(crate::eth::CRSPin)),
        (("eth", "RX_DV"), quote!(crate::eth::RXDVPin)),
        (("eth", "RXD0"), quote!(crate::eth::RXD0Pin)),
        (("eth", "RXD1"), quote!(crate::eth::RXD1Pin)),
        (("eth", "RXD2"), quote!(crate::eth::RXD2Pin)),
        (("eth", "RXD3"), quote!(crate::eth::RXD3Pin)),
        (("eth", "TXD0"), quote!(crate::eth::TXD0Pin)),
        (("eth", "TXD1"), quote!(crate::eth::TXD1Pin)),
        (("eth", "TXD2"), quote!(crate::eth::TXD2Pin)),
        (("eth", "TXD3"), quote!(crate::eth::TXD3Pin)),
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
        (("fmc", "CLK"), quote!(crate::fmc::ClkPin)),
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
        (("quadspi", "BK1_IO0"), quote!(crate::qspi::BK1D0Pin)),
        (("quadspi", "BK1_IO1"), quote!(crate::qspi::BK1D1Pin)),
        (("quadspi", "BK1_IO2"), quote!(crate::qspi::BK1D2Pin)),
        (("quadspi", "BK1_IO3"), quote!(crate::qspi::BK1D3Pin)),
        (("quadspi", "BK1_NCS"), quote!(crate::qspi::BK1NSSPin)),
        (("quadspi", "BK2_IO0"), quote!(crate::qspi::BK2D0Pin)),
        (("quadspi", "BK2_IO1"), quote!(crate::qspi::BK2D1Pin)),
        (("quadspi", "BK2_IO2"), quote!(crate::qspi::BK2D2Pin)),
        (("quadspi", "BK2_IO3"), quote!(crate::qspi::BK2D3Pin)),
        (("quadspi", "BK2_NCS"), quote!(crate::qspi::BK2NSSPin)),
        (("quadspi", "CLK"), quote!(crate::qspi::SckPin)),
    ].into();

    for p in METADATA.peripherals {
        if let Some(regs) = &p.registers {
            for pin in p.pins {
                let key = (regs.kind, pin.signal);
                if let Some(tr) = signals.get(&key) {
                    let mut peri = format_ident!("{}", p.name);

                    let pin_name = {
                        // If we encounter a _C pin but the split_feature for this pin is not enabled, skip it
                        if pin.pin.ends_with("_C") && !split_features.iter().any(|x| x.pin_name_with_c == pin.pin) {
                            continue;
                        }

                        format_ident!("{}", pin.pin)
                    };

                    let af = pin.af.unwrap_or(0);

                    // MCO is special
                    if pin.signal.starts_with("MCO") {
                        peri = format_ident!("{}", pin.signal.replace('_', ""));
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
                    let pin_name = {
                        // If we encounter a _C pin but the split_feature for this pin is not enabled, skip it
                        if pin.pin.ends_with("_C") && !split_features.iter().any(|x| x.pin_name_with_c == pin.pin) {
                            continue;
                        }
                        format_ident!("{}", pin.pin)
                    };

                    // H7 has differential voltage measurements
                    let ch: Option<u8> = if pin.signal.starts_with("INP") {
                        Some(pin.signal.strip_prefix("INP").unwrap().parse().unwrap())
                    } else if pin.signal.starts_with("INN") {
                        // TODO handle in the future when embassy supports differential measurements
                        None
                    } else if pin.signal.starts_with("IN") && pin.signal.ends_with('b') {
                        // we number STM32L1 ADC bank 1 as 0..=31, bank 2 as 32..=63
                        let signal = pin.signal.strip_prefix("IN").unwrap().strip_suffix('b').unwrap();
                        Some(32u8 + signal.parse::<u8>().unwrap())
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

                if regs.kind == "opamp" {
                    if pin.signal.starts_with("VP") {
                        // Impl NonInvertingPin for the VP* signals (VP0, VP1, VP2, etc)
                        let peri = format_ident!("{}", p.name);
                        let pin_name = format_ident!("{}", pin.pin);
                        let ch: u8 = pin.signal.strip_prefix("VP").unwrap().parse().unwrap();

                        g.extend(quote! {
                            impl_opamp_vp_pin!( #peri, #pin_name, #ch);
                        })
                    } else if pin.signal == "VOUT" {
                        // Impl OutputPin for the VOUT pin
                        let peri = format_ident!("{}", p.name);
                        let pin_name = format_ident!("{}", pin.pin);
                        g.extend(quote! {
                            impl_opamp_vout_pin!( #peri, #pin_name );
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
        (("sai", "A"), quote!(crate::sai::Dma<A>)),
        (("sai", "B"), quote!(crate::sai::Dma<B>)),
        (("spi", "RX"), quote!(crate::spi::RxDma)),
        (("spi", "TX"), quote!(crate::spi::TxDma)),
        (("i2c", "RX"), quote!(crate::i2c::RxDma)),
        (("i2c", "TX"), quote!(crate::i2c::TxDma)),
        (("dcmi", "DCMI"), quote!(crate::dcmi::FrameDma)),
        (("dcmi", "PSSI"), quote!(crate::dcmi::FrameDma)),
        // SDMMCv1 uses the same channel for both directions, so just implement for RX
        (("sdmmc", "RX"), quote!(crate::sdmmc::SdmmcDma)),
        (("quadspi", "QUADSPI"), quote!(crate::qspi::QuadDma)),
        (("dac", "CH1"), quote!(crate::dac::DacDma1)),
        (("dac", "CH2"), quote!(crate::dac::DacDma2)),
        (("timer", "UP"), quote!(crate::timer::UpDma)),
        (("timer", "CH1"), quote!(crate::timer::Ch1Dma)),
        (("timer", "CH2"), quote!(crate::timer::Ch2Dma)),
        (("timer", "CH3"), quote!(crate::timer::Ch3Dma)),
        (("timer", "CH4"), quote!(crate::timer::Ch4Dma)),
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
    // Generate Div/Mul impls for RCC prescalers/dividers/multipliers.
    for e in rcc_registers.ir.enums {
        fn is_rcc_name(e: &str) -> bool {
            match e {
                "Pllp" | "Pllq" | "Pllr" | "Pllm" | "Plln" => true,
                "Timpre" | "Pllrclkpre" => false,
                e if e.ends_with("pre") || e.ends_with("pres") || e.ends_with("div") || e.ends_with("mul") => true,
                _ => false,
            }
        }

        #[derive(Copy, Clone, Debug)]
        struct Frac {
            num: u32,
            denom: u32,
        }

        impl Frac {
            fn simplify(self) -> Self {
                let d = gcd(self.num, self.denom);
                Self {
                    num: self.num / d,
                    denom: self.denom / d,
                }
            }
        }

        fn gcd(a: u32, b: u32) -> u32 {
            if b == 0 {
                return a;
            }
            gcd(b, a % b)
        }

        fn parse_num(n: &str) -> Result<Frac, ()> {
            for prefix in ["DIV", "MUL"] {
                if let Some(n) = n.strip_prefix(prefix) {
                    let exponent = n.find('_').map(|e| n.len() - 1 - e).unwrap_or(0) as u32;
                    let mantissa = n.replace('_', "").parse().map_err(|_| ())?;
                    let f = Frac {
                        num: mantissa,
                        denom: 10u32.pow(exponent),
                    };
                    return Ok(f.simplify());
                }
            }
            Err(())
        }

        if is_rcc_name(e.name) {
            let enum_name = format_ident!("{}", e.name);
            let mut muls = Vec::new();
            let mut divs = Vec::new();
            for v in e.variants {
                let Ok(val) = parse_num(v.name) else {
                    panic!("could not parse mul/div. enum={} variant={}", e.name, v.name)
                };
                let variant_name = format_ident!("{}", v.name);
                let variant = quote!(crate::pac::rcc::vals::#enum_name::#variant_name);
                let num = val.num;
                let denom = val.denom;
                muls.push(quote!(#variant => self * #num / #denom,));
                divs.push(quote!(#variant => self * #denom / #num,));
            }

            g.extend(quote! {
                impl core::ops::Div<crate::pac::rcc::vals::#enum_name> for crate::time::Hertz {
                    type Output = crate::time::Hertz;
                    fn div(self, rhs: crate::pac::rcc::vals::#enum_name) -> Self::Output {
                        match rhs {
                            #(#divs)*
                            #[allow(unreachable_patterns)]
                            _ => unreachable!(),
                        }
                    }
                }
                impl core::ops::Mul<crate::pac::rcc::vals::#enum_name> for crate::time::Hertz {
                    type Output = crate::time::Hertz;
                    fn mul(self, rhs: crate::pac::rcc::vals::#enum_name) -> Self::Output {
                        match rhs {
                            #(#muls)*
                            #[allow(unreachable_patterns)]
                            _ => unreachable!(),
                        }
                    }
                }
            });
        }
    }

    // ========
    // Write peripheral_interrupts module.
    let mut mt = TokenStream::new();
    for p in METADATA.peripherals {
        let mut pt = TokenStream::new();

        for irq in p.interrupts {
            let iname = format_ident!("{}", irq.interrupt);
            let sname = format_ident!("{}", irq.signal);
            pt.extend(quote!(pub type #sname = crate::interrupt::typelevel::#iname;));
        }

        let pname = format_ident!("{}", p.name);
        mt.extend(quote!(pub mod #pname { #pt }));
    }
    g.extend(quote!(#[allow(non_camel_case_types)] pub mod peripheral_interrupts { #mt }));

    // ========
    // Write foreach_foo! macrotables

    let mut flash_regions_table: Vec<Vec<String>> = Vec::new();
    let mut interrupts_table: Vec<Vec<String>> = Vec::new();
    let mut peripherals_table: Vec<Vec<String>> = Vec::new();
    let mut pins_table: Vec<Vec<String>> = Vec::new();
    let mut dma_channels_table: Vec<Vec<String>> = Vec::new();
    let mut adc_common_table: Vec<Vec<String>> = Vec::new();

    /*
        If ADC3_COMMON exists, ADC3 and higher are assigned to it
        All other ADCs are assigned to ADC_COMMON

        ADC3 and higher are assigned to the adc34 clock in the table
        The adc3_common cfg directive is added if ADC3_COMMON exists
    */
    let has_adc3 = METADATA.peripherals.iter().any(|p| p.name == "ADC3_COMMON");
    let set_adc345 = HashSet::from(["ADC3", "ADC4", "ADC5"]);

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
                        pin_name.clone(),
                        p.name.to_string(),
                        port_num.to_string(),
                        pin_num.to_string(),
                        format!("EXTI{}", pin_num),
                    ]);

                    // If we have the split pins, we need to do a little extra work:
                    // Add the "_C" variant to the table. The solution is not optimal, though.
                    // Adding them only when the corresponding GPIOx also appears.
                    // This should avoid unintended side-effects as much as possible.
                    #[cfg(feature = "_split-pins-enabled")]
                    for split_feature in &split_features {
                        if split_feature.pin_name_without_c == pin_name {
                            pins_table.push(vec![
                                split_feature.pin_name_with_c.to_string(),
                                p.name.to_string(),
                                port_num.to_string(),
                                pin_num.to_string(),
                                format!("EXTI{}", pin_num),
                            ]);
                        }
                    }
                }
            }

            if regs.kind == "adc" {
                let (adc_common, adc_clock) = if set_adc345.contains(p.name) && has_adc3 {
                    ("ADC3_COMMON", "adc34")
                } else {
                    ("ADC_COMMON", "adc")
                };

                let row = vec![p.name.to_string(), adc_common.to_string(), adc_clock.to_string()];
                adc_common_table.push(row);
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

    let mut m = clocks_macro.to_string();

    // DO NOT ADD more macros like these.
    // These turned to be a bad idea!
    // Instead, make build.rs generate the final code.
    make_table(&mut m, "foreach_flash_region", &flash_regions_table);
    make_table(&mut m, "foreach_interrupt", &interrupts_table);
    make_table(&mut m, "foreach_peripheral", &peripherals_table);
    make_table(&mut m, "foreach_pin", &pins_table);
    make_table(&mut m, "foreach_dma_channel", &dma_channels_table);
    make_table(&mut m, "foreach_adc", &adc_common_table);

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
    }

    // =======
    // ADC3_COMMON is present
    #[allow(clippy::print_literal)]
    if has_adc3 {
        println!("cargo:rustc-cfg={}", "adc3_common");
    }

    // =======
    // Features for targeting groups of chips

    if &chip_name[..8] == "stm32wba" {
        println!("cargo:rustc-cfg={}", &chip_name[..8]); // stm32wba
        println!("cargo:rustc-cfg={}", &chip_name[..10]); // stm32wba52
        println!("cargo:rustc-cfg=package_{}", &chip_name[10..11]);
        println!("cargo:rustc-cfg=flashsize_{}", &chip_name[11..12]);
    } else {
        println!("cargo:rustc-cfg={}", &chip_name[..7]); // stm32f4
        println!("cargo:rustc-cfg={}", &chip_name[..9]); // stm32f429
        println!("cargo:rustc-cfg={}x", &chip_name[..8]); // stm32f42x
        println!("cargo:rustc-cfg={}x{}", &chip_name[..7], &chip_name[8..9]); // stm32f4x9
        println!("cargo:rustc-cfg=package_{}", &chip_name[9..10]);
        println!("cargo:rustc-cfg=flashsize_{}", &chip_name[10..11]);
    }

    // Mark the L4+ chips as they have many differences to regular L4.
    if &chip_name[..7] == "stm32l4" {
        if "pqrs".contains(&chip_name[7..8]) {
            println!("cargo:rustc-cfg=stm32l4_plus");
        } else {
            println!("cargo:rustc-cfg=stm32l4_nonplus");
        }
    }

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
