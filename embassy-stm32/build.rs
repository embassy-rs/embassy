use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let chip_name = env::vars_os()
        .map(|(a, _)| a.to_string_lossy().to_string())
        .find(|x| x.starts_with("CARGO_FEATURE_STM32"))
        .expect("No stm32xx Cargo feature enabled")
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

    println!("cargo:rerun-if-changed=build.rs");
}
