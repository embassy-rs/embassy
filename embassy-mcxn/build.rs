use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use build_common::CfgSet;
use nxp_pac::metadata::METADATA;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use regex::Regex;

mod build_common;

fn main() {
    let mut cfgs = CfgSet::new();
    build_common::set_target_cfgs(&mut cfgs);

    fn driver_to_cfg_name(name: &str) -> String {
        match name.split_once("::") {
            Some((path, _block)) => path,
            None => name,
        }
        .replace("/", "_")
        .to_lowercase()
    }

    // Declare all drivers in nxp-pac (used or unused)
    for peripheral in nxp_pac::metadata::META_PERIPHERALS {
        cfgs.declare(driver_to_cfg_name(peripheral));
    }

    // Enable all drivers for this chip
    for peripheral in METADATA.peripherals {
        if peripheral.driver_name.is_empty() {
            continue;
        }

        cfgs.enable(driver_to_cfg_name(peripheral.driver_name));
    }

    let generated = [
        generate_peripherals_call(),
        generate_interrupt_mod_call(),
        generate_gpio_pin_impls(),
    ];

    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("_generated.rs");
    fs::write(&out_file, generated.into_iter().collect::<TokenStream>().to_string()).unwrap();
    rustfmt(&out_file);
}

fn generate_peripherals_call() -> TokenStream {
    struct Singleton {
        name: String,
        feature: Option<String>,
    }

    impl Singleton {
        pub fn just_name(name: impl ToString) -> Self {
            Self {
                name: name.to_string(),
                feature: None,
            }
        }
    }

    let mut singletons: Vec<Singleton> = Vec::new();
    // Add pins
    singletons.extend(METADATA.pins.iter().map(|pin| Singleton {
        name: pin.name.to_owned(),
        feature: pin.feature.map(|str| str.to_owned()),
    }));

    // Add peripherals
    singletons.extend(
        METADATA
            .peripherals
            .iter()
            .map(|peripheral| Singleton::just_name(peripheral.name)),
    );

    // Output the singletons
    let singleton_tokens: Vec<_> = singletons
        .iter()
        .map(|s| {
            let feature = s
                .feature
                .as_ref()
                .map_or(TokenStream::default(), |feature| quote! { #[cfg(feature = #feature)] });
            let name = format_ident!("{}", s.name);
            quote! { #feature #name }
        })
        .collect();

    quote! {
        embassy_hal_internal::peripherals!(#(#singleton_tokens),*);
    }
}

fn generate_interrupt_mod_call() -> TokenStream {
    let mut irqs = Vec::new();
    for (name, _) in METADATA.interrupts {
        irqs.push(format_ident!("{}", name));
    }

    quote! {
        embassy_hal_internal::interrupt_mod!(
            #(
                #irqs,
            )*
        );
    }
}

fn pin_feature_gate(pin_name: &str) -> TokenStream {
    let pin = METADATA
        .pins
        .iter()
        .find(|pin| pin.name == pin_name)
        .unwrap_or_else(|| panic!("Failed to find pin {pin_name}"));
    pin.feature
        .as_ref()
        .map_or(TokenStream::default(), |feature| quote! { #[cfg(feature = #feature)] })
}

fn generate_gpio_pin_impls() -> TokenStream {
    let mut generated = TokenStream::new();

    let gpio_regex = Regex::new(r"(?i:^GPIO)(\d+)").unwrap();
    for (gpio_num, gpio_peripheral) in METADATA
        .peripherals
        .iter()
        .filter_map(|p| get_regex_num(p.name, &gpio_regex).map(|gpio_num| (gpio_num, p)))
    {
        let gpio_num = proc_macro2::Literal::u32_unsuffixed(gpio_num);

        for s in gpio_peripheral.signals {
            assert_eq!(s.pins.len(), 1, "Each gpio signal should only have 1 pin: {s:?}");
            let pin = format_ident!("{}", s.pins[0].pin);
            let pin_num = proc_macro2::Literal::u32_unsuffixed(s.name.parse().unwrap());
            let feature_gate = pin_feature_gate(s.pins[0].pin);

            generated.extend(quote! {
                #feature_gate
                crate::impl_gpio_pin!(#pin, #gpio_num, #pin_num);
            });
        }
    }

    generated
}

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

fn get_regex_num(string: &str, regex: &Regex) -> Option<u32> {
    regex
        .captures(string)
        .map(|cap| cap.extract::<1>().1[0].parse::<u32>().unwrap())
}
