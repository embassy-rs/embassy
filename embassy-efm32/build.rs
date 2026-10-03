//! Generates the chip-specific code (`_generated.rs`) from the chip data at the end of this file:
//! the peripheral singletons, the interrupts, and the GPIO and USART pin trait impls.
//!
//! The dies of a family share one set of peripherals and pin functions, but each package only
//! bonds out a subset of the GPIO pins, so only those are generated.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use proc_macro2::{Ident, Literal, Span};
use quote::quote;

#[path = "./build_common.rs"]
mod common;

const PORTS: [char; 6] = ['A', 'B', 'C', 'D', 'E', 'F'];

fn main() {
    let mut cfgs = common::CfgSet::new();
    common::set_target_cfgs(&mut cfgs);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build_common.rs");

    // Catch a chip feature (`efm32gg` and the die number) added to `Cargo.toml` but not to the chip
    // data.
    for (var, _) in env::vars() {
        if let Some(number) = var.strip_prefix("CARGO_FEATURE_EFM32GG")
            && !number.is_empty()
            && number.bytes().all(|b| b.is_ascii_digit())
        {
            let chip = format!("efm32gg{number}");
            if !efm32gg::CHIPS.iter().any(|(chips, _)| chips.contains(&chip.as_str())) {
                panic!("Chip feature `{chip}` has no entry in `build.rs`.");
            }
        }
    }

    let chips: Vec<(&str, &str)> = efm32gg::CHIPS
        .iter()
        .flat_map(|(chips, missing)| chips.iter().map(move |chip| (*chip, *missing)))
        .filter(|(chip, _)| feature(chip))
        .collect();
    let missing_pins = match chips.as_slice() {
        [(_, missing)] => missing,
        [] => panic!("No chip feature enabled. Enable exactly one, e.g. `efm32gg990`."),
        _ => panic!(
            "Several chip features enabled ({}). Enable exactly one.",
            chips.iter().map(|(chip, _)| *chip).collect::<Vec<_>>().join(", ")
        ),
    };

    generate_code(&missing_pins.split_whitespace().collect::<Vec<_>>());
}

/// Whether the Cargo feature `name` is enabled.
fn feature(name: &str) -> bool {
    env::var_os(format!("CARGO_FEATURE_{}", name.to_uppercase().replace('-', "_"))).is_some()
}

fn enabled(gate: &Gate) -> bool {
    match gate {
        Gate::Always => true,
        Gate::Feature(name) => feature(name),
        Gate::NotFeature(name) => !feature(name),
    }
}

fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

