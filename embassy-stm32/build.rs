use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

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

    struct Peripheral {
        kind: String,
        name: String,
        version: String,
    }

    let mut peripheral_version_mapping = HashMap::<String, String>::new();
    stm32_metapac::peripheral_versions!(
        ($peri:ident, $version:ident) => {
            peripheral_version_mapping.insert(stringify!($peri).to_string(), stringify!($version).to_string());
            println!("cargo:rustc-cfg={}", stringify!($peri));
            println!("cargo:rustc-cfg={}_{}", stringify!($peri), stringify!($version));
        };
    );

    let mut peripherals: Vec<Peripheral> = Vec::new();
    stm32_metapac::peripherals!(
        ($kind:ident, $name:ident) => {
            peripherals.push(Peripheral{
                kind: stringify!($kind).to_string(),
                name: stringify!($name).to_string(),
                version: peripheral_version_mapping[&stringify!($kind).to_ascii_lowercase()].clone()
            });
        };
    );

    let mut singletons: Vec<String> = Vec::new();
    for p in peripherals {
        match p.kind.as_str() {
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
                if p.version == "h7" {
                    singletons.push("MCO1".to_string());
                    singletons.push("MCO2".to_string());
                }
                singletons.push(p.name.clone());
            }
            //"dbgmcu" => {}
            //"syscfg" => {}
            //"dma" => {}
            //"bdma" => {}
            //"dmamux" => {}

            // For other peripherals, one singleton per peri
            _ => singletons.push(p.name.clone()),
        }
    }

    // One singleton per EXTI line
    for pin_num in 0..16 {
        singletons.push(format!("EXTI{}", pin_num));
    }

    // One singleton per DMA channel
    stm32_metapac::dma_channels! {
        ($channel_peri:ident, $dma_peri:ident, $version:ident, $channel_num:expr, $ignore:tt) => {
            singletons.push(stringify!($channel_peri).to_string());
        };
    }

    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("generated.rs").to_string_lossy().to_string();
    fs::write(
        out_file,
        format!(
            "embassy_hal_common::peripherals!({});",
            singletons.join(",")
        ),
    )
    .unwrap();

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
