use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use stm32_metapac::metadata::ir::BitOffset;
use stm32_metapac::metadata::{
    MemoryRegion, MemoryRegionKind, PeripheralRccKernelClock, PeripheralRccRegister, PeripheralRegisters, StopMode,
    ALL_CHIPS, ALL_PERIPHERAL_VERSIONS, METADATA,
};

#[path = "./build_common.rs"]
mod common;

fn main() {
    let mut cfgs = common::CfgSet::new();
    common::set_target_cfgs(&mut cfgs);

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

    eprintln!("chip: {chip_name}");

    for p in METADATA.peripherals {
        if let Some(r) = &p.registers {
            cfgs.enable(r.kind);
            cfgs.enable(format!("{}_{}", r.kind, r.version));
        }
    }

    for &(kind, versions) in ALL_PERIPHERAL_VERSIONS.iter() {
        cfgs.declare(kind);
        for &version in versions.iter() {
            cfgs.declare(format!("{}_{}", kind, version));
        }
    }

    // ========
    // Generate singletons

    let mut singletons: Vec<String> = Vec::new();

    // Generate one singleton per pin
    for p in METADATA.pins {
        singletons.push(p.name.to_string());
    }

    // generate one singleton per peripheral (with many exceptions...)
    for p in METADATA.peripherals {
        if let Some(r) = &p.registers {
            if r.kind == "adccommon"
                || r.kind == "sai"
                || r.kind == "ucpd"
                || r.kind == "otg"
                || r.kind == "octospi"
                || r.kind == "xspi"
            {
                // TODO: should we emit this for all peripherals? if so, we will need a list of all
                // possible peripherals across all chips, so that we can declare the configs
                // (replacing the hard-coded list of `peri_*` cfgs below)
                cfgs.enable(format!("peri_{}", p.name.to_ascii_lowercase()));
            }

            match r.kind {
                // handled above
                "gpio" => {}

                // No singleton for these, the HAL handles them specially.
                "exti" => {}

                // We *shouldn't* have singletons for these, but the HAL currently requires
                // singletons, for using with RccPeripheral to enable/disable clocks to them.
                "rcc" => {
                    for pin in p.pins {
                        if pin.signal.starts_with("MCO") {
                            let name = pin.signal.replace('_', "").to_string();
                            if !singletons.contains(&name) {
                                cfgs.enable(name.to_ascii_lowercase());
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

    cfgs.declare_all(&[
        "peri_adc1_common",
        "peri_adc3_common",
        "peri_adc12_common",
        "peri_adc34_common",
        "peri_sai1",
        "peri_sai2",
        "peri_sai3",
        "peri_sai4",
        "peri_ucpd1",
        "peri_ucpd2",
        "peri_usb_otg_fs",
        "peri_usb_otg_hs",
        "peri_octospi2",
        "peri_xspi2",
    ]);
    cfgs.declare_all(&["mco", "mco1", "mco2"]);

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
        Err(GetOneError::Multiple) => panic!("Multiple time-driver-xxx Cargo features enabled"),
    };

    let time_driver_singleton = match time_driver.as_ref().map(|x| x.as_ref()) {
        None => "",
        Some("tim1") => "TIM1",
        Some("tim2") => "TIM2",
        Some("tim3") => "TIM3",
        Some("tim4") => "TIM4",
        Some("tim5") => "TIM5",
        Some("tim8") => "TIM8",
        Some("tim9") => "TIM9",
        Some("tim12") => "TIM12",
        Some("tim15") => "TIM15",
        Some("tim20") => "TIM20",
        Some("tim21") => "TIM21",
        Some("tim22") => "TIM22",
        Some("tim23") => "TIM23",
        Some("tim24") => "TIM24",
        Some("any") => {
            // Order of TIM candidators:
            // 1. 2CH -> 2CH_CMP -> GP16 -> GP32 -> ADV
            // 2. In same catagory: larger TIM number first
            [
                "TIM22", "TIM21", "TIM12", "TIM9",  // 2CH
                "TIM15", // 2CH_CMP
                "TIM19", "TIM4", "TIM3", // GP16
                "TIM24", "TIM23", "TIM5", "TIM2", // GP32
                "TIM20", "TIM8", "TIM1", //ADV
            ]
            .iter()
            .find(|tim| singletons.contains(&tim.to_string())).expect("time-driver-any requested, but the chip doesn't have TIM1, TIM2, TIM3, TIM4, TIM5, TIM8, TIM9, TIM12, TIM15, TIM20, TIM21, TIM22, TIM23 or TIM24.")
        }
        _ => panic!("unknown time_driver {:?}", time_driver),
    };

    if !time_driver_singleton.is_empty() {
        cfgs.enable(format!("time_driver_{}", time_driver_singleton.to_lowercase()));
    }
    for tim in [
        "tim1", "tim2", "tim3", "tim4", "tim5", "tim8", "tim9", "tim12", "tim15", "tim20", "tim21", "tim22", "tim23",
        "tim24",
    ] {
        cfgs.declare(format!("time_driver_{}", tim));
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
            pub struct #region_type<'d, MODE = crate::flash::Async>(pub &'static crate::flash::FlashRegion, pub(crate) embassy_hal_internal::Peri<'d, crate::peripherals::FLASH>, pub(crate) core::marker::PhantomData<MODE>);
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
            pub(crate) fn new(p: embassy_hal_internal::Peri<'d, crate::peripherals::FLASH>) -> Self {
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
    // Extract the rcc registers

    let rcc_registers = METADATA
        .peripherals
        .iter()
        .filter_map(|p| p.registers.as_ref())
        .find(|r| r.kind == "rcc")
        .unwrap();
    let rcc_block = rcc_registers.ir.blocks.iter().find(|b| b.name == "Rcc").unwrap();

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

    struct ClockGen<'a> {
        rcc_registers: &'a PeripheralRegisters,
        chained_muxes: HashMap<&'a str, &'a PeripheralRccRegister>,

        clock_names: BTreeSet<String>,
        muxes: BTreeSet<(Ident, Ident, Ident)>,
    }

    let mut clock_gen = ClockGen {
        rcc_registers,
        chained_muxes: HashMap::new(),

        clock_names: BTreeSet::new(),
        muxes: BTreeSet::new(),
    };
    if chip_name.starts_with("stm32h5") {
        clock_gen.chained_muxes.insert(
            "PER",
            &PeripheralRccRegister {
                register: "CCIPR5",
                field: "PERSEL",
            },
        );
    }

    if chip_name.starts_with("stm32h7r") || chip_name.starts_with("stm32h7s") {
        clock_gen.chained_muxes.insert(
            "PER",
            &PeripheralRccRegister {
                register: "AHBPERCKSELR",
                field: "PERSEL",
            },
        );
    } else if chip_name.starts_with("stm32h7") {
        clock_gen.chained_muxes.insert(
            "PER",
            &PeripheralRccRegister {
                register: "D1CCIPR",
                field: "PERSEL",
            },
        );
    }
    if chip_name.starts_with("stm32u5") {
        clock_gen.chained_muxes.insert(
            "ICLK",
            &PeripheralRccRegister {
                register: "CCIPR1",
                field: "ICLKSEL",
            },
        );
    }
    if chip_name.starts_with("stm32wb") && !chip_name.starts_with("stm32wba") {
        clock_gen.chained_muxes.insert(
            "CLK48",
            &PeripheralRccRegister {
                register: "CCIPR",
                field: "CLK48SEL",
            },
        );
    }
    if chip_name.starts_with("stm32f7") {
        clock_gen.chained_muxes.insert(
            "CLK48",
            &PeripheralRccRegister {
                register: "DCKCFGR2",
                field: "CLK48SEL",
            },
        );
    }
    if chip_name.starts_with("stm32f4") && !chip_name.starts_with("stm32f410") {
        clock_gen.chained_muxes.insert(
            "CLK48",
            &PeripheralRccRegister {
                register: "DCKCFGR",
                field: "CLK48SEL",
            },
        );
    }

    impl<'a> ClockGen<'a> {
        fn parse_mul_div(name: &str) -> (&str, Frac) {
            if name == "hse_div_rtcpre" {
                return (name, Frac { num: 1, denom: 1 });
            }

            if let Some(i) = name.find("_div_") {
                let n = &name[..i];
                let val: u32 = name[i + 5..].parse().unwrap();
                (n, Frac { num: 1, denom: val })
            } else if let Some(i) = name.find("_mul_") {
                let n = &name[..i];
                let val: u32 = name[i + 5..].parse().unwrap();
                (n, Frac { num: val, denom: 1 })
            } else {
                (name, Frac { num: 1, denom: 1 })
            }
        }

        fn gen_clock(&mut self, peripheral: &str, name: &str) -> TokenStream {
            let name = name.to_ascii_lowercase();
            let (name, frac) = Self::parse_mul_div(&name);
            let clock_name = format_ident!("{}", name);
            self.clock_names.insert(name.to_string());

            let mut muldiv = quote!();
            if frac.num != 1 {
                let val = frac.num;
                muldiv.extend(quote!(* #val));
            }
            if frac.denom != 1 {
                let val = frac.denom;
                muldiv.extend(quote!(/ #val));
            }
            quote!(unsafe {
                unwrap!(
                    crate::rcc::get_freqs().#clock_name.to_hertz(),
                    "peripheral '{}' is configured to use the '{}' clock, which is not running. \
                    Either enable it in 'config.rcc' or change 'config.rcc.mux' to use another clock",
                    #peripheral,
                    #name
                )
                #muldiv
            })
        }

        fn gen_mux(&mut self, peripheral: &str, mux: &PeripheralRccRegister) -> TokenStream {
            let ir = &self.rcc_registers.ir;
            let fieldset_name = mux.register.to_ascii_lowercase();
            let fieldset = ir
                .fieldsets
                .iter()
                .find(|i| i.name.eq_ignore_ascii_case(&fieldset_name))
                .unwrap();
            let field_name = mux.field.to_ascii_lowercase();
            let field = fieldset.fields.iter().find(|i| i.name == field_name).unwrap();
            let enum_name = field.enumm.unwrap();
            let enumm = ir.enums.iter().find(|i| i.name == enum_name).unwrap();

            let fieldset_name = format_ident!("{}", fieldset_name);
            let field_name = format_ident!("{}", field_name);
            let enum_name = format_ident!("{}", enum_name);

            self.muxes
                .insert((fieldset_name.clone(), field_name.clone(), enum_name.clone()));

            let mut match_arms = TokenStream::new();

            for v in enumm.variants.iter().filter(|v| v.name != "DISABLE") {
                let variant_name = format_ident!("{}", v.name);
                let expr = if let Some(mux) = self.chained_muxes.get(&v.name) {
                    self.gen_mux(peripheral, mux)
                } else {
                    self.gen_clock(peripheral, v.name)
                };
                match_arms.extend(quote! {
                    crate::pac::rcc::vals::#enum_name::#variant_name => #expr,
                });
            }

            quote! {
                match crate::pac::RCC.#fieldset_name().read().#field_name() {
                    #match_arms
                    #[allow(unreachable_patterns)]
                    _ => panic!(
                        "attempted to use peripheral '{}' but its clock mux is not set to a valid \
                         clock. Change 'config.rcc.mux' to another clock.",
                        #peripheral
                    )
                }
            }
        }
    }

    let mut refcount_idxs = HashMap::new();

    for p in METADATA.peripherals {
        if !singletons.contains(&p.name.to_string()) {
            continue;
        }

        if let Some(rcc) = &p.rcc {
            let rst_reg = rcc.reset.as_ref();
            let en_reg = rcc.enable.as_ref().unwrap();
            let pname = format_ident!("{}", p.name);

            let get_offset_and_bit = |reg: &PeripheralRccRegister| -> TokenStream {
                let reg_offset = rcc_block
                    .items
                    .iter()
                    .find(|i| i.name.eq_ignore_ascii_case(reg.register))
                    .unwrap()
                    .byte_offset;
                let reg_offset: u8 = (reg_offset / 4).try_into().unwrap();

                let bit_offset = &rcc_registers
                    .ir
                    .fieldsets
                    .iter()
                    .find(|i| i.name.eq_ignore_ascii_case(reg.register))
                    .unwrap()
                    .fields
                    .iter()
                    .find(|i| i.name.eq_ignore_ascii_case(reg.field))
                    .unwrap()
                    .bit_offset;
                let BitOffset::Regular(bit_offset) = bit_offset else {
                    panic!("cursed bit offset")
                };
                let bit_offset: u8 = bit_offset.offset.try_into().unwrap();

                quote! { (#reg_offset, #bit_offset) }
            };

            let reset_offset_and_bit = match rst_reg {
                Some(rst_reg) => {
                    let reset_offset_and_bit = get_offset_and_bit(rst_reg);
                    quote! { Some(#reset_offset_and_bit) }
                }
                None => quote! { None },
            };
            let enable_offset_and_bit = get_offset_and_bit(en_reg);

            let needs_refcount = *rcc_field_count.get(&(en_reg.register, en_reg.field)).unwrap() > 1;
            let refcount_idx = if needs_refcount {
                let next_refcount_idx = refcount_idxs.len() as u8;
                let refcount_idx = *refcount_idxs
                    .entry((en_reg.register, en_reg.field))
                    .or_insert(next_refcount_idx);
                quote! { Some(#refcount_idx) }
            } else {
                quote! { None }
            };

            let clock_frequency = match &rcc.kernel_clock {
                PeripheralRccKernelClock::Mux(mux) => clock_gen.gen_mux(p.name, mux),
                PeripheralRccKernelClock::Clock(clock) => clock_gen.gen_clock(p.name, clock),
            };

            let bus_clock_frequency = clock_gen.gen_clock(p.name, &rcc.bus_clock);

            // A refcount leak can result if the same field is shared by peripherals with different stop modes
            // This condition should be checked in stm32-data
            let stop_mode = match rcc.stop_mode {
                StopMode::Standby => quote! { crate::rcc::StopMode::Standby },
                StopMode::Stop2 => quote! { crate::rcc::StopMode::Stop2 },
                StopMode::Stop1 => quote! { crate::rcc::StopMode::Stop1 },
            };

            g.extend(quote! {
                impl crate::rcc::SealedRccPeripheral for peripherals::#pname {
                    fn frequency() -> crate::time::Hertz {
                        #clock_frequency
                    }
                    fn bus_frequency() -> crate::time::Hertz {
                        #bus_clock_frequency
                    }

                    const RCC_INFO: crate::rcc::RccInfo = unsafe {
                        crate::rcc::RccInfo::new(
                            #reset_offset_and_bit,
                            #enable_offset_and_bit,
                            #refcount_idx,
                            #[cfg(feature = "low-power")]
                            #stop_mode,
                        )
                    };
                }

                impl crate::rcc::RccPeripheral for peripherals::#pname {}
            });
        }
    }

    g.extend({
        let refcounts_len = refcount_idxs.len();
        let refcount_zeros: TokenStream = refcount_idxs.iter().map(|_| quote! { 0u8, }).collect();
        quote! {
            pub(crate) static mut REFCOUNTS: [u8; #refcounts_len] = [#refcount_zeros];
        }
    });

    let struct_fields: Vec<_> = clock_gen
        .muxes
        .iter()
        .map(|(_fieldset, fieldname, enum_name)| {
            quote! {
                pub #fieldname: #enum_name
            }
        })
        .collect();

    let mut inits = TokenStream::new();
    for fieldset in clock_gen
        .muxes
        .iter()
        .map(|(f, _, _)| f)
        .collect::<BTreeSet<_>>()
        .into_iter()
    {
        let setters: Vec<_> = clock_gen
            .muxes
            .iter()
            .filter(|(f, _, _)| f == fieldset)
            .map(|(_, fieldname, _)| {
                let setter = format_ident!("set_{}", fieldname);
                quote! {
                    w.#setter(self.#fieldname);
                }
            })
            .collect();

        inits.extend(quote! {
            crate::pac::RCC.#fieldset().modify(|w| {
                #(#setters)*
            });
        })
    }

    let enum_names: BTreeSet<_> = clock_gen.muxes.iter().map(|(_, _, enum_name)| enum_name).collect();

    g.extend(quote! {
        pub mod mux {
            #(pub use crate::pac::rcc::vals::#enum_names as #enum_names; )*

            #[derive(Clone, Copy)]
            #[non_exhaustive]
            pub struct ClockMux {
                #( #struct_fields, )*
            }

            impl ClockMux {
                pub(crate) const fn default() -> Self {
                    // safety: zero value is valid for all PAC enums.
                    unsafe { ::core::mem::zeroed() }
                }
            }

            impl Default for ClockMux {
                fn default() -> Self {
                    Self::default()
                }
            }

            impl ClockMux {
                pub(crate) fn init(&self) {
                    #inits
                }
            }
        }
    });

    // Generate RCC
    clock_gen.clock_names.insert("sys".to_string());
    clock_gen.clock_names.insert("rtc".to_string());

    // STM32F4 SPI in I2S mode receives a clock input from the dedicated I2S PLL.
    // For this, there is an additional clock MUX, which is not present in other
    // peripherals and does not fit the current RCC structure of stm32-data.
    if chip_name.starts_with("stm32f4") && !chip_name.starts_with("stm32f410") {
        clock_gen.clock_names.insert("plli2s1_p".to_string());
        clock_gen.clock_names.insert("plli2s1_q".to_string());
        clock_gen.clock_names.insert("plli2s1_r".to_string());
    }

    let clock_idents: Vec<_> = clock_gen.clock_names.iter().map(|n| format_ident!("{}", n)).collect();
    g.extend(quote! {
        #[derive(Clone, Copy, Debug)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        #[repr(C)]
        pub struct Clocks {
            #(
                pub #clock_idents: crate::time::MaybeHertz,
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
                        #( #clock_idents: all.#clock_idents.into(), )*
                    });
                }
            };
        }
    );

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
        (("ucpd", "CC1"), quote!(crate::ucpd::Cc1Pin)),
        (("ucpd", "CC2"), quote!(crate::ucpd::Cc2Pin)),
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
        (("dsihost", "TE"), quote!(crate::dsihost::TePin)),
        (("ltdc", "CLK"), quote!(crate::ltdc::ClkPin)),
        (("ltdc", "HSYNC"), quote!(crate::ltdc::HsyncPin)),
        (("ltdc", "VSYNC"), quote!(crate::ltdc::VsyncPin)),
        (("ltdc", "DE"), quote!(crate::ltdc::DePin)),
        (("ltdc", "R0"), quote!(crate::ltdc::R0Pin)),
        (("ltdc", "R1"), quote!(crate::ltdc::R1Pin)),
        (("ltdc", "R2"), quote!(crate::ltdc::R2Pin)),
        (("ltdc", "R3"), quote!(crate::ltdc::R3Pin)),
        (("ltdc", "R4"), quote!(crate::ltdc::R4Pin)),
        (("ltdc", "R5"), quote!(crate::ltdc::R5Pin)),
        (("ltdc", "R6"), quote!(crate::ltdc::R6Pin)),
        (("ltdc", "R7"), quote!(crate::ltdc::R7Pin)),
        (("ltdc", "G0"), quote!(crate::ltdc::G0Pin)),
        (("ltdc", "G1"), quote!(crate::ltdc::G1Pin)),
        (("ltdc", "G2"), quote!(crate::ltdc::G2Pin)),
        (("ltdc", "G3"), quote!(crate::ltdc::G3Pin)),
        (("ltdc", "G4"), quote!(crate::ltdc::G4Pin)),
        (("ltdc", "G5"), quote!(crate::ltdc::G5Pin)),
        (("ltdc", "G6"), quote!(crate::ltdc::G6Pin)),
        (("ltdc", "G7"), quote!(crate::ltdc::G7Pin)),
        (("ltdc", "B0"), quote!(crate::ltdc::B0Pin)),
        (("ltdc", "B1"), quote!(crate::ltdc::B1Pin)),
        (("ltdc", "B2"), quote!(crate::ltdc::B2Pin)),
        (("ltdc", "B3"), quote!(crate::ltdc::B3Pin)),
        (("ltdc", "B4"), quote!(crate::ltdc::B4Pin)),
        (("ltdc", "B5"), quote!(crate::ltdc::B5Pin)),
        (("ltdc", "B6"), quote!(crate::ltdc::B6Pin)),
        (("ltdc", "B7"), quote!(crate::ltdc::B7Pin)),
        (("usb", "DP"), quote!(crate::usb::DpPin)),
        (("usb", "DM"), quote!(crate::usb::DmPin)),
        (("usb", "SOF"), quote!(crate::usb::SofPin)),
        (("otg", "DP"), quote!(crate::usb::DpPin)),
        (("otg", "DM"), quote!(crate::usb::DmPin)),
        (("otg", "ULPI_CK"), quote!(crate::usb::UlpiClkPin)),
        (("otg", "ULPI_DIR"), quote!(crate::usb::UlpiDirPin)),
        (("otg", "ULPI_NXT"), quote!(crate::usb::UlpiNxtPin)),
        (("otg", "ULPI_STP"), quote!(crate::usb::UlpiStpPin)),
        (("otg", "ULPI_D0"), quote!(crate::usb::UlpiD0Pin)),
        (("otg", "ULPI_D1"), quote!(crate::usb::UlpiD1Pin)),
        (("otg", "ULPI_D2"), quote!(crate::usb::UlpiD2Pin)),
        (("otg", "ULPI_D3"), quote!(crate::usb::UlpiD3Pin)),
        (("otg", "ULPI_D4"), quote!(crate::usb::UlpiD4Pin)),
        (("otg", "ULPI_D5"), quote!(crate::usb::UlpiD5Pin)),
        (("otg", "ULPI_D6"), quote!(crate::usb::UlpiD6Pin)),
        (("otg", "ULPI_D7"), quote!(crate::usb::UlpiD7Pin)),
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
        (("lptim", "CH1"), quote!(crate::lptim::Channel1Pin)),
        (("lptim", "CH2"), quote!(crate::lptim::Channel2Pin)),
        (("lptim", "OUT"), quote!(crate::lptim::OutputPin)),
        (("sdmmc", "CK"), quote!(crate::sdmmc::CkPin)),
        (("sdmmc", "CMD"), quote!(crate::sdmmc::CmdPin)),
        (("sdmmc", "D0"), quote!(crate::sdmmc::D0Pin)),
        (("sdmmc", "D1"), quote!(crate::sdmmc::D1Pin)),
        (("sdmmc", "D2"), quote!(crate::sdmmc::D2Pin)),
        (("sdmmc", "D3"), quote!(crate::sdmmc::D3Pin)),
        (("sdmmc", "D4"), quote!(crate::sdmmc::D4Pin)),
        (("sdmmc", "D5"), quote!(crate::sdmmc::D5Pin)),
        (("sdmmc", "D6"), quote!(crate::sdmmc::D6Pin)),
        (("sdmmc", "D7"), quote!(crate::sdmmc::D7Pin)),
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
        (("octospi", "IO0"), quote!(crate::ospi::D0Pin)),
        (("octospi", "IO1"), quote!(crate::ospi::D1Pin)),
        (("octospi", "IO2"), quote!(crate::ospi::D2Pin)),
        (("octospi", "IO3"), quote!(crate::ospi::D3Pin)),
        (("octospi", "IO4"), quote!(crate::ospi::D4Pin)),
        (("octospi", "IO5"), quote!(crate::ospi::D5Pin)),
        (("octospi", "IO6"), quote!(crate::ospi::D6Pin)),
        (("octospi", "IO7"), quote!(crate::ospi::D7Pin)),
        (("octospi", "DQS"), quote!(crate::ospi::DQSPin)),
        (("octospi", "NCS"), quote!(crate::ospi::NSSPin)),
        (("octospi", "CLK"), quote!(crate::ospi::SckPin)),
        (("octospi", "NCLK"), quote!(crate::ospi::NckPin)),
        (("octospim", "P1_IO0"), quote!(crate::ospi::D0Pin)),
        (("octospim", "P1_IO1"), quote!(crate::ospi::D1Pin)),
        (("octospim", "P1_IO2"), quote!(crate::ospi::D2Pin)),
        (("octospim", "P1_IO3"), quote!(crate::ospi::D3Pin)),
        (("octospim", "P1_IO4"), quote!(crate::ospi::D4Pin)),
        (("octospim", "P1_IO5"), quote!(crate::ospi::D5Pin)),
        (("octospim", "P1_IO6"), quote!(crate::ospi::D6Pin)),
        (("octospim", "P1_IO7"), quote!(crate::ospi::D7Pin)),
        (("octospim", "P1_DQS"), quote!(crate::ospi::DQSPin)),
        (("octospim", "P1_NCS"), quote!(crate::ospi::NSSPin)),
        (("octospim", "P1_CLK"), quote!(crate::ospi::SckPin)),
        (("octospim", "P1_NCLK"), quote!(crate::ospi::NckPin)),
        (("octospim", "P2_IO0"), quote!(crate::ospi::D0Pin)),
        (("octospim", "P2_IO1"), quote!(crate::ospi::D1Pin)),
        (("octospim", "P2_IO2"), quote!(crate::ospi::D2Pin)),
        (("octospim", "P2_IO3"), quote!(crate::ospi::D3Pin)),
        (("octospim", "P2_IO4"), quote!(crate::ospi::D4Pin)),
        (("octospim", "P2_IO5"), quote!(crate::ospi::D5Pin)),
        (("octospim", "P2_IO6"), quote!(crate::ospi::D6Pin)),
        (("octospim", "P2_IO7"), quote!(crate::ospi::D7Pin)),
        (("octospim", "P2_DQS"), quote!(crate::ospi::DQSPin)),
        (("octospim", "P2_NCS"), quote!(crate::ospi::NSSPin)),
        (("octospim", "P2_CLK"), quote!(crate::ospi::SckPin)),
        (("octospim", "P2_NCLK"), quote!(crate::ospi::NckPin)),
        (("xspi", "IO0"), quote!(crate::xspi::D0Pin)),
        (("xspi", "IO1"), quote!(crate::xspi::D1Pin)),
        (("xspi", "IO2"), quote!(crate::xspi::D2Pin)),
        (("xspi", "IO3"), quote!(crate::xspi::D3Pin)),
        (("xspi", "IO4"), quote!(crate::xspi::D4Pin)),
        (("xspi", "IO5"), quote!(crate::xspi::D5Pin)),
        (("xspi", "IO6"), quote!(crate::xspi::D6Pin)),
        (("xspi", "IO7"), quote!(crate::xspi::D7Pin)),
        (("xspi", "IO8"), quote!(crate::xspi::D8Pin)),
        (("xspi", "IO9"), quote!(crate::xspi::D9Pin)),
        (("xspi", "IO10"), quote!(crate::xspi::D10Pin)),
        (("xspi", "IO11"), quote!(crate::xspi::D11Pin)),
        (("xspi", "IO12"), quote!(crate::xspi::D12Pin)),
        (("xspi", "IO13"), quote!(crate::xspi::D13Pin)),
        (("xspi", "IO14"), quote!(crate::xspi::D14Pin)),
        (("xspi", "IO15"), quote!(crate::xspi::D15Pin)),
        (("xspi", "DQS0"), quote!(crate::xspi::DQS0Pin)),
        (("xspi", "DQS1"), quote!(crate::xspi::DQS1Pin)),
        (("xspi", "NCS1"), quote!(crate::xspi::NCSPin)),
        (("xspi", "NCS2"), quote!(crate::xspi::NCSPin)),
        (("xspi", "CLK"), quote!(crate::xspi::CLKPin)),
        (("xspi", "NCLK"), quote!(crate::xspi::NCLKPin)),
        (("xspim", "P1_IO0"), quote!(crate::xspi::D0Pin)),
        (("xspim", "P1_IO1"), quote!(crate::xspi::D1Pin)),
        (("xspim", "P1_IO2"), quote!(crate::xspi::D2Pin)),
        (("xspim", "P1_IO3"), quote!(crate::xspi::D3Pin)),
        (("xspim", "P1_IO4"), quote!(crate::xspi::D4Pin)),
        (("xspim", "P1_IO5"), quote!(crate::xspi::D5Pin)),
        (("xspim", "P1_IO6"), quote!(crate::xspi::D6Pin)),
        (("xspim", "P1_IO7"), quote!(crate::xspi::D7Pin)),
        (("xspim", "P1_IO8"), quote!(crate::xspi::D8Pin)),
        (("xspim", "P1_IO9"), quote!(crate::xspi::D9Pin)),
        (("xspim", "P1_IO10"), quote!(crate::xspi::D10Pin)),
        (("xspim", "P1_IO11"), quote!(crate::xspi::D11Pin)),
        (("xspim", "P1_IO12"), quote!(crate::xspi::D12Pin)),
        (("xspim", "P1_IO13"), quote!(crate::xspi::D13Pin)),
        (("xspim", "P1_IO14"), quote!(crate::xspi::D14Pin)),
        (("xspim", "P1_IO15"), quote!(crate::xspi::D15Pin)),
        (("xspim", "P1_DQS0"), quote!(crate::xspi::DQS0Pin)),
        (("xspim", "P1_DQS1"), quote!(crate::xspi::DQS1Pin)),
        (("xspim", "P1_NCS1"), quote!(crate::xspi::NCSPin)),
        (("xspim", "P1_NCS2"), quote!(crate::xspi::NCSPin)),
        (("xspim", "P1_CLK"), quote!(crate::xspi::CLKPin)),
        (("xspim", "P1_NCLK"), quote!(crate::xspi::NCLKPin)),
        (("xspim", "P2_IO0"), quote!(crate::xspi::D0Pin)),
        (("xspim", "P2_IO1"), quote!(crate::xspi::D1Pin)),
        (("xspim", "P2_IO2"), quote!(crate::xspi::D2Pin)),
        (("xspim", "P2_IO3"), quote!(crate::xspi::D3Pin)),
        (("xspim", "P2_IO4"), quote!(crate::xspi::D4Pin)),
        (("xspim", "P2_IO5"), quote!(crate::xspi::D5Pin)),
        (("xspim", "P2_IO6"), quote!(crate::xspi::D6Pin)),
        (("xspim", "P2_IO7"), quote!(crate::xspi::D7Pin)),
        (("xspim", "P2_IO8"), quote!(crate::xspi::D8Pin)),
        (("xspim", "P2_IO9"), quote!(crate::xspi::D9Pin)),
        (("xspim", "P2_IO10"), quote!(crate::xspi::D10Pin)),
        (("xspim", "P2_IO11"), quote!(crate::xspi::D11Pin)),
        (("xspim", "P2_IO12"), quote!(crate::xspi::D12Pin)),
        (("xspim", "P2_IO13"), quote!(crate::xspi::D13Pin)),
        (("xspim", "P2_IO14"), quote!(crate::xspi::D14Pin)),
        (("xspim", "P2_IO15"), quote!(crate::xspi::D15Pin)),
        (("xspim", "P2_DQS0"), quote!(crate::xspi::DQS0Pin)),
        (("xspim", "P2_DQS1"), quote!(crate::xspi::DQS1Pin)),
        (("xspim", "P2_NCS1"), quote!(crate::xspi::NCSPin)),
        (("xspim", "P2_NCS2"), quote!(crate::xspi::NCSPin)),
        (("xspim", "P2_CLK"), quote!(crate::xspi::CLKPin)),
        (("xspim", "P2_NCLK"), quote!(crate::xspi::NCLKPin)),
        (("hspi", "IO0"), quote!(crate::hspi::D0Pin)),
        (("hspi", "IO1"), quote!(crate::hspi::D1Pin)),
        (("hspi", "IO2"), quote!(crate::hspi::D2Pin)),
        (("hspi", "IO3"), quote!(crate::hspi::D3Pin)),
        (("hspi", "IO4"), quote!(crate::hspi::D4Pin)),
        (("hspi", "IO5"), quote!(crate::hspi::D5Pin)),
        (("hspi", "IO6"), quote!(crate::hspi::D6Pin)),
        (("hspi", "IO7"), quote!(crate::hspi::D7Pin)),
        (("hspi", "IO8"), quote!(crate::hspi::D8Pin)),
        (("hspi", "IO9"), quote!(crate::hspi::D9Pin)),
        (("hspi", "IO10"), quote!(crate::hspi::D10Pin)),
        (("hspi", "IO11"), quote!(crate::hspi::D11Pin)),
        (("hspi", "IO12"), quote!(crate::hspi::D12Pin)),
        (("hspi", "IO13"), quote!(crate::hspi::D13Pin)),
        (("hspi", "IO14"), quote!(crate::hspi::D14Pin)),
        (("hspi", "IO15"), quote!(crate::hspi::D15Pin)),
        (("hspi", "DQS0"), quote!(crate::hspi::DQS0Pin)),
        (("hspi", "DQS1"), quote!(crate::hspi::DQS1Pin)),
        (("hspi", "NCS"), quote!(crate::hspi::NSSPin)),
        (("hspi", "CLK"), quote!(crate::hspi::SckPin)),
        (("hspi", "NCLK"), quote!(crate::hspi::NckPin)),
        (("tsc", "G1_IO1"), quote!(crate::tsc::G1IO1Pin)),
        (("tsc", "G1_IO2"), quote!(crate::tsc::G1IO2Pin)),
        (("tsc", "G1_IO3"), quote!(crate::tsc::G1IO3Pin)),
        (("tsc", "G1_IO4"), quote!(crate::tsc::G1IO4Pin)),
        (("tsc", "G2_IO1"), quote!(crate::tsc::G2IO1Pin)),
        (("tsc", "G2_IO2"), quote!(crate::tsc::G2IO2Pin)),
        (("tsc", "G2_IO3"), quote!(crate::tsc::G2IO3Pin)),
        (("tsc", "G2_IO4"), quote!(crate::tsc::G2IO4Pin)),
        (("tsc", "G3_IO1"), quote!(crate::tsc::G3IO1Pin)),
        (("tsc", "G3_IO2"), quote!(crate::tsc::G3IO2Pin)),
        (("tsc", "G3_IO3"), quote!(crate::tsc::G3IO3Pin)),
        (("tsc", "G3_IO4"), quote!(crate::tsc::G3IO4Pin)),
        (("tsc", "G4_IO1"), quote!(crate::tsc::G4IO1Pin)),
        (("tsc", "G4_IO2"), quote!(crate::tsc::G4IO2Pin)),
        (("tsc", "G4_IO3"), quote!(crate::tsc::G4IO3Pin)),
        (("tsc", "G4_IO4"), quote!(crate::tsc::G4IO4Pin)),
        (("tsc", "G5_IO1"), quote!(crate::tsc::G5IO1Pin)),
        (("tsc", "G5_IO2"), quote!(crate::tsc::G5IO2Pin)),
        (("tsc", "G5_IO3"), quote!(crate::tsc::G5IO3Pin)),
        (("tsc", "G5_IO4"), quote!(crate::tsc::G5IO4Pin)),
        (("tsc", "G6_IO1"), quote!(crate::tsc::G6IO1Pin)),
        (("tsc", "G6_IO2"), quote!(crate::tsc::G6IO2Pin)),
        (("tsc", "G6_IO3"), quote!(crate::tsc::G6IO3Pin)),
        (("tsc", "G6_IO4"), quote!(crate::tsc::G6IO4Pin)),
        (("tsc", "G7_IO1"), quote!(crate::tsc::G7IO1Pin)),
        (("tsc", "G7_IO2"), quote!(crate::tsc::G7IO2Pin)),
        (("tsc", "G7_IO3"), quote!(crate::tsc::G7IO3Pin)),
        (("tsc", "G7_IO4"), quote!(crate::tsc::G7IO4Pin)),
        (("tsc", "G8_IO1"), quote!(crate::tsc::G8IO1Pin)),
        (("tsc", "G8_IO2"), quote!(crate::tsc::G8IO2Pin)),
        (("tsc", "G8_IO3"), quote!(crate::tsc::G8IO3Pin)),
        (("tsc", "G8_IO4"), quote!(crate::tsc::G8IO4Pin)),
        (("dac", "OUT1"), quote!(crate::dac::DacPin<Ch1>)),
        (("dac", "OUT2"), quote!(crate::dac::DacPin<Ch2>)),
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

                    // OCTOSPIM is special
                    if p.name == "OCTOSPIM" {
                        // Some chips have OCTOSPIM but not OCTOSPI2.
                        if METADATA.peripherals.iter().any(|p| p.name == "OCTOSPI2") {
                            peri = format_ident!("{}", "OCTOSPI2");
                            g.extend(quote! {
                                pin_trait_impl!(#tr, #peri, #pin_name, #af);
                            });
                        }
                        peri = format_ident!("{}", "OCTOSPI1");
                    }

                    // XSPIM  is special
                    if p.name == "XSPIM" {
                        if pin.signal.starts_with("P1") {
                            peri = format_ident!("{}", "XSPI1");
                        } else if pin.signal.starts_with("P2") {
                            peri = format_ident!("{}", "XSPI2");
                        } else {
                            panic! {"malformed XSPIM pin: {:?}", pin}
                        }
                    }

                    // XSPI NCS pin to CSSEL mapping
                    if pin.signal.ends_with("NCS1") {
                        g.extend(quote! {
                            sel_trait_impl!(crate::xspi::NCSEither, #peri, #pin_name, 0);
                        })
                    }
                    if pin.signal.ends_with("NCS2") {
                        g.extend(quote! {
                            sel_trait_impl!(crate::xspi::NCSEither, #peri, #pin_name, 1);
                        })
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
                    } else if pin.signal.starts_with("VINM") {
                        // Impl NonInvertingPin for the VINM* signals ( VINM0, VINM1, etc)
                        // STM32G4
                        let peri = format_ident!("{}", p.name);
                        let pin_name = format_ident!("{}", pin.pin);
                        let ch: Result<u8, _> = pin.signal.strip_prefix("VINM").unwrap().parse();

                        if let Ok(ch) = ch {
                            g.extend(quote! {
                                impl_opamp_vn_pin!( #peri, #pin_name, #ch);
                            })
                        }
                    } else if pin.signal == "VOUT" {
                        // Impl OutputPin for the VOUT pin
                        let peri = format_ident!("{}", p.name);
                        let pin_name = format_ident!("{}", pin.pin);
                        g.extend(quote! {
                            impl_opamp_vout_pin!( #peri, #pin_name );
                        })
                    }
                }

                if regs.kind == "spdifrx" {
                    let peri = format_ident!("{}", p.name);
                    let pin_name = format_ident!("{}", pin.pin);
                    let af = pin.af.unwrap_or(0);
                    let sel: u8 = pin.signal.strip_prefix("IN").unwrap().parse().unwrap();

                    g.extend(quote! {
                    impl_spdifrx_pin!( #peri, #pin_name, #af, #sel);
                    })
                }
            }
        }
    }

    // ========
    // Generate dma_trait_impl!

    let mut signals: HashMap<_, _> = [
        // (kind, signal) => trait
        (("adc", "ADC"), quote!(crate::adc::RxDma)),
        (("adc", "ADC1"), quote!(crate::adc::RxDma)),
        (("adc", "ADC2"), quote!(crate::adc::RxDma)),
        (("adc", "ADC3"), quote!(crate::adc::RxDma)),
        (("ucpd", "RX"), quote!(crate::ucpd::RxDma)),
        (("ucpd", "TX"), quote!(crate::ucpd::TxDma)),
        (("usart", "RX"), quote!(crate::usart::RxDma)),
        (("usart", "TX"), quote!(crate::usart::TxDma)),
        (("lpuart", "RX"), quote!(crate::usart::RxDma)),
        (("lpuart", "TX"), quote!(crate::usart::TxDma)),
        (("sai", "A"), quote!(crate::sai::Dma<A>)),
        (("sai", "B"), quote!(crate::sai::Dma<B>)),
        (("spi", "RX"), quote!(crate::spi::RxDma)),
        (("spi", "TX"), quote!(crate::spi::TxDma)),
        (("spdifrx", "RX"), quote!(crate::spdifrx::Dma)),
        (("i2c", "RX"), quote!(crate::i2c::RxDma)),
        (("i2c", "TX"), quote!(crate::i2c::TxDma)),
        (("dcmi", "DCMI"), quote!(crate::dcmi::FrameDma)),
        (("dcmi", "PSSI"), quote!(crate::dcmi::FrameDma)),
        // SDMMCv1 uses the same channel for both directions, so just implement for RX
        (("sdmmc", "RX"), quote!(crate::sdmmc::SdmmcDma)),
        (("quadspi", "QUADSPI"), quote!(crate::qspi::QuadDma)),
        (("octospi", "OCTOSPI1"), quote!(crate::ospi::OctoDma)),
        (("hspi", "HSPI1"), quote!(crate::hspi::HspiDma)),
        (("dac", "CH1"), quote!(crate::dac::Dma<Ch1>)),
        (("dac", "CH2"), quote!(crate::dac::Dma<Ch2>)),
        (("timer", "UP"), quote!(crate::timer::UpDma)),
        (("hash", "IN"), quote!(crate::hash::Dma)),
        (("cryp", "IN"), quote!(crate::cryp::DmaIn)),
        (("cryp", "OUT"), quote!(crate::cryp::DmaOut)),
        (("timer", "CH1"), quote!(crate::timer::Ch1Dma)),
        (("timer", "CH2"), quote!(crate::timer::Ch2Dma)),
        (("timer", "CH3"), quote!(crate::timer::Ch3Dma)),
        (("timer", "CH4"), quote!(crate::timer::Ch4Dma)),
        (("cordic", "WRITE"), quote!(crate::cordic::WriteDma)), // FIXME: stm32u5a crash on Cordic driver
        (("cordic", "READ"), quote!(crate::cordic::ReadDma)),   // FIXME: stm32u5a crash on Cordic driver
    ]
    .into();

    if chip_name.starts_with("stm32u5") {
        signals.insert(("adc", "ADC4"), quote!(crate::adc::RxDma4));
    } else {
        signals.insert(("adc", "ADC4"), quote!(crate::adc::RxDma));
    }

    if chip_name.starts_with("stm32g4") {
        let line_number = chip_name.chars().skip(8).next().unwrap();
        if line_number == '3' || line_number == '4' {
            signals.insert(("adc", "ADC5"), quote!(crate::adc::RxDma));
        }
    }

    for p in METADATA.peripherals {
        if let Some(regs) = &p.registers {
            // FIXME: stm32u5a crash on Cordic driver
            if chip_name.starts_with("stm32u5a") && regs.kind == "cordic" {
                continue;
            }

            let mut dupe = HashSet::new();
            for ch in p.dma_channels {
                if let Some(tr) = signals.get(&(regs.kind, ch.signal)) {
                    let peri = format_ident!("{}", p.name);

                    let channels = if let Some(channel) = &ch.channel {
                        // Chip with DMA/BDMA, without DMAMUX
                        vec![*channel]
                    } else if let Some(dmamux) = &ch.dmamux {
                        // Chip with DMAMUX
                        METADATA
                            .dma_channels
                            .iter()
                            .filter(|ch| ch.dmamux == Some(*dmamux))
                            .map(|ch| ch.name)
                            .collect()
                    } else if let Some(dma) = &ch.dma {
                        // Chip with GPDMA
                        METADATA
                            .dma_channels
                            .iter()
                            .filter(|ch| ch.dma == *dma)
                            .map(|ch| ch.name)
                            .collect()
                    } else {
                        unreachable!();
                    };

                    for channel in channels {
                        // Some chips have multiple request numbers for the same (peri, signal, channel) combos.
                        // Ignore the dupes, picking the first one. Otherwise this causes conflicting trait impls
                        let key = (ch.signal, channel.to_string());
                        if !dupe.insert(key) {
                            continue;
                        }

                        let request = if let Some(request) = ch.request {
                            let request = request as u8;
                            quote!(#request)
                        } else {
                            quote!(())
                        };

                        let channel = format_ident!("{}", channel);
                        g.extend(quote! {
                            dma_trait_impl!(#tr, #peri, #channel, #request);
                        });
                    }
                }
            }
        }
    }

    // ========
    // Generate Div/Mul impls for RCC prescalers/dividers/multipliers.
    for e in rcc_registers.ir.enums {
        fn is_rcc_name(e: &str) -> bool {
            match e {
                "Pllp" | "Pllq" | "Pllr" | "Pllm" | "Plln" | "Prediv1" | "Prediv2" => true,
                "Timpre" | "Pllrclkpre" => false,
                e if e.ends_with("pre") || e.ends_with("pres") || e.ends_with("div") || e.ends_with("mul") => true,
                _ => false,
            }
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
    let mut adc_table: Vec<Vec<String>> = Vec::new();

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

    for pin in METADATA.pins {
        let port_letter = pin.name.chars().nth(1).unwrap();
        let pname = format!("GPIO{}", port_letter);
        let p = METADATA.peripherals.iter().find(|p| p.name == pname).unwrap();
        assert_eq!(0, (p.address as u32 - gpio_base) % gpio_stride);
        let port_num = (p.address as u32 - gpio_base) / gpio_stride;
        let pin_num: u32 = pin.name[2..].parse().unwrap();

        pins_table.push(vec![
            pin.name.to_string(),
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
            if split_feature.pin_name_without_c == pin.name {
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

    for p in METADATA.peripherals {
        if let Some(regs) = &p.registers {
            if regs.kind == "adc" {
                let adc_num = p.name.strip_prefix("ADC").unwrap();
                let mut adc_common = None;
                for p2 in METADATA.peripherals {
                    if let Some(common_nums) = p2.name.strip_prefix("ADC").and_then(|s| s.strip_suffix("_COMMON")) {
                        if common_nums.contains(adc_num) {
                            adc_common = Some(p2);
                        }
                    }
                }
                let adc_common = adc_common.map(|p| p.name).unwrap_or("none");
                let row = vec![p.name.to_string(), adc_common.to_string(), "adc".to_string()];
                adc_table.push(row);
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

    let mut dmas = TokenStream::new();
    let has_dmamux = METADATA
        .peripherals
        .iter()
        .flat_map(|p| &p.registers)
        .any(|p| p.kind == "dmamux");

    let mut dma_irqs: BTreeMap<&str, Vec<String>> = BTreeMap::new();

    for p in METADATA.peripherals {
        if let Some(r) = &p.registers {
            if r.kind == "dma" || r.kind == "bdma" || r.kind == "gpdma" || r.kind == "lpdma" {
                for irq in p.interrupts {
                    let ch_name = format!("{}_{}", p.name, irq.signal);
                    let ch = METADATA.dma_channels.iter().find(|c| c.name == ch_name).unwrap();

                    // Some H7 chips have BDMA1 hardcoded for DFSDM, ie no DMAMUX. It's unsupported, skip it.
                    if has_dmamux && ch.dmamux.is_none() {
                        continue;
                    }

                    dma_irqs.entry(irq.interrupt).or_default().push(ch_name);
                }
            }
        }
    }

    #[cfg(feature = "_dual-core")]
    let mut dma_ch_to_irq: BTreeMap<&str, Vec<String>> = BTreeMap::new();

    #[cfg(feature = "_dual-core")]
    for (irq, channels) in &dma_irqs {
        for channel in channels {
            dma_ch_to_irq.entry(channel).or_default().push(irq.to_string());
        }
    }

    for (ch_idx, ch) in METADATA.dma_channels.iter().enumerate() {
        // Some H7 chips have BDMA1 hardcoded for DFSDM, ie no DMAMUX. It's unsupported, skip it.
        if has_dmamux && ch.dmamux.is_none() {
            continue;
        }

        let name = format_ident!("{}", ch.name);
        let idx = ch_idx as u8;
        #[cfg(feature = "_dual-core")]
        let irq = {
            let irq_name = if let Some(x) = &dma_ch_to_irq.get(ch.name) {
                format_ident!("{}", x.get(0).unwrap())
            } else {
                panic!("failed to find dma interrupt")
            };
            quote!(crate::pac::Interrupt::#irq_name)
        };

        g.extend(quote!(dma_channel_impl!(#name, #idx);));

        let dma = format_ident!("{}", ch.dma);
        let ch_num = ch.channel as usize;

        let dma_peri = METADATA.peripherals.iter().find(|p| p.name == ch.dma).unwrap();
        let bi = dma_peri.registers.as_ref().unwrap();

        let dma_info = match bi.kind {
            "dma" => quote!(crate::dma::DmaInfo::Dma(crate::pac::#dma)),
            "bdma" => quote!(crate::dma::DmaInfo::Bdma(crate::pac::#dma)),
            "gpdma" => quote!(crate::pac::#dma),
            "lpdma" => quote!(unsafe { crate::pac::gpdma::Gpdma::from_ptr(crate::pac::#dma.as_ptr())}),
            _ => panic!("bad dma channel kind {}", bi.kind),
        };

        let dmamux = match &ch.dmamux {
            Some(dmamux) => {
                let dmamux = format_ident!("{}", dmamux);
                let num = ch.dmamux_channel.unwrap() as usize;
                quote! {
                    dmamux: crate::dma::DmamuxInfo {
                        mux: crate::pac::#dmamux,
                        num: #num,
                    },
                }
            }
            None => quote!(),
        };

        #[cfg(not(feature = "_dual-core"))]
        dmas.extend(quote! {
            crate::dma::ChannelInfo {
                dma: #dma_info,
                num: #ch_num,
                #dmamux
            },
        });
        #[cfg(feature = "_dual-core")]
        dmas.extend(quote! {
            crate::dma::ChannelInfo {
                dma: #dma_info,
                num: #ch_num,
                irq: #irq,
                #dmamux
            },
        });
    }

    // ========
    // Generate DMA IRQs.

    let dma_irqs: TokenStream = dma_irqs
        .iter()
        .map(|(irq, channels)| {
            let irq = format_ident!("{}", irq);

            let channels = channels.iter().map(|c| format_ident!("{}", c));

            quote! {
                #[cfg(feature = "rt")]
                #[crate::interrupt]
                unsafe fn #irq () {
                    #(
                        <crate::peripherals::#channels as crate::dma::ChannelInterrupt>::on_irq();
                    )*
                }
            }
        })
        .collect();

    g.extend(dma_irqs);

    g.extend(quote! {
        pub(crate) const DMA_CHANNELS: &[crate::dma::ChannelInfo] = &[#dmas];
    });

    // ========
    // Generate gpio_block() function

    let gpio_base = METADATA.peripherals.iter().find(|p| p.name == "GPIOA").unwrap().address as usize;
    let gpio_stride = 0x400 as usize;

    for p in METADATA.peripherals {
        if let Some(bi) = &p.registers {
            if bi.kind == "gpio" {
                assert_eq!(0, (p.address as usize - gpio_base) % gpio_stride);
            }
        }
    }

    g.extend(quote!(
        pub fn gpio_block(n: usize) -> crate::pac::gpio::Gpio {{
            unsafe {{ crate::pac::gpio::Gpio::from_ptr((#gpio_base + #gpio_stride*n) as _) }}
        }}
    ));

    // ========
    // Generate flash constants

    let flash_regions: Vec<&MemoryRegion> = METADATA
        .memory
        .iter()
        .filter(|x| x.kind == MemoryRegionKind::Flash && x.name.starts_with("BANK_"))
        .collect();
    let first_flash = flash_regions.first().unwrap();
    let total_flash_size = flash_regions
        .iter()
        .map(|x| x.size)
        .reduce(|acc, item| acc + item)
        .unwrap();
    let write_sizes: HashSet<_> = flash_regions
        .iter()
        .map(|r| r.settings.as_ref().unwrap().write_size)
        .collect();
    assert_eq!(1, write_sizes.len());

    let flash_base = first_flash.address as usize;
    let total_flash_size = total_flash_size as usize;
    let write_size = (*write_sizes.iter().next().unwrap()) as usize;

    g.extend(quote!(
        pub const FLASH_BASE: usize = #flash_base;
        pub const FLASH_SIZE: usize = #total_flash_size;
        pub const WRITE_SIZE: usize = #write_size;
    ));

    // ========
    // Generate macro-tables

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
    make_table(&mut m, "foreach_adc", &adc_table);

    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("_macros.rs").to_string_lossy().to_string();
    fs::write(&out_file, m).unwrap();
    rustfmt(&out_file);

    // ========
    // Write generated.rs

    let out_file = out_dir.join("_generated.rs").to_string_lossy().to_string();
    fs::write(&out_file, g.to_string()).unwrap();
    rustfmt(&out_file);

    // ========
    // Configs for multicore and for targeting groups of chips

    fn get_chip_cfgs(chip_name: &str) -> Vec<String> {
        let mut cfgs = Vec::new();

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
            cfgs.push(format!("{}_{}", &chip_name[..chip_name.len() - 2], core));
        }

        // Configs for targeting groups of chips
        if &chip_name[..8] == "stm32wba" {
            cfgs.push(chip_name[..8].to_owned()); // stm32wba
            cfgs.push(chip_name[..10].to_owned()); // stm32wba52
            cfgs.push(format!("package_{}", &chip_name[10..11]));
            cfgs.push(format!("flashsize_{}", &chip_name[11..12]));
        } else {
            if &chip_name[..8] == "stm32h7r" || &chip_name[..8] == "stm32h7s" {
                cfgs.push("stm32h7rs".to_owned());
            } else {
                cfgs.push(chip_name[..7].to_owned()); // stm32f4
            }
            cfgs.push(chip_name[..9].to_owned()); // stm32f429
            cfgs.push(format!("{}x", &chip_name[..8])); // stm32f42x
            cfgs.push(format!("{}x{}", &chip_name[..7], &chip_name[8..9])); // stm32f4x9
            cfgs.push(format!("package_{}", &chip_name[9..10]));
            cfgs.push(format!("flashsize_{}", &chip_name[10..11]));
        }

        // Mark the L4+ chips as they have many differences to regular L4.
        if &chip_name[..7] == "stm32l4" {
            if "pqrs".contains(&chip_name[7..8]) {
                cfgs.push("stm32l4_plus".to_owned());
            } else {
                cfgs.push("stm32l4_nonplus".to_owned());
            }
        }

        cfgs
    }

    cfgs.enable_all(&get_chip_cfgs(&chip_name));
    for &chip_name in ALL_CHIPS.iter() {
        cfgs.declare_all(&get_chip_cfgs(&chip_name.to_ascii_lowercase()));
    }

    println!("cargo:rerun-if-changed=build.rs");

    if cfg!(feature = "memory-x") {
        gen_memory_x(out_dir);
        println!("cargo:rustc-link-search={}", out_dir.display());
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

fn gen_memory_x(out_dir: &Path) {
    let mut memory_x = String::new();

    let flash = get_memory_range(MemoryRegionKind::Flash);
    let ram = get_memory_range(MemoryRegionKind::Ram);

    write!(memory_x, "MEMORY\n{{\n").unwrap();
    writeln!(
        memory_x,
        "    FLASH : ORIGIN = 0x{:08x}, LENGTH = {:>4}K /* {} */",
        flash.0,
        flash.1 / 1024,
        flash.2
    )
    .unwrap();
    writeln!(
        memory_x,
        "    RAM   : ORIGIN = 0x{:08x}, LENGTH = {:>4}K /* {} */",
        ram.0,
        ram.1 / 1024,
        ram.2
    )
    .unwrap();
    write!(memory_x, "}}").unwrap();

    std::fs::write(out_dir.join("memory.x"), memory_x.as_bytes()).unwrap();
}

fn get_memory_range(kind: MemoryRegionKind) -> (u32, u32, String) {
    let mut mems: Vec<_> = METADATA
        .memory
        .iter()
        .filter(|m| m.kind == kind && m.size != 0)
        .collect();
    mems.sort_by_key(|m| m.address);

    let mut start = u32::MAX;
    let mut end = u32::MAX;
    let mut names = Vec::new();
    let mut best: Option<(u32, u32, String)> = None;
    for m in mems {
        if !mem_filter(&METADATA.name, &m.name) {
            continue;
        }

        if m.address != end {
            names = Vec::new();
            start = m.address;
            end = m.address;
        }

        end += m.size;
        names.push(m.name.to_string());

        if best.is_none() || end - start > best.as_ref().unwrap().1 {
            best = Some((start, end - start, names.join(" + ")));
        }
    }

    best.unwrap()
}

fn mem_filter(chip: &str, region: &str) -> bool {
    // in STM32WB, SRAM2a/SRAM2b are reserved for the radio core.
    if chip.starts_with("STM32WB")
        && !chip.starts_with("STM32WBA")
        && !chip.starts_with("STM32WB0")
        && region.starts_with("SRAM2")
    {
        return false;
    }

    true
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
