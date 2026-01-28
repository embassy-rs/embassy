use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Write;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;
use std::{env, fs};

use common::CfgSet;
use mspm0_metapac::metadata::{ALL_CHIPS, METADATA};
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{format_ident, quote};

#[path = "./build_common.rs"]
mod common;

fn main() {
    let mut cfgs = common::CfgSet::new();
    common::set_target_cfgs(&mut cfgs);

    generate_code(&mut cfgs);
    select_gpio_features(&mut cfgs);
    interrupt_group_linker_magic();
}

fn generate_code(cfgs: &mut CfgSet) {
    #[cfg(any(feature = "rt"))]
    println!(
        "cargo:rustc-link-search={}",
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).display(),
    );

    cfgs.declare_all(&["gpio_pb", "gpio_pc", "int_group1", "unicomm"]);

    let chip_name = match env::vars()
        .map(|(a, _)| a)
        .filter(|x| x.starts_with("CARGO_FEATURE_MSPM0") || x.starts_with("CARGO_FEATURE_MSPS"))
        .get_one()
    {
        Ok(x) => x,
        Err(GetOneError::None) => panic!("No mspm0xx/mspsxx Cargo feature enabled"),
        Err(GetOneError::Multiple) => panic!("Multiple mspm0xx/mspsxx Cargo features enabled"),
    }
    .strip_prefix("CARGO_FEATURE_")
    .unwrap()
    .to_ascii_lowercase()
    .replace('_', "-");

    eprintln!("chip: {chip_name}");

    cfgs.enable_all(&get_chip_cfgs(&chip_name));
    for chip in ALL_CHIPS {
        cfgs.declare_all(&get_chip_cfgs(&chip));
    }

    let mut singletons = get_singletons(cfgs);

    time_driver(&mut singletons, cfgs);
    pin_features(&mut singletons);

    let mut g = TokenStream::new();

    g.extend(generate_singletons(&singletons));
    g.extend(generate_pincm_mapping());
    g.extend(generate_pin());
    g.extend(generate_timers());
    g.extend(generate_interrupts());
    g.extend(generate_peripheral_instances());
    g.extend(generate_pin_trait_impls());
    g.extend(generate_groups());
    g.extend(generate_dma_channel_count());
    g.extend(generate_adc_constants(cfgs));

    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("_generated.rs").to_string_lossy().to_string();
    fs::write(&out_file, g.to_string()).unwrap();
    rustfmt(&out_file);
}

fn get_chip_cfgs(chip_name: &str) -> Vec<String> {
    let mut cfgs = Vec::new();

    // GPIO on C110x is special as it does not belong to an interrupt group.
    if chip_name.starts_with("mspm0c1103") || chip_name.starts_with("mspm0c1104") || chip_name.starts_with("msps003f") {
        cfgs.push("mspm0c110x".to_string());
    }

    if chip_name.starts_with("mspm0c1105") || chip_name.starts_with("mspm0c1106") {
        cfgs.push("mspm0c1105_c1106".to_string());
    }

    // Family ranges (temporary until int groups are generated)
    //
    // TODO: Remove this once int group stuff is generated.
    if chip_name.starts_with("mspm0g110") {
        cfgs.push("mspm0g110x".to_string());
    }

    if chip_name.starts_with("mspm0g150") {
        cfgs.push("mspm0g150x".to_string());
    }

    if chip_name.starts_with("mspm0g151") {
        cfgs.push("mspm0g151x".to_string());
    }

    if chip_name.starts_with("mspm0g310") {
        cfgs.push("mspm0g310x".to_string());
    }

    if chip_name.starts_with("mspm0g350") {
        cfgs.push("mspm0g350x".to_string());
    }

    if chip_name.starts_with("mspm0g351") {
        cfgs.push("mspm0g351x".to_string());
    }

    if chip_name.starts_with("mspm0g518") {
        cfgs.push("mspm0g518x".to_string());
    }

    if chip_name.starts_with("mspm0h321") {
        cfgs.push("mspm0h321x".to_string());
    }

    if chip_name.starts_with("mspm0l110") {
        cfgs.push("mspm0l110x".to_string());
    }

    if chip_name.starts_with("mspm0l122") {
        cfgs.push("mspm0l122x".to_string());
    }

    if chip_name.starts_with("mspm0l130") {
        cfgs.push("mspm0l130x".to_string());
    }

    if chip_name.starts_with("mspm0l134") {
        cfgs.push("mspm0l134x".to_string());
    }

    if chip_name.starts_with("mspm0l222") {
        cfgs.push("mspm0l222x".to_string());
    }

    cfgs
}

