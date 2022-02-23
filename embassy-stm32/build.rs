use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
use std::env;
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
                    if r.version == "h7" {
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

    let mut dma_irqs: HashSet<&str> = HashSet::new();
    let mut bdma_irqs: HashSet<&str> = HashSet::new();

    for p in METADATA.peripherals {
        if let Some(r) = &p.registers {
            match r.kind {
                "dma" => {
                    for irq in p.interrupts {
                        dma_irqs.insert(irq.interrupt);
                    }
                }
                "bdma" => {
                    for irq in p.interrupts {
                        bdma_irqs.insert(irq.interrupt);
                    }
                }
                _ => {}
            }
        }
    }

    let tokens: Vec<_> = dma_irqs.iter().map(|s| format_ident!("{}", s)).collect();
    g.extend(quote! {
        #(
            #[crate::interrupt]
            unsafe fn #tokens () {
                crate::dma::dma::on_irq();
            }
        )*
    });

    let tokens: Vec<_> = bdma_irqs.iter().map(|s| format_ident!("{}", s)).collect();
    g.extend(quote! {
        #(
            #[crate::interrupt]
            unsafe fn #tokens () {
                crate::dma::bdma::on_irq();
            }
        )*
    });

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
    // Write generated.rs

    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("generated.rs").to_string_lossy().to_string();
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
        Some("any") => {
            if singletons.contains(&"TIM2".to_string()) {
                println!("cargo:rustc-cfg=time_driver_tim2");
            } else if singletons.contains(&"TIM3".to_string()) {
                println!("cargo:rustc-cfg=time_driver_tim3");
            } else if singletons.contains(&"TIM4".to_string()) {
                println!("cargo:rustc-cfg=time_driver_tim4");
            } else if singletons.contains(&"TIM5".to_string()) {
                println!("cargo:rustc-cfg=time_driver_tim5");
            } else {
                panic!("time-driver-any requested, but the chip doesn't have TIM2, TIM3, TIM4 or TIM5.")
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
