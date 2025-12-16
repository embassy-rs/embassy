use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use cfg_aliases::cfg_aliases;
use nxp_pac::metadata;
use nxp_pac::metadata::{METADATA, Peripheral};
#[allow(unused)]
use proc_macro2::TokenStream;
use proc_macro2::{Ident, Literal, Span};
use quote::format_ident;
#[allow(unused)]
use quote::quote;

#[path = "./build_common.rs"]
mod common;

fn main() {
    let mut cfgs = common::CfgSet::new();
    common::set_target_cfgs(&mut cfgs);

    let chip_name = match env::vars()
        .map(|(a, _)| a)
        .filter(|x| x.starts_with("CARGO_FEATURE_MIMXRT") || x.starts_with("CARGO_FEATURE_LPC"))
        .get_one()
    {
        Ok(x) => x,
        Err(GetOneError::None) => panic!("No mimxrt/lpc Cargo feature enabled"),
        Err(GetOneError::Multiple) => panic!("Multiple mimxrt/lpc Cargo features enabled"),
    }
    .strip_prefix("CARGO_FEATURE_")
    .unwrap()
    .to_ascii_lowercase();

    let singletons = singletons(&mut cfgs);

    cfg_aliases! {
        rt1xxx: { any(feature = "mimxrt1011", feature = "mimxrt1062") },
    }

    eprintln!("chip: {chip_name}");

    generate_code(&mut cfgs, &singletons);
}

/// A peripheral singleton returned by `embassy_nxp::init`.
struct Singleton {
    name: String,

    /// A cfg guard which indicates whether the `Peripherals` struct will give the user this singleton.
    cfg: Option<TokenStream>,
}

fn singletons(cfgs: &mut common::CfgSet) -> Vec<Singleton> {
    let mut singletons = Vec::new();

    for peripheral in METADATA.peripherals {
        // GPIO and DMA are generated in a 2nd pass.
        let skip_singleton = if peripheral.name.starts_with("GPIO") || peripheral.name.starts_with("DMA") {
            true
        } else {
            false
        };

        if !skip_singleton {
            singletons.push(Singleton {
                name: peripheral.name.into(),
                cfg: None,
            });
        }
    }

    cfgs.declare_all(&[
        "gpio1",
        "gpio1_hi",
        "gpio2",
        "gpio2_hi",
        "gpio3",
        "gpio3_hi",
        "gpio4",
        "gpio4_hi",
        "gpio5",
        "gpio5_hi",
        "gpio10",
        "gpio10_hi",
    ]);

    for peripheral in METADATA.peripherals.iter().filter(|p| p.name.starts_with("GPIO")) {
        let number = peripheral.name.strip_prefix("GPIO").unwrap();
        assert!(number.parse::<u8>().is_ok());
        cfgs.enable(format!("gpio{}", number));

        for signal in peripheral.signals.iter() {
            let pin_number = signal.name.parse::<u8>().unwrap();

            if pin_number > 15 {
                cfgs.enable(format!("gpio{}_hi", number));
            }

            // GPIO signals only defined a single signal, on a single pin.
            assert_eq!(signal.pins.len(), 1);

            singletons.push(Singleton {
                name: signal.pins[0].pin.into(),
                cfg: None,
            });
        }
    }

    for peripheral in METADATA.peripherals.iter().filter(|p| p.name.starts_with("DMA")) {
        let instance = peripheral.name.strip_prefix("DMA").unwrap();
        assert!(instance.parse::<u8>().is_ok());

        for signal in peripheral.signals.iter() {
            let channel_number = signal.name.parse::<u8>().unwrap();
            let name = format!("DMA{instance}_CH{channel_number}");

            // DMA has no pins.
            assert!(signal.pins.is_empty());

            singletons.push(Singleton { name, cfg: None });
        }
    }

    for peripheral in METADATA.peripherals.iter().filter(|p| p.name.starts_with("SCT")) {
        let instance = peripheral.name.strip_prefix("SCT").unwrap();
        assert!(instance.parse::<u8>().is_ok());

        for signal in peripheral.signals.iter() {
            if !signal.name.starts_with("OUT") {
                continue;
            }

            let channel_number = signal.name.strip_prefix("OUT").unwrap().parse::<u8>().unwrap();
            let name = format!("SCT{instance}_OUT{channel_number}");

            singletons.push(Singleton { name, cfg: None });
        }
    }

    singletons
}