/// Interrupt groups use a weakly linked symbols and #[linkage = "extern_weak"] is nightly we need to
/// do some linker magic to create weak linkage.
fn interrupt_group_linker_magic() {
    let mut file = String::new();

    for group in METADATA.interrupt_groups {
        for interrupt in group.interrupts.iter() {
            let name = interrupt.name;

            writeln!(&mut file, "PROVIDE({name} = DefaultHandler);").unwrap();
        }
    }

    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_file = out_dir.join("interrupt_group.x");
    fs::write(&out_file, file).unwrap();
}

fn generate_groups() -> TokenStream {
    let group_vectors = METADATA.interrupt_groups.iter().map(|group| {
        let vectors = group.interrupts.iter().map(|interrupt| {
            let fn_name = Ident::new(interrupt.name, Span::call_site());

            quote! {
                pub(crate) fn #fn_name();
            }
        });

        quote! { #(#vectors)* }
    });

    let groups = METADATA.interrupt_groups.iter().map(|group| {
        let interrupt_group_name = Ident::new(group.name, Span::call_site());
        let group_enum = Ident::new(&format!("Group{}", &group.name[5..]), Span::call_site());
        let group_number = Literal::u32_unsuffixed(group.number);

        let matches = group.interrupts.iter().map(|interrupt| {
            let variant = Ident::new(&interrupt.name, Span::call_site());

            quote! {
                #group_enum::#variant => unsafe { group_vectors::#variant() },
            }
        });

        quote! {
            #[cfg(feature = "rt")]
            #[crate::pac::interrupt]
            fn #interrupt_group_name() {
                use crate::pac::#group_enum;

                let group = crate::pac::CPUSS.int_group(#group_number);
                let stat = group.iidx().read().stat();

                // check for spurious interrupts
                if stat == crate::pac::cpuss::vals::Iidx::NO_INTR {
                    return;
                }

                // MUST subtract by 1 because NO_INTR offsets IIDX values.
                let iidx = stat.to_bits() - 1;

                let Ok(group) = #group_enum::try_from(iidx as u8) else {
                    return;
                };

                match group {
                    #(#matches)*
                }
            }
        }
    });

    quote! {
        #(#groups)*

        #[cfg(feature = "rt")]
        mod group_vectors {
            unsafe extern "Rust" {
                #(#group_vectors)*
            }
        }
    }
}