fn generate_code(missing_pins: &[&str]) {
    // GPIO pins: those the package bonds out, minus the reserved ones not handed out as GPIOs.
    let pins: Vec<(String, char, u8)> = PORTS
        .iter()
        .flat_map(|&port| (0..16).map(move |n| (format!("P{port}{n}"), port, n)))
        .filter(|(name, _, _)| !missing_pins.contains(&name.as_str()))
        .filter(|(name, _, _)| {
            efm32gg::RESERVED_PINS
                .iter()
                .all(|(pin, released_by)| pin != name || feature(released_by))
        })
        .collect();
    let has_pin = |name: &str| pins.iter().any(|(pin, _, _)| pin == name);

    let peripherals: Vec<&str> = efm32gg::PERIPHERALS
        .iter()
        .filter(|(_, gate)| enabled(gate))
        .map(|(name, _)| *name)
        .collect();
    let has_peripheral = |name: &str| peripherals.contains(&name);

    let singletons = peripherals
        .iter()
        .map(|name| ident(name))
        .chain(pins.iter().map(|(name, _, _)| ident(name)));

    let interrupts = efm32gg::INTERRUPTS
        .iter()
        .filter(|(_, gate)| enabled(gate))
        .map(|(name, _)| ident(name));

    let pin_impls = pins.iter().map(|(name, port, n)| {
        let (name, port, n) = (ident(name), ident(&port.to_string()), Literal::u8_unsuffixed(*n));
        quote!(impl_pin!(#name, #port, #n);)
    });

    let usart_impls =
        efm32gg::USARTS
            .iter()
            .filter(|(_, _, _, gate)| enabled(gate))
            .map(|(name, rx_irq, tx_irq, _)| {
                let (name, rx_irq, tx_irq) = (ident(name), ident(rx_irq), ident(tx_irq));
                quote!(impl_usart!(#name, #rx_irq, #tx_irq);)
            });

    let usart_pin_impls = efm32gg::USART_PINS
        .iter()
        .filter(|(peripheral, _, pin, _)| has_peripheral(peripheral) && has_pin(pin))
        .map(|(peripheral, signal, pin, location)| {
            let (peripheral, signal, pin) = (ident(peripheral), ident(signal), ident(pin));
            let location = Literal::u8_unsuffixed(*location);
            quote!(impl_usart_pin!(#peripheral, #signal, #pin, #location);)
        });

    let g = quote! {
        embassy_hal_internal::peripherals!(#(#singletons),*);

        // In a module of its own for the `allow`: `interrupt_mod!` generates an `unsafe trait`
        // without a `# Safety` section.
        #[allow(clippy::missing_safety_doc)]
        mod interrupts {
            embassy_hal_internal::interrupt_mod!(#(#interrupts),*);
        }
        pub use interrupts::interrupt;

        #(#pin_impls)*
        #(#usart_impls)*
        #(#usart_pin_impls)*
    };

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("_generated.rs");
    fs::write(&out_file, g.to_string()).unwrap();
    rustfmt(&out_file);
}

/// rustfmt a given path.
/// Failures are logged to stderr and ignored.
fn rustfmt(path: impl AsRef<Path>) {
    let path = path.as_ref();
    match Command::new("rustfmt").args([path]).output() {
        Err(e) => {
            eprintln!("failed to exec rustfmt {:?}: {:?}", path, e);
        }
        Ok(out) => {
            if !out.status.success() {
                eprintln!("rustfmt {:?} failed:", path);
                eprintln!("=== STDOUT:");
                std::io::stderr().write_all(&out.stdout).unwrap();
                eprintln!("=== STDERR:");
                std::io::stderr().write_all(&out.stderr).unwrap();
            }
        }
    }
}

/// When an item is present, in terms of the crate's Cargo features.
enum Gate {
    Always,
    Feature(&'static str),
    NotFeature(&'static str),
}

/// Chip data for the EFM32GG (Giant Gecko) family.
///
/// `efm32gg-pac` doesn't carry this kind of metadata (unlike e.g. `stm32-metapac`), so it's kept
/// here. Once the PAC has it (a `metadata` feature, as `nxp-pac` has), `build.rs` should read it
/// from there instead, like the other HALs do. Sources:
/// - Pins per package: the "GPIO Pinout Overview" tables of the EFM32GG data sheet (Rev. 2.1),
///   cross-checked against its pinout tables.
/// - USART/UART pin locations: Silicon Labs' `efm32gg_af_pins.h`.
/// - Peripherals and interrupts: the `efm32gg-pac` chip modules.
mod efm32gg {
    use super::Gate;

    /// Peripheral singletons, other than the GPIO pins.
    ///
    /// System blocks the HAL itself owns (CMU, EMU, RMU, MSC, GPIO) and the ETM (debug trace) aren't
    /// singletons.
    pub const PERIPHERALS: &[(&str, Gate)] = &[
        ("ACMP0", Gate::Always),
        ("ACMP1", Gate::Always),
        ("ADC0", Gate::Always),
        ("AES", Gate::Always),
        ("BURTC", Gate::Always),
        ("DAC0", Gate::Always),
        ("DMA", Gate::Always),
        ("EBI", Gate::Feature("_uart")),
        ("I2C0", Gate::Always),
        ("I2C1", Gate::Always),
        ("LCD", Gate::Feature("_lcd")),
        ("LESENSE", Gate::Always),
        ("LETIMER0", Gate::Always),
        ("LEUART0", Gate::Always),
        ("LEUART1", Gate::Always),
        ("PCNT0", Gate::Always),
        ("PCNT1", Gate::Always),
        ("PCNT2", Gate::Always),
        ("PRS", Gate::Always),
        // Used by the time driver.
        ("RTC", Gate::NotFeature("time-driver-rtc")),
        ("TIMER0", Gate::Always),
        ("TIMER1", Gate::Always),
        ("TIMER2", Gate::Always),
        ("TIMER3", Gate::Always),
        ("UART0", Gate::Feature("_uart")),
        ("UART1", Gate::Feature("_uart")),
        ("USART0", Gate::Always),
        ("USART1", Gate::Always),
        ("USART2", Gate::Always),
        ("USB", Gate::Feature("_usb")),
        ("VCMP", Gate::Always),
        ("WDOG", Gate::Always),
    ];

    /// Pins with a dedicated function, only handed out as GPIOs with the given feature.
    pub const RESERVED_PINS: &[(&str, &str)] = &[
        // LFXO crystal (LFXTAL_P, LFXTAL_N).
        ("PB7", "lfxo-as-gpio"),
        ("PB8", "lfxo-as-gpio"),
        // SWD (SWCLK, SWDIO).
        ("PF0", "swd-as-gpio"),
        ("PF1", "swd-as-gpio"),
    ];

    /// The chips of the family, as their Cargo features, grouped by package pinout: `(chips, pins
    /// missing on their package)`. All other pins of ports A to F exist.
    pub const CHIPS: &[(&[&str], &str)] = &[
        (
            &["efm32gg230"],
            "PA7 PA11 PA12 PA13 PA14 PB0 PB1 PB2 PB3 PB4 PB5 PB6 PB9 PB10 PB15 PD9 PD10 PD11 PD12 PD13 \
             PD14 PD15 PE0 PE1 PE2 PE3 PE4 PE5 PE6 PE7 PF6 PF7 PF8 PF9 PF10 PF11 PF12 PF13 PF14 PF15",
        ),
        (
            &["efm32gg232"],
            "PA6 PA7 PA11 PA12 PA13 PA14 PA15 PB0 PB1 PB2 PB3 PB4 PB5 PB6 PB9 PB10 PB12 PB15 PD9 PD10 \
             PD11 PD12 PD13 PD14 PD15 PE0 PE1 PE2 PE3 PE4 PE5 PE6 PE7 PF6 PF7 PF8 PF9 PF10 PF11 PF12 \
             PF13 PF14 PF15",
        ),
        (
            &["efm32gg280", "efm32gg880"],
            "PB15 PD13 PD14 PD15 PF10 PF11 PF12 PF13 PF14 PF15",
        ),
        (&["efm32gg290", "efm32gg890"], "PF10 PF11 PF12 PF13 PF14 PF15"),
        (
            &["efm32gg295", "efm32gg395", "efm32gg895", "efm32gg900", "efm32gg995"],
            "PF13 PF14 PF15",
        ),
        (
            &["efm32gg330"],
            "PA7 PA11 PA12 PA13 PA14 PB0 PB1 PB2 PB3 PB4 PB5 PB6 PB9 PB10 PB15 PC12 PC13 PC14 PC15 PD9 \
             PD10 PD11 PD12 PD13 PD14 PD15 PE0 PE1 PE2 PE3 PE4 PE5 PE6 PE7 PF3 PF4 PF6 PF7 PF8 PF9 \
             PF13 PF14 PF15",
        ),
        (
            &["efm32gg332"],
            "PA6 PA7 PA11 PA12 PA13 PA14 PA15 PB0 PB1 PB2 PB3 PB4 PB5 PB6 PB9 PB10 PB12 PB15 PC12 PC13 \
             PC14 PC15 PD9 PD10 PD11 PD12 PD13 PD14 PD15 PE0 PE1 PE2 PE3 PE4 PE5 PE6 PE7 PF3 PF4 PF6 \
             PF7 PF8 PF9 PF13 PF14 PF15",
        ),
        (
            &["efm32gg380", "efm32gg980"],
            "PB15 PC12 PC13 PC14 PC15 PD13 PD14 PD15 PF3 PF4 PF13 PF14 PF15",
        ),
        (
            &["efm32gg390", "efm32gg990"],
            "PC12 PC13 PC14 PC15 PF3 PF4 PF13 PF14 PF15",
        ),
        (
            &["efm32gg840"],
            "PA7 PA8 PA9 PA10 PA11 PB0 PB1 PB2 PB9 PB10 PB15 PC0 PC1 PC2 PC3 PC8 PC9 PC10 PC11 PD9 PD10 \
             PD11 PD12 PD13 PD14 PD15 PE0 PE1 PE2 PE3 PF6 PF7 PF8 PF9 PF10 PF11 PF12 PF13 PF14 PF15",
        ),
        (
            &["efm32gg842"],
            "PA6 PA7 PA8 PA9 PA10 PA11 PA15 PB0 PB1 PB2 PB9 PB10 PB12 PB15 PC0 PC1 PC2 PC3 PC8 PC9 PC10 \
             PC11 PD9 PD10 PD11 PD12 PD13 PD14 PD15 PE0 PE1 PE2 PE3 PF6 PF7 PF8 PF9 PF10 PF11 PF12 PF13 \
             PF14 PF15",
        ),
        (
            &["efm32gg940"],
            "PA7 PA8 PA9 PA10 PA11 PB0 PB1 PB2 PB9 PB10 PB15 PC0 PC1 PC2 PC3 PC8 PC9 PC10 PC11 PC12 PC13 \
             PC14 PC15 PD9 PD10 PD11 PD12 PD13 PD14 PD15 PE0 PE1 PE2 PE3 PF3 PF4 PF6 PF7 PF8 PF9 PF13 \
             PF14 PF15",
        ),
        (
            &["efm32gg942"],
            "PA6 PA7 PA8 PA9 PA10 PA11 PA15 PB0 PB1 PB2 PB9 PB10 PB12 PB15 PC0 PC1 PC2 PC3 PC8 PC9 PC10 \
             PC11 PC12 PC13 PC14 PC15 PD9 PD10 PD11 PD12 PD13 PD14 PD15 PE0 PE1 PE2 PE3 PF3 PF4 PF6 PF7 \
             PF8 PF9 PF13 PF14 PF15",
        ),
    ];

    /// Interrupts.
    pub const INTERRUPTS: &[(&str, Gate)] = &[
        ("DMA", Gate::Always),
        ("GPIO_EVEN", Gate::Always),
        ("TIMER0", Gate::Always),
        ("USART0_RX", Gate::Always),
        ("USART0_TX", Gate::Always),
        ("ACMP0", Gate::Always),
        ("ADC0", Gate::Always),
        ("DAC0", Gate::Always),
        ("I2C0", Gate::Always),
        ("I2C1", Gate::Always),
        ("GPIO_ODD", Gate::Always),
        ("TIMER1", Gate::Always),
        ("TIMER2", Gate::Always),
        ("TIMER3", Gate::Always),
        ("USART1_RX", Gate::Always),
        ("USART1_TX", Gate::Always),
        ("LESENSE", Gate::Always),
        ("USART2_RX", Gate::Always),
        ("USART2_TX", Gate::Always),
        ("LEUART0", Gate::Always),
        ("LEUART1", Gate::Always),
        ("LETIMER0", Gate::Always),
        ("PCNT0", Gate::Always),
        ("PCNT1", Gate::Always),
        ("PCNT2", Gate::Always),
        ("RTC", Gate::Always),
        ("BURTC", Gate::Always),
        ("CMU", Gate::Always),
        ("VCMP", Gate::Always),
        ("MSC", Gate::Always),
        ("AES", Gate::Always),
        ("EMU", Gate::Always),
        ("USB", Gate::Feature("_usb")),
        ("UART0_RX", Gate::Feature("_uart")),
        ("UART0_TX", Gate::Feature("_uart")),
        ("UART1_RX", Gate::Feature("_uart")),
        ("UART1_TX", Gate::Feature("_uart")),
        ("EBI", Gate::Feature("_uart")),
        ("LCD", Gate::Feature("_lcd")),
    ];

    /// USART/UART instances: `(peripheral, RX interrupt, TX interrupt, gate)`.
    pub const USARTS: &[(&str, &str, &str, Gate)] = &[
        ("USART0", "USART0_RX", "USART0_TX", Gate::Always),
        ("USART1", "USART1_RX", "USART1_TX", Gate::Always),
        ("USART2", "USART2_RX", "USART2_TX", Gate::Always),
        ("UART0", "UART0_RX", "UART0_TX", Gate::Feature("_uart")),
        ("UART1", "UART1_RX", "UART1_TX", Gate::Feature("_uart")),
    ];

    /// USART/UART TX/RX pins: `(peripheral, pin trait, pin, ROUTE location)`. Only generated for pins
    /// the package has, and peripherals the chip has.
    pub const USART_PINS: &[(&str, &str, &str, u8)] = &[
        ("USART0", "TxPin", "PE10", 0),
        ("USART0", "RxPin", "PE11", 0),
        ("USART0", "TxPin", "PE7", 1),
        ("USART0", "RxPin", "PE6", 1),
        ("USART0", "TxPin", "PC11", 2),
        ("USART0", "RxPin", "PC10", 2),
        ("USART0", "TxPin", "PE13", 3),
        ("USART0", "RxPin", "PE12", 3),
        ("USART0", "TxPin", "PB7", 4),
        ("USART0", "RxPin", "PB8", 4),
        ("USART0", "TxPin", "PC0", 5),
        ("USART0", "RxPin", "PC1", 5),
        ("USART1", "TxPin", "PC0", 0),
        ("USART1", "RxPin", "PC1", 0),
        ("USART1", "TxPin", "PD0", 1),
        ("USART1", "RxPin", "PD1", 1),
        ("USART1", "TxPin", "PD7", 2),
        ("USART1", "RxPin", "PD6", 2),
        ("USART2", "TxPin", "PC2", 0),
        ("USART2", "RxPin", "PC3", 0),
        ("USART2", "TxPin", "PB3", 1),
        ("USART2", "RxPin", "PB4", 1),
        ("UART0", "TxPin", "PF6", 0),
        ("UART0", "RxPin", "PF7", 0),
        ("UART0", "TxPin", "PE0", 1),
        ("UART0", "RxPin", "PE1", 1),
        ("UART0", "TxPin", "PA3", 2),
        ("UART0", "RxPin", "PA4", 2),
        ("UART0", "TxPin", "PC14", 3),
        ("UART0", "RxPin", "PC15", 3),
        ("UART1", "TxPin", "PC12", 0),
        ("UART1", "RxPin", "PC13", 0),
        ("UART1", "TxPin", "PF10", 1),
        ("UART1", "RxPin", "PF11", 1),
        ("UART1", "TxPin", "PB9", 2),
        ("UART1", "RxPin", "PB10", 2),
        ("UART1", "TxPin", "PE2", 3),
        ("UART1", "RxPin", "PE3", 3),
    ];
}