#[cfg(feature = "_rt1xxx")]
fn generate_iomuxc() -> TokenStream {
    let iomuxc_pad_impls = metadata::METADATA
        .pins
        .iter()
        .filter(|p| p.iomuxc.as_ref().filter(|i| i.mux.is_some()).is_some())
        .map(|pin| {
            let Some(ref iomuxc) = pin.iomuxc else {
                panic!("Pin {} has no IOMUXC definitions", pin.name);
            };

            let name = Ident::new(pin.name, Span::call_site());
            let mux = iomuxc.mux.unwrap();
            let pad = iomuxc.pad;

            quote! {
                impl_iomuxc_pad!(#name, #pad, #mux);
            }
        });

    let base_match_arms = metadata::METADATA
        .peripherals
        .iter()
        .filter(|p| p.name.starts_with("GPIO"))
        .map(|peripheral| {
            peripheral.signals.iter().map(|signal| {
                // All GPIO signals have a single pin.
                let pin = &signal.pins[0];
                let instance = peripheral.name.strip_prefix("GPIO").unwrap();
                let bank_match = format_ident!("Gpio{}", instance);
                let pin_number = signal.name.parse::<u8>().unwrap();
                let pin_ident = Ident::new(pin.pin, Span::call_site());

                quote! {
                    (Bank::#bank_match, #pin_number) => <crate::peripherals::#pin_ident as crate::iomuxc::SealedPad>
                }
            })
        })
        .flatten()
        .collect::<Vec<_>>();

    let pad_match_arms = base_match_arms.iter().map(|arm| {
        quote! { #arm::PAD }
    });

    let mux_match_arms = base_match_arms.iter().map(|arm| {
        quote! { #arm::MUX }
    });

    quote! {
        #(#iomuxc_pad_impls)*

        pub(crate) fn iomuxc_pad(bank: crate::gpio::Bank, pin: u8) -> *mut () {
            use crate::gpio::Bank;

            match (bank, pin) {
                #(#pad_match_arms),*,
                _ => unreachable!()
            }
        }

        pub(crate) fn iomuxc_mux(bank: crate::gpio::Bank, pin: u8) -> Option<*mut ()> {
            use crate::gpio::Bank;

            match (bank, pin) {
                #(#mux_match_arms),*,
                _ => unreachable!()
            }
        }
    }
}

fn generate_code(cfgs: &mut common::CfgSet, singletons: &[Singleton]) {
    #[allow(unused)]
    use std::fmt::Write;

    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    #[allow(unused_mut)]
    let mut output = String::new();

    writeln!(&mut output, "{}", peripherals(singletons)).unwrap();

    #[cfg(feature = "_rt1xxx")]
    writeln!(&mut output, "{}", generate_iomuxc()).unwrap();

    writeln!(&mut output, "{}", interrupts()).unwrap();
    writeln!(&mut output, "{}", impl_peripherals(cfgs, singletons)).unwrap();

    let out_file = out_dir.join("_generated.rs").to_string_lossy().to_string();
    fs::write(&out_file, output).unwrap();
    rustfmt(&out_file);
}

fn interrupts() -> TokenStream {
    let interrupts = METADATA.interrupts.iter().map(|interrupt| format_ident!("{interrupt}"));

    quote! {
        embassy_hal_internal::interrupt_mod!(#(#interrupts),*);
    }
}

fn peripherals(singletons: &[Singleton]) -> TokenStream {
    let defs = singletons.iter().map(|s| {
        let ident = Ident::new(&s.name, Span::call_site());
        quote! { #ident }
    });

    let peripherals = singletons.iter().map(|s| {
        let ident = Ident::new(&s.name, Span::call_site());
        let cfg = s.cfg.clone().unwrap_or_else(|| quote! {});
        quote! {
            #cfg
            #ident
        }
    });

    quote! {
        embassy_hal_internal::peripherals_definition!(#(#defs),*);
        embassy_hal_internal::peripherals_struct!(#(#peripherals),*);
    }
}

fn impl_gpio_pin(impls: &mut Vec<TokenStream>, peripheral: &Peripheral) {
    let instance = peripheral.name.strip_prefix("GPIO").unwrap();
    let bank = format_ident!("Gpio{}", instance);
    // let pin =

    for signal in peripheral.signals.iter() {
        let pin_number = signal.name.parse::<u8>().unwrap();
        let pin = Ident::new(signal.pins[0].pin, Span::call_site());

        impls.push(quote! {
            impl_pin!(#pin, #bank, #pin_number);
        });
    }
}

fn impl_dma_channel(impls: &mut Vec<TokenStream>, peripheral: &Peripheral) {
    let instance = Ident::new(peripheral.name, Span::call_site());

    for signal in peripheral.signals.iter() {
        let channel_number = signal.name.parse::<u8>().unwrap();
        let channel_name = format_ident!("{instance}_CH{channel_number}");

        impls.push(quote! {
            impl_dma_channel!(#instance, #channel_name, #channel_number);
        });
    }
}

fn impl_usart(impls: &mut Vec<TokenStream>, peripheral: &Peripheral) {
    let instance = Ident::new(peripheral.name, Span::call_site());
    let flexcomm = Ident::new(
        peripheral.flexcomm.expect("LPC55 must specify FLEXCOMM instance"),
        Span::call_site(),
    );
    let number = Literal::u8_unsuffixed(peripheral.name.strip_prefix("USART").unwrap().parse::<u8>().unwrap());

    impls.push(quote! {
        impl_usart_instance!(#instance, #flexcomm, #number);
    });

    for signal in peripheral.signals {
        let r#macro = match signal.name {
            "TXD" => format_ident!("impl_usart_txd_pin"),
            "RXD" => format_ident!("impl_usart_rxd_pin"),
            _ => unreachable!(),
        };

        for pin in signal.pins {
            let alt = format_ident!("ALT{}", pin.alt);
            let pin = format_ident!("{}", pin.pin);

            impls.push(quote! {
                #r#macro!(#pin, #instance, #alt);
            });
        }
    }

    for dma_mux in peripheral.dma_muxing {
        assert_eq!(dma_mux.mux, "DMA0", "TODO: USART for more than LPC55");

        let r#macro = match dma_mux.signal {
            "TX" => format_ident!("impl_usart_tx_channel"),
            "RX" => format_ident!("impl_usart_rx_channel"),
            _ => unreachable!(),
        };

        let channel = format_ident!("DMA0_CH{}", dma_mux.request);

        impls.push(quote! {
            #r#macro!(#instance, #channel);
        });
    }
}

fn impl_sct(impls: &mut Vec<TokenStream>, peripheral: &Peripheral) {
    let instance = Ident::new(peripheral.name, Span::call_site());

    impls.push(quote! {
        impl_sct_instance!(#instance);
    });

    for signal in peripheral.signals.iter() {
        if signal.name.starts_with("OUT") {
            let channel_number = signal.name.strip_prefix("OUT").unwrap().parse::<u8>().unwrap();

            let channel_name = format_ident!("{instance}_OUT{channel_number}");

            impls.push(quote! {
                impl_sct_output_instance!(#instance, #channel_name, #channel_number);
            });

            if signal.name.starts_with("OUT") {
                for pin in signal.pins {
                    let pin_name = format_ident!("{}", pin.pin);
                    let alt = format_ident!("ALT{}", pin.alt);

                    impls.push(quote! {
                        impl_sct_output_pin!(#instance, #channel_name, #pin_name, #alt);
                    });
                }
            }
        }
    }
}

fn impl_peripherals(_cfgs: &mut common::CfgSet, _singletons: &[Singleton]) -> TokenStream {
    let mut impls = Vec::new();

    for peripheral in metadata::METADATA.peripherals.iter() {
        if peripheral.name.starts_with("GPIO") {
            impl_gpio_pin(&mut impls, peripheral);
        }

        if peripheral.name.starts_with("DMA") {
            impl_dma_channel(&mut impls, peripheral);
        }

        if peripheral.name.starts_with("USART") {
            impl_usart(&mut impls, peripheral);
        }

        if peripheral.name.starts_with("SCT") {
            impl_sct(&mut impls, peripheral);
        }
    }

    quote! {
        #(#impls)*
    }
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