fn generate_dma_channel_count() -> TokenStream {
    let count = METADATA.dma_channels.len();

    quote! { pub const DMA_CHANNELS: usize = #count; }
}

fn generate_adc_constants(cfgs: &mut CfgSet) -> TokenStream {
    let vrsel = METADATA.adc_vrsel;
    let memctl = METADATA.adc_memctl;

    cfgs.declare("adc_neg_vref");
    match vrsel {
        3 => (),
        5 => cfgs.enable("adc_neg_vref"),
        _ => panic!("Unsupported ADC VRSEL value: {vrsel}"),
    }
    quote! {
        pub const ADC_VRSEL: u8 = #vrsel;
        pub const ADC_MEMCTL: u8 = #memctl;
    }
}

#[derive(Debug, Clone)]
struct Singleton {
    name: String,

    /// `#[cfg]` guard which enables this singleton instance to be obtained.
    cfg: Option<TokenStream>,
}

impl PartialEq for Singleton {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Singleton {}

impl PartialOrd for Singleton {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Singleton {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

fn get_singletons(cfgs: &mut common::CfgSet) -> Vec<Singleton> {
    let mut singletons = Vec::<Singleton>::new();

    for peripheral in METADATA.peripherals {
        // Some peripherals do not generate a singleton, but generate a singleton for each pin.
        let skip_peripheral_singleton = match peripheral.kind {
            "gpio" => {
                // Also enable ports that are present.
                match peripheral.name {
                    "GPIOB" => cfgs.enable("gpio_pb"),
                    "GPIOC" => cfgs.enable("gpio_pc"),
                    _ => (),
                }

                true
            }

            // Each channel gets a singleton, handled separately.
            "dma" => true,

            // These peripherals do not exist as singletons, and have no signals but are managed
            // by the HAL.
            "iomux" | "cpuss" => true,

            // Unicomm instances get their own singletons, but we need to enable a cfg for unicomm drivers.
            "unicomm" => {
                cfgs.enable("unicomm");
                false
            }

            // TODO: Remove after TIMB is fixed
            "tim" if peripheral.name.starts_with("TIMB") => true,

            _ => false,
        };

        if !skip_peripheral_singleton {
            singletons.push(Singleton {
                name: peripheral.name.to_string(),
                cfg: None,
            });
        }

        // Generate each GPIO pin singleton
        if peripheral.name.starts_with("GPIO") {
            for pin in peripheral.pins {
                let singleton = make_valid_identifier(&pin.signal);
                singletons.push(singleton);
            }
        }
    }

    // DMA channels get their own singletons
    for dma_channel in METADATA.dma_channels.iter() {
        singletons.push(Singleton {
            name: format!("DMA_CH{}", dma_channel.number),
            cfg: None,
        });
    }

    singletons.sort_by(|a, b| a.name.cmp(&b.name));
    singletons
}

fn make_valid_identifier(s: &str) -> Singleton {
    let name = s.replace('+', "_P").replace("-", "_N");

    Singleton { name, cfg: None }
}

fn generate_pincm_mapping() -> TokenStream {
    let pincms = METADATA.pins.iter().map(|mapping| {
        let port_letter = mapping.pin.strip_prefix("P").unwrap();
        let port_base = (port_letter.chars().next().unwrap() as u8 - b'A') * 32;
        // This assumes all ports are single letter length.
        // This is fine unless TI releases a part with 833+ GPIO pins.
        let pin_number = mapping.pin[2..].parse::<u8>().unwrap();

        let num = port_base + pin_number;

        // But subtract 1 since pincm indices start from 0, not 1.
        let pincm = Literal::u8_unsuffixed(mapping.pincm - 1);
        quote! {
            #num => #pincm
        }
    });

    quote! {
        #[doc = "Get the mapping from GPIO pin port to IOMUX PINCM index. This is required since the mapping from IO to PINCM index is not consistent across parts."]
        pub(crate) fn gpio_pincm(pin_port: u8) -> u8 {
            match pin_port {
                #(#pincms),*,
                _ => unreachable!(),
            }
        }
    }
}

fn generate_pin() -> TokenStream {
    let pin_impls = METADATA.pins.iter().map(|pin| {
        let name = Ident::new(&pin.pin, Span::call_site());
        let port_letter = pin.pin.strip_prefix("P").unwrap();
        let port_letter = port_letter.chars().next().unwrap();
        let pin_number = Literal::u8_unsuffixed(pin.pin[2..].parse::<u8>().unwrap());

        let port = Ident::new(&format!("Port{}", port_letter), Span::call_site());

        // TODO: Feature gate pins that can be used as NRST

        quote! {
            impl_pin!(#name, crate::gpio::Port::#port, #pin_number);
        }
    });

    quote! {
        #(#pin_impls)*
    }
}

fn time_driver(singletons: &mut Vec<Singleton>, cfgs: &mut CfgSet) {
    // Timer features
    for (timer, _) in TIMERS.iter() {
        let name = timer.to_lowercase();
        cfgs.declare(&format!("time_driver_{}", name));
    }

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

    // Verify the selected timer is available
    let selected_timer = match time_driver.as_ref().map(|x| x.as_ref()) {
        None => "",
        // TODO: Fix TIMB0
        // Some("timb0") => "TIMB0",
        Some("timg0") => "TIMG0",
        Some("timg1") => "TIMG1",
        Some("timg2") => "TIMG2",
        Some("timg3") => "TIMG3",
        Some("timg4") => "TIMG4",
        Some("timg5") => "TIMG5",
        Some("timg6") => "TIMG6",
        Some("timg7") => "TIMG7",
        Some("timg8") => "TIMG8",
        Some("timg9") => "TIMG9",
        Some("timg10") => "TIMG10",
        Some("timg11") => "TIMG11",
        Some("timg14") => "TIMG14",
        Some("tima0") => "TIMA0",
        Some("tima1") => "TIMA1",
        Some("any") => {
            // Order of timer candidates:
            // 1. Basic timers
            // 2. 16-bit, 2 channel
            // 3. 16-bit, 2 channel with shadow registers
            // 4. 16-bit, 4 channel
            // 5. 16-bit with QEI
            // 6. Advanced timers
            //
            // TODO: 32-bit timers are not considered yet
            [
                // basic timers. No PWM pins
                // "TIMB0", // 16-bit, 2 channel
                "TIMG0", "TIMG1", "TIMG2", "TIMG3", // 16-bit, 2 channel with shadow registers
                "TIMG4", "TIMG5", "TIMG6", "TIMG7",  // 16-bit, 4 channel
                "TIMG14", // 16-bit with QEI
                "TIMG8", "TIMG9", "TIMG10", "TIMG11", // Advanced timers
                "TIMA0", "TIMA1",
            ]
            .iter()
            .find(|tim| singletons.iter().any(|s| s.name == **tim))
            .expect("Could not find any timer")
        }
        _ => panic!("unknown time_driver {:?}", time_driver),
    };

    if !selected_timer.is_empty() {
        cfgs.enable(format!("time_driver_{}", selected_timer.to_lowercase()));
    }

    // Apply cfgs to each timer and it's pins
    for singleton in singletons.iter_mut() {
        if singleton.name.starts_with("TIM") {
            // Remove suffixes for pin singletons.
            let name = if singleton.name.contains("_CCP") {
                singleton.name.split_once("_CCP").unwrap().0
            } else if singleton.name.contains("_FAULT") {
                singleton.name.split_once("_FAULT").unwrap().0
            } else if singleton.name.contains("_IDX") {
                singleton.name.split_once("_IDX").unwrap().0
            } else {
                &singleton.name
            };

            let feature = format!("time-driver-{}", name.to_lowercase());

            if singleton.name.contains(selected_timer) {
                singleton.cfg = Some(quote! { #[cfg(not(all(feature = "time-driver-any", feature = #feature)))] });
            } else {
                singleton.cfg = Some(quote! { #[cfg(not(feature = #feature))] });
            }
        }
    }
}

fn pin_features(singletons: &mut Vec<Singleton>) {
    let sysctl = METADATA
        .peripherals
        .iter()
        .find(|p| p.name == "SYSCTL")
        .expect("no SYSCTL peripheral");

    // Some packages make NRST share a physical pin with a GPIO.
    if let Some(pin) = sysctl.pins.iter().find(|p| p.signal == "NRST" && p.pin != "NRST") {
        let pin = singletons
            .iter_mut()
            .find(|s| s.name == pin.pin)
            .expect("Could not find NRST pin to cfg gate");

        pin.cfg = Some(quote! { #[cfg(feature = "nrst-pin-as-gpio")] });
    }

    let debugss = METADATA
        .peripherals
        .iter()
        .find(|p| p.name == "DEBUGSS")
        .expect("Could not find DEBUGSS peripheral");

    for pin in debugss.pins.iter() {
        let pin = singletons
            .iter_mut()
            .find(|s| s.name == pin.pin)
            .expect("Could not find SWD pin to cfg gate");

        pin.cfg = Some(quote! { #[cfg(feature = "swd-pins-as-gpio")] });
    }
}

fn generate_singletons(singletons: &[Singleton]) -> TokenStream {
    let singletons_peripherals_struct = singletons
        .iter()
        .map(|s| {
            let cfg = s.cfg.clone().unwrap_or_default();

            let ident = format_ident!("{}", s.name);

            quote! {
                #cfg
                #ident
            }
        })
        .collect::<Vec<_>>();

    let singletons_peripherals_def = singletons
        .iter()
        .map(|s| {
            let ident = format_ident!("{}", s.name);

            quote! {
                #ident
            }
        })
        .collect::<Vec<_>>();

    quote! {
        embassy_hal_internal::peripherals_definition!(#(#singletons_peripherals_def),*);
        embassy_hal_internal::peripherals_struct!(#(#singletons_peripherals_struct),*);
    }
}

fn generate_timers() -> TokenStream {
    // Generate timers
    let timer_impls = METADATA
        .peripherals
        .iter()
        .filter(|p| p.name.starts_with("TIM"))
        // TODO: Fix TIMB when used at time driver.
        .filter(|p| !p.name.starts_with("TIMB"))
        .map(|peripheral| {
            let name = Ident::new(&peripheral.name, Span::call_site());
            let timers = &*TIMERS;

            let timer = timers.get(peripheral.name).expect("Timer does not exist");
            assert!(timer.bits == 16 || timer.bits == 32);
            let bits = if timer.bits == 16 {
                quote! { Bits16 }
            } else {
                quote! { Bits32 }
            };

            quote! {
                impl_timer!(#name, #bits);
            }
        });

    quote! {
        #(#timer_impls)*
    }
}

fn generate_interrupts() -> TokenStream {
    // Generate interrupt module
    let interrupts: Vec<Ident> = METADATA
        .interrupts
        .iter()
        .map(|interrupt| Ident::new(interrupt.name, Span::call_site()))
        .collect();

    let group_interrupt_enables = METADATA
        .interrupts
        .iter()
        .filter(|interrupt| interrupt.name.contains("GROUP"))
        .map(|interrupt| {
            let name = Ident::new(interrupt.name, Span::call_site());

            quote! {
                crate::interrupt::typelevel::#name::enable();
            }
        });

    // Generate interrupt enables for groups
    quote! {
        embassy_hal_internal::interrupt_mod! {
            #(#interrupts),*
        }

        pub fn enable_group_interrupts(_cs: critical_section::CriticalSection) {
            use crate::interrupt::typelevel::Interrupt;

            // This is empty for C1105/6
            #[allow(unused_unsafe)]
            unsafe {
                #(#group_interrupt_enables)*
            }
        }
    }
}

fn generate_peripheral_instances() -> TokenStream {
    let mut impls = Vec::<TokenStream>::new();

    for peripheral in METADATA.peripherals {
        let peri = format_ident!("{}", peripheral.name);
        let fifo_size = peripheral.sys_fentries;

        let tokens = match peripheral.kind {
            "uart" => Some(quote! { impl_uart_instance!(#peri); }),
            "i2c" => Some(quote! { impl_i2c_instance!(#peri, #fifo_size); }),
            "wwdt" => Some(quote! { impl_wwdt_instance!(#peri); }),
            "adc" => Some(quote! { impl_adc_instance!(#peri); }),
            "mathacl" => Some(quote! { impl_mathacl_instance!(#peri); }),
            _ => None,
        };

        if let Some(tokens) = tokens {
            impls.push(tokens);
        }
    }

    // DMA channels
    for dma_channel in METADATA.dma_channels.iter() {
        let peri = format_ident!("DMA_CH{}", dma_channel.number);
        let num = dma_channel.number;

        if dma_channel.full {
            impls.push(quote! { impl_full_dma_channel!(#peri, #num); });
        } else {
            impls.push(quote! { impl_dma_channel!(#peri, #num); });
        }
    }

    quote! {
        #(#impls)*
    }
}

fn generate_pin_trait_impls() -> TokenStream {
    let mut impls = Vec::<TokenStream>::new();

    for peripheral in METADATA.peripherals {
        for pin in peripheral.pins {
            let key = (peripheral.kind, pin.signal);

            let pin_name = format_ident!("{}", pin.pin);
            let peri = format_ident!("{}", peripheral.name);
            let pf = pin.pf;

            // Will be filled in when uart implementation is finished
            let _ = pin_name;
            let _ = peri;
            let _ = pf;

            let tokens = match key {
                ("uart", "TX") => Some(quote! { impl_uart_tx_pin!(#peri, #pin_name, #pf); }),
                ("uart", "RX") => Some(quote! { impl_uart_rx_pin!(#peri, #pin_name, #pf); }),
                ("uart", "CTS") => Some(quote! { impl_uart_cts_pin!(#peri, #pin_name, #pf); }),
                ("uart", "RTS") => Some(quote! { impl_uart_rts_pin!(#peri, #pin_name, #pf); }),
                ("i2c", "SDA") => Some(quote! { impl_i2c_sda_pin!(#peri, #pin_name, #pf); }),
                ("i2c", "SCL") => Some(quote! { impl_i2c_scl_pin!(#peri, #pin_name, #pf); }),
                ("adc", s) => {
                    let signal = s.parse::<u8>().unwrap();
                    Some(quote! { impl_adc_pin!(#peri, #pin_name, #signal); })
                }

                _ => None,
            };

            if let Some(tokens) = tokens {
                impls.push(tokens);
            }
        }
    }

    quote! {
        #(#impls)*
    }
}

fn select_gpio_features(cfgs: &mut CfgSet) {
    cfgs.declare_all(&[
        "gpioa_interrupt",
        "gpioa_group",
        "gpiob_interrupt",
        "gpiob_group",
        "gpioc_group",
    ]);

    for interrupt in METADATA.interrupts.iter() {
        match interrupt.name {
            "GPIOA" => cfgs.enable("gpioa_interrupt"),
            "GPIOB" => cfgs.enable("gpiob_interrupt"),
            _ => (),
        }
    }

    for group in METADATA.interrupt_groups.iter() {
        for interrupt in group.interrupts {
            match interrupt.name {
                "GPIOA" => cfgs.enable("gpioa_group"),
                "GPIOB" => cfgs.enable("gpiob_group"),
                "GPIOC" => cfgs.enable("gpioc_group"),
                _ => (),
            }
        }
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

#[allow(dead_code)]
struct TimerDesc {
    bits: u8,
    /// Is there an 8-bit prescaler
    prescaler: bool,
    /// Is there a repeat counter
    repeat_counter: bool,
    ccp_channels_internal: u8,
    ccp_channels_external: u8,
    external_pwm_channels: u8,
    phase_load: bool,
    shadow_load: bool,
    shadow_ccs: bool,
    deadband: bool,
    fault_handler: bool,
    qei_hall: bool,
}

/// Description of all timer instances.
const TIMERS: LazyLock<HashMap<String, TimerDesc>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(
        "TIMB0".into(),
        TimerDesc {
            bits: 16,
            prescaler: true,
            repeat_counter: false,
            ccp_channels_internal: 2,
            ccp_channels_external: 2,
            external_pwm_channels: 0,
            phase_load: false,
            shadow_load: false,
            shadow_ccs: false,
            deadband: false,
            fault_handler: false,
            qei_hall: false,
        },
    );

    map.insert(
        "TIMG0".into(),
        TimerDesc {
            bits: 16,
            prescaler: true,
            repeat_counter: false,
            ccp_channels_internal: 2,
            ccp_channels_external: 2,
            external_pwm_channels: 2,
            phase_load: false,
            shadow_load: false,
            shadow_ccs: false,
            deadband: false,
            fault_handler: false,
            qei_hall: false,
        },
    );

    map.insert(
        "TIMG1".into(),
        TimerDesc {
            bits: 16,
            prescaler: true,
            repeat_counter: false,
            ccp_channels_internal: 2,
            ccp_channels_external: 2,
            external_pwm_channels: 2,
            phase_load: false,
            shadow_load: false,
            shadow_ccs: false,
            deadband: false,
            fault_handler: false,
            qei_hall: false,
        },
    );

    map.insert(
        "TIMG2".into(),
        TimerDesc {
            bits: 16,
            prescaler: true,
            repeat_counter: false,
            ccp_channels_internal: 2,
            ccp_channels_external: 2,
            external_pwm_channels: 2,
            phase_load: false,
            shadow_load: false,
            shadow_ccs: false,
            deadband: false,
            fault_handler: false,
            qei_hall: false,
        },
    );

    map.insert(
        "TIMG3".into(),
        TimerDesc {
            bits: 16,
            prescaler: true,
            repeat_counter: false,
            ccp_channels_internal: 2,
            ccp_channels_external: 2,
            external_pwm_channels: 2,
            phase_load: false,
            shadow_load: false,
            shadow_ccs: false,
            deadband: false,
            fault_handler: false,
            qei_hall: false,
        },
    );

    map.insert(
        "TIMG4".into(),
        TimerDesc {
            bits: 16,
            prescaler: true,
            repeat_counter: false,
            ccp_channels_internal: 2,
            ccp_channels_external: 2,
            external_pwm_channels: 2,
            phase_load: false,
            shadow_load: true,
            shadow_ccs: true,
            deadband: false,
            fault_handler: false,
            qei_hall: false,
        },
    );

    map.insert(
        "TIMG5".into(),
        TimerDesc {
            bits: 16,
            prescaler: true,
            repeat_counter: false,
            ccp_channels_internal: 2,
            ccp_channels_external: 2,
            external_pwm_channels: 2,
            phase_load: false,
            shadow_load: true,
            shadow_ccs: true,
            deadband: false,
            fault_handler: false,
            qei_hall: false,
        },
    );

    map.insert(
        "TIMG6".into(),
        TimerDesc {
            bits: 16,
            prescaler: true,
            repeat_counter: false,
            ccp_channels_internal: 2,
            ccp_channels_external: 2,
            external_pwm_channels: 2,
            phase_load: false,
            shadow_load: true,
            shadow_ccs: true,
            deadband: false,
            fault_handler: false,
            qei_hall: false,
        },
    );

    map.insert(
        "TIMG7".into(),
        TimerDesc {
            bits: 16,
            prescaler: true,
            repeat_counter: false,
            ccp_channels_internal: 2,
            ccp_channels_external: 2,
            external_pwm_channels: 2,
            phase_load: false,
            shadow_load: true,
            shadow_ccs: true,
            deadband: false,
            fault_handler: false,
            qei_hall: false,
        },
    );

    map.insert(
        "TIMG8".into(),
        TimerDesc {
            bits: 16,
            prescaler: true,
            repeat_counter: false,
            ccp_channels_internal: 2,
            ccp_channels_external: 2,
            external_pwm_channels: 2,
            phase_load: false,
            shadow_load: false,
            shadow_ccs: false,
            deadband: false,
            fault_handler: false,
            qei_hall: true,
        },
    );

    map.insert(
        "TIMG9".into(),
        TimerDesc {
            bits: 16,
            prescaler: true,
            repeat_counter: false,
            ccp_channels_internal: 2,
            ccp_channels_external: 2,
            external_pwm_channels: 2,
            phase_load: false,
            shadow_load: false,
            shadow_ccs: false,
            deadband: false,
            fault_handler: false,
            qei_hall: true,
        },
    );

    map.insert(
        "TIMG10".into(),
        TimerDesc {
            bits: 16,
            prescaler: true,
            repeat_counter: false,
            ccp_channels_internal: 2,
            ccp_channels_external: 2,
            external_pwm_channels: 2,
            phase_load: false,
            shadow_load: false,
            shadow_ccs: false,
            deadband: false,
            fault_handler: false,
            qei_hall: true,
        },
    );

    map.insert(
        "TIMG11".into(),
        TimerDesc {
            bits: 16,
            prescaler: true,
            repeat_counter: false,
            ccp_channels_internal: 2,
            ccp_channels_external: 2,
            external_pwm_channels: 2,
            phase_load: false,
            shadow_load: false,
            shadow_ccs: false,
            deadband: false,
            fault_handler: false,
            qei_hall: true,
        },
    );

    map.insert(
        "TIMG12".into(),
        TimerDesc {
            bits: 32,
            prescaler: false,
            repeat_counter: false,
            ccp_channels_internal: 2,
            ccp_channels_external: 2,
            external_pwm_channels: 2,
            phase_load: false,
            shadow_load: false,
            shadow_ccs: true,
            deadband: false,
            fault_handler: false,
            qei_hall: false,
        },
    );

    map.insert(
        "TIMG13".into(),
        TimerDesc {
            bits: 32,
            prescaler: false,
            repeat_counter: false,
            ccp_channels_internal: 2,
            ccp_channels_external: 2,
            external_pwm_channels: 2,
            phase_load: false,
            shadow_load: false,
            shadow_ccs: true,
            deadband: false,
            fault_handler: false,
            qei_hall: false,
        },
    );

    map.insert(
        "TIMG14".into(),
        TimerDesc {
            bits: 16,
            prescaler: true,
            repeat_counter: false,
            ccp_channels_internal: 4,
            ccp_channels_external: 4,
            external_pwm_channels: 4,
            phase_load: false,
            shadow_load: false,
            shadow_ccs: false,
            deadband: false,
            fault_handler: false,
            qei_hall: false,
        },
    );

    map.insert(
        "TIMA0".into(),
        TimerDesc {
            bits: 16,
            prescaler: true,
            repeat_counter: true,
            ccp_channels_internal: 4,
            ccp_channels_external: 2,
            external_pwm_channels: 8,
            phase_load: true,
            shadow_load: true,
            shadow_ccs: true,
            deadband: true,
            fault_handler: true,
            qei_hall: false,
        },
    );

    map.insert(
        "TIMA1".into(),
        TimerDesc {
            bits: 16,
            prescaler: true,
            repeat_counter: true,
            ccp_channels_internal: 2,
            ccp_channels_external: 2,
            external_pwm_channels: 4,
            phase_load: true,
            shadow_load: true,
            shadow_ccs: true,
            deadband: true,
            fault_handler: true,
            qei_hall: false,
        },
    );

    map
});

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
