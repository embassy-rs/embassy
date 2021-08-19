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
    }

    let mut peripherals: Vec<Peripheral> = Vec::new();
    stm32_metapac::peripherals!(
        ($kind:ident, $name:ident) => {
            peripherals.push(Peripheral{
                kind: stringify!($kind).to_string(),
                name: stringify!($name).to_string(),
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
            //"rcc" => {}
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

    stm32_metapac::peripheral_versions!(
        ($peri:ident, $version:ident) => {
            println!("cargo:rustc-cfg={}", stringify!($peri));
            println!("cargo:rustc-cfg={}_{}", stringify!($peri), stringify!($version));
        };
    );

    let mut chip_and_core = chip_name.split('_');
    let chip = chip_and_core.next().expect("Unexpected stm32xx feature");

    if let Some(core) = chip_and_core.next() {
        println!("cargo:rustc-cfg={}_{}", &chip[..(chip.len() - 2)], core);
    } else {
        println!("cargo:rustc-cfg={}", &chip[..(chip.len() - 2)]);
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=gen.py");
}
