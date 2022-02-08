use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
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
