use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use nxp_pac::metadata::METADATA;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use regex::Regex;

use build_common::CfgSet;

mod build_common;

fn main() {
    let mut cfgs = CfgSet::new();
    build_common::set_target_cfgs(&mut cfgs);

    // TODO: Declare all possible driver cfgs. Needs extra info in pac metadata

    // Enable all drivers for this chip
    for peripheral in METADATA.peripherals {
        if peripheral.driver_name.is_empty() {
            continue;
        }

        let cfg_name = match peripheral.driver_name.split_once("::") {
            Some((path, _block)) => path,
            None => peripheral.driver_name,
        }.replace("/", "_");

        cfgs.enable(&cfg_name);
        // Temporary until todo above is removed
        cfgs.declare(&cfg_name);
    }

    let get_regex_num = |string, regex: &Regex| {
        regex
            .captures(string)
            .map(|cap| cap.extract::<1>().1[0].parse::<u32>().unwrap())
    };

    let mut singletons: Vec<String> = Vec::new();
    // Add pins
    singletons.extend(METADATA.pins.iter().map(|pin| pin.name.to_owned()));
    // Add peripherals
    singletons.extend(METADATA.peripherals.iter().map(|peripheral| peripheral.name.to_owned()));

    // Add DMA channels
    let dma_regex = Regex::new(r"(?i:^DMA)(\d+)").unwrap();
    let dma_channels_regex = Regex::new(r"(?i:DMA)(\d+)").unwrap();
    for (dma_num, dma_peripheral) in METADATA
        .peripherals
        .iter()
        .filter_map(|p| get_regex_num(p.name, &dma_regex).map(|dma_num| (dma_num, p)))
    {
        let channels_num = get_regex_num(dma_peripheral.driver_name, &dma_channels_regex).unwrap();

        for channel in 0..channels_num {
            singletons.push(format!("DMA{dma_num}_CH{channel}"));
        }
    }

    // Add CTIMER channels
    let ctimer_regex = Regex::new(r"(?i:^CTIMER)(\d+)").unwrap();
    for ctimer_num in METADATA
        .peripherals
        .iter()
        .filter_map(|p| get_regex_num(p.name, &ctimer_regex))
    {
        for num in 0..4 {
            singletons.push(format!("CTIMER{ctimer_num}_CH{num}"));
        }
    }

    // TODO: Remove singletons for pins with dual-use based on feature flags

    let mut generated = TokenStream::new();

    // Output the singletons
    let singleton_tokens: Vec<_> = singletons.iter().map(|s| format_ident!("{}", s)).collect();

    generated.extend(quote! {
        embassy_hal_internal::peripherals!(#(#singleton_tokens),*);
    });

    // Output the interrupts

    let mut irqs = Vec::new();
    for (name, _) in METADATA.interrupts {
        irqs.push(format_ident!("{}", name));
    }

    generated.extend(quote! {
        embassy_hal_internal::interrupt_mod!(
            #(
                #irqs,
            )*
        );
    });

    // Impl gpio pins
    let gpio_regex = Regex::new(r"(?i:^GPIO)(\d+)").unwrap();
    for (gpio_num, gpio_peripheral) in METADATA
        .peripherals
        .iter()
        .filter_map(|p| get_regex_num(p.name, &gpio_regex).map(|gpio_num| (gpio_num, p)))
    {
        let peripheral = format_ident!("{}", gpio_peripheral.name);
        let gpio_num = proc_macro2::Literal::u32_unsuffixed(gpio_num);

        for s in gpio_peripheral.signals {
            assert_eq!(s.pins.len(), 1, "Each gpio signal should only have 1 pin: {s:?}");
            let pin = format_ident!("{}", s.pins[0].pin);
            let pin_num = proc_macro2::Literal::u32_unsuffixed(s.name.parse().unwrap());

            generated.extend(quote! {
                impl_pin!(#pin, #gpio_num, #pin_num, #peripheral);
            });
        }
    }

    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("_generated.rs");
    fs::write(&out_file, generated.to_string()).unwrap();
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
