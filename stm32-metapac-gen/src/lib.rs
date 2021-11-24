use chiptool::generate::CommonModule;
use chiptool::ir::IR;
use proc_macro2::TokenStream;
use regex::Regex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use chiptool::util::ToSanitizedSnakeCase;
use chiptool::{generate, ir, transform};

mod data;
use data::*;

fn find_reg<'c>(rcc: &'c ir::IR, reg_regex: &str, field_name: &str) -> Option<(&'c str, &'c str)> {
    let reg_regex = Regex::new(reg_regex).unwrap();

    for (name, fieldset) in &rcc.fieldsets {
        // Workaround for some families that prefix register aliases with C1_, which does
        // not help matching for clock name.
        if !name.starts_with("C1") && !name.starts_with("C2") && reg_regex.is_match(name) {
            for field in &fieldset.fields {
                if field_name == field.name {
                    return Some((name.as_str(), field.name.as_str()));
                }
            }
        }
    }

    None
}

fn make_peripheral_counts(out: &mut String, data: &BTreeMap<String, u8>) {
    write!(
        out,
        "#[macro_export]
macro_rules! peripheral_count {{
    "
    )
    .unwrap();
    for (name, count) in data {
        write!(out, "({}) => ({});\n", name, count,).unwrap();
    }
    write!(out, " }}\n").unwrap();
}

fn make_dma_channel_counts(out: &mut String, data: &BTreeMap<String, u8>) {
    if data.len() == 0 {
        return;
    }
    write!(
        out,
        "#[macro_export]
macro_rules! dma_channels_count {{
    "
    )
    .unwrap();
    for (name, count) in data {
        write!(out, "({}) => ({});\n", name, count,).unwrap();
    }
    write!(out, " }}\n").unwrap();
}

fn make_table(out: &mut String, name: &str, data: &Vec<Vec<String>>) {
    write!(
        out,
        "#[macro_export]
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
        write!(out, "        __{}_inner!(({}));\n", name, row.join(",")).unwrap();
    }

    write!(
        out,
        "    }};
}}"
    )
    .unwrap();
}

pub struct Options {
    pub chips: Vec<String>,
    pub out_dir: PathBuf,
    pub data_dir: PathBuf,
}

pub fn gen_chip(
    options: &Options,
    chip_core_name: &str,
    chip: &Chip,
    core: &Core,
    core_index: usize,
    all_peripheral_versions: &mut HashSet<(String, String)>,
) {
    let mut ir = ir::IR::new();

    let mut dev = ir::Device {
        interrupts: Vec::new(),
        peripherals: Vec::new(),
    };

    // Load DBGMCU register for chip
    let mut dbgmcu: Option<ir::IR> = core.peripherals.iter().find_map(|(name, p)| {
        if name == "DBGMCU" {
            p.block.as_ref().map(|block| {
                let bi = BlockInfo::parse(block);
                let dbgmcu_reg_path = options
                    .data_dir
                    .join("registers")
                    .join(&format!("{}_{}.yaml", bi.module, bi.version));
                serde_yaml::from_reader(File::open(dbgmcu_reg_path).unwrap()).unwrap()
            })
        } else {
            None
        }
    });

    // Load RCC register for chip
    let (_, rcc) = core
        .peripherals
        .iter()
        .find(|(name, _)| name == &"RCC")
        .expect("RCC peripheral missing");

    let rcc_block = rcc.block.as_ref().expect("RCC peripheral has no block");
    let bi = BlockInfo::parse(&rcc_block);
    let rcc_reg_path = options
        .data_dir
        .join("registers")
        .join(&format!("{}_{}.yaml", bi.module, bi.version));
    let rcc: IR = serde_yaml::from_reader(File::open(rcc_reg_path).unwrap()).unwrap();

    let mut peripheral_versions: BTreeMap<String, String> = BTreeMap::new();
    let mut pin_table: Vec<Vec<String>> = Vec::new();
    let mut interrupt_table: Vec<Vec<String>> = Vec::new();
    let mut peripherals_table: Vec<Vec<String>> = Vec::new();
    let mut peripheral_pins_table: Vec<Vec<String>> = Vec::new();
    let mut peripheral_rcc_table: Vec<Vec<String>> = Vec::new();
    let mut dma_channels_table: Vec<Vec<String>> = Vec::new();
    let mut peripheral_dma_channels_table: Vec<Vec<String>> = Vec::new();
    let mut peripheral_counts: BTreeMap<String, u8> = BTreeMap::new();
    let mut dma_channel_counts: BTreeMap<String, u8> = BTreeMap::new();
    let mut dbgmcu_table: Vec<Vec<String>> = Vec::new();
    let mut gpio_rcc_table: Vec<Vec<String>> = Vec::new();
    let mut gpio_regs: HashSet<String> = HashSet::new();

    let gpio_base = core.peripherals.get(&"GPIOA".to_string()).unwrap().address as u32;
    let gpio_stride = 0x400;

    let number_suffix_re = Regex::new("^(.*?)[0-9]*$").unwrap();

    if let Some(ref mut reg) = dbgmcu {
        if let Some(ref cr) = reg.fieldsets.get("CR") {
            for field in cr.fields.iter().filter(|e| e.name.contains("DBG")) {
                let mut fn_name = String::new();
                fn_name.push_str("set_");
                fn_name.push_str(&field.name.to_sanitized_snake_case());
                dbgmcu_table.push(vec!["cr".into(), fn_name]);
            }
        }
    }

    for (name, p) in &core.peripherals {
        let captures = number_suffix_re.captures(&name).unwrap();
        let root_peri_name = captures.get(1).unwrap().as_str().to_string();
        peripheral_counts.insert(
            root_peri_name.clone(),
            peripheral_counts.get(&root_peri_name).map_or(1, |v| v + 1),
        );
        let mut ir_peri = ir::Peripheral {
            name: name.clone(),
            array: None,
            base_address: p.address,
            block: None,
            description: None,
            interrupts: HashMap::new(),
        };

        if let Some(block) = &p.block {
            let bi = BlockInfo::parse(block);

            peripheral_counts.insert(
                bi.module.clone(),
                peripheral_counts.get(&bi.module).map_or(1, |v| v + 1),
            );

            for pin in &p.pins {
                let mut row = Vec::new();
                row.push(name.clone());
                row.push(bi.module.clone());
                row.push(bi.block.clone());
                row.push(pin.pin.clone());
                row.push(pin.signal.clone());
                if let Some(ref af) = pin.af {
                    row.push(af.clone());
                }
                peripheral_pins_table.push(row);
            }

            for (signal, irq_name) in &p.interrupts {
                let mut row = Vec::new();
                row.push(name.clone());
                row.push(bi.module.clone());
                row.push(bi.block.clone());
                row.push(signal.clone());
                row.push(irq_name.to_ascii_uppercase());
                interrupt_table.push(row)
            }

            for (request, dma_channels) in &p.dma_channels {
                for channel in dma_channels.iter() {
                    let mut row = Vec::new();
                    row.push(name.clone());
                    row.push(bi.module.clone());
                    row.push(bi.block.clone());
                    row.push(request.clone());
                    row.push(if let Some(channel) = &channel.channel {
                        format!("{{channel: {}}}", channel)
                    } else if let Some(dmamux) = &channel.dmamux {
                        format!("{{dmamux: {}}}", dmamux)
                    } else {
                        unreachable!();
                    });

                    row.push(if let Some(request) = channel.request {
                        request.to_string()
                    } else {
                        "()".to_string()
                    });

                    if peripheral_dma_channels_table
                        .iter()
                        .find(|a| a[..a.len() - 1] == row[..row.len() - 1])
                        .is_none()
                    {
                        peripheral_dma_channels_table.push(row);
                    }
                }
            }

            let mut peripheral_row = Vec::new();
            peripheral_row.push(bi.module.clone());
            peripheral_row.push(name.clone());
            peripherals_table.push(peripheral_row);

            if let Some(old_version) =
                peripheral_versions.insert(bi.module.clone(), bi.version.clone())
            {
                if old_version != bi.version {
                    panic!(
                        "Peripheral {} has multiple versions: {} and {}",
                        bi.module, old_version, bi.version
                    );
                }
            }
            ir_peri.block = Some(format!("{}::{}", bi.module, bi.block));

            match bi.module.as_str() {
                "gpio" => {
                    let port_letter = name.chars().skip(4).next().unwrap();
                    assert_eq!(0, (p.address as u32 - gpio_base) % gpio_stride);
                    let port_num = (p.address as u32 - gpio_base) / gpio_stride;

                    for pin_num in 0..16 {
                        let pin_name = format!("P{}{}", port_letter, pin_num);
                        pin_table.push(vec![
                            pin_name.clone(),
                            name.clone(),
                            port_num.to_string(),
                            pin_num.to_string(),
                            format!("EXTI{}", pin_num),
                        ]);
                    }
                }
                _ => {}
            }

            // Workaround for clock registers being split on some chip families. Assume fields are
            // named after peripheral and look for first field matching and use that register.
            let mut en = find_reg(&rcc, "^.+ENR\\d*$", &format!("{}EN", name));
            let mut rst = find_reg(&rcc, "^.+RSTR\\d*$", &format!("{}RST", name));

            if en.is_none() && name.ends_with("1") {
                en = find_reg(
                    &rcc,
                    "^.+ENR\\d*$",
                    &format!("{}EN", &name[..name.len() - 1]),
                );
                rst = find_reg(
                    &rcc,
                    "^.+RSTR\\d*$",
                    &format!("{}RST", &name[..name.len() - 1]),
                );
            }

            match (en, rst) {
                (Some((enable_reg, enable_field)), reset_reg_field) => {
                    let clock = match &p.clock {
                        Some(clock) => clock.as_str(),
                        None => {
                            // No clock was specified, derive the clock name from the enable register name.
                            // N.B. STM32G0 has only one APB bus but split ENR registers
                            // (e.g. APBENR1).
                            Regex::new("([A-Z]+\\d*)ENR\\d*")
                                .unwrap()
                                .captures(enable_reg)
                                .unwrap()
                                .get(1)
                                .unwrap()
                                .as_str()
                        }
                    };

                    let clock = if name.starts_with("TIM") {
                        format!("{}_tim", clock.to_ascii_lowercase())
                    } else {
                        clock.to_ascii_lowercase()
                    };

                    let mut row = Vec::with_capacity(6);
                    row.push(name.clone());
                    row.push(clock);
                    row.push(enable_reg.to_ascii_lowercase());

                    if let Some((reset_reg, reset_field)) = reset_reg_field {
                        row.push(reset_reg.to_ascii_lowercase());
                        row.push(format!("set_{}", enable_field.to_ascii_lowercase()));
                        row.push(format!("set_{}", reset_field.to_ascii_lowercase()));
                    } else {
                        row.push(format!("set_{}", enable_field.to_ascii_lowercase()));
                    }

                    if !name.starts_with("GPIO") {
                        peripheral_rcc_table.push(row);
                    } else {
                        gpio_rcc_table.push(row);
                        gpio_regs.insert(enable_reg.to_ascii_lowercase());
                    }
                }
                (None, Some(_)) => {
                    println!("Unable to find enable register for {}", name)
                }
                (None, None) => {
                    println!("Unable to find enable and reset register for {}", name)
                }
            }
        }

        dev.peripherals.push(ir_peri);
    }

    for reg in gpio_regs {
        gpio_rcc_table.push(vec![reg]);
    }

    // We should always find GPIO RCC regs. If not, it means something
    // is broken and GPIO won't work because it's not enabled.
    assert!(!gpio_rcc_table.is_empty());

    for (id, channel_info) in &core.dma_channels {
        let mut row = Vec::new();
        let dma_peri = core.peripherals.get(&channel_info.dma).unwrap();
        let bi = BlockInfo::parse(dma_peri.block.as_ref().unwrap());

        row.push(id.clone());
        row.push(channel_info.dma.clone());
        row.push(bi.module.clone());
        row.push(channel_info.channel.to_string());
        if let Some(dmamux) = &channel_info.dmamux {
            let dmamux_channel = channel_info.dmamux_channel.unwrap();
            row.push(format!(
                "{{dmamux: {}, dmamux_channel: {}}}",
                dmamux, dmamux_channel
            ));
        } else {
            row.push("{}".to_string());
        }

        dma_channels_table.push(row);

        let dma_peri_name = channel_info.dma.clone();
        dma_channel_counts.insert(
            dma_peri_name.clone(),
            dma_channel_counts.get(&dma_peri_name).map_or(1, |v| v + 1),
        );
    }

    for (name, &num) in &core.interrupts {
        dev.interrupts.push(ir::Interrupt {
            name: name.clone(),
            description: None,
            value: num,
        });

        let name = name.to_ascii_uppercase();

        interrupt_table.push(vec![name.clone()]);

        if name.contains("EXTI") {
            interrupt_table.push(vec!["EXTI".to_string(), name.clone()]);
        }
    }

    ir.devices.insert("".to_string(), dev);

    let mut extra = format!(
        "pub fn GPIO(n: usize) -> gpio::Gpio {{
            gpio::Gpio(({} + {}*n) as _)
        }}",
        gpio_base, gpio_stride,
    );

    for (module, version) in &peripheral_versions {
        all_peripheral_versions.insert((module.clone(), version.clone()));
        write!(
            &mut extra,
            "#[path=\"../../peripherals/{}_{}.rs\"] pub mod {};\n",
            module, version, module
        )
        .unwrap();
    }
    write!(
        &mut extra,
        "pub const CORE_INDEX: usize = {};\n",
        core_index
    )
    .unwrap();

    // Cleanups!
    transform::sort::Sort {}.run(&mut ir).unwrap();
    transform::Sanitize {}.run(&mut ir).unwrap();

    // ==============================
    // Setup chip dir

    let chip_dir = options
        .out_dir
        .join("src/chips")
        .join(chip_core_name.to_ascii_lowercase());
    fs::create_dir_all(&chip_dir).unwrap();

    // ==============================
    // generate pac.rs

    let data = generate::render(&ir, &gen_opts()).unwrap().to_string();
    let data = data.replace("] ", "]\n");

    // Remove inner attributes like #![no_std]
    let data = Regex::new("# *! *\\[.*\\]").unwrap().replace_all(&data, "");

    let mut file = File::create(chip_dir.join("pac.rs")).unwrap();
    file.write_all(data.as_bytes()).unwrap();
    file.write_all(extra.as_bytes()).unwrap();

    let mut device_x = String::new();

    for (name, _) in &core.interrupts {
        write!(
            &mut device_x,
            "PROVIDE({} = DefaultHandler);\n",
            name.to_ascii_uppercase()
        )
        .unwrap();
    }

    // ==============================
    // generate mod.rs

    let mut data = String::new();

    write!(&mut data, "#[cfg(feature=\"pac\")] mod pac;").unwrap();
    write!(&mut data, "#[cfg(feature=\"pac\")] pub use pac::*; ").unwrap();

    let peripheral_version_table = peripheral_versions
        .iter()
        .map(|(kind, version)| vec![kind.clone(), version.clone()])
        .collect();

    make_table(&mut data, "pins", &pin_table);
    make_table(&mut data, "interrupts", &interrupt_table);
    make_table(&mut data, "peripherals", &peripherals_table);
    make_table(&mut data, "peripheral_versions", &peripheral_version_table);
    make_table(&mut data, "peripheral_pins", &peripheral_pins_table);
    make_table(
        &mut data,
        "peripheral_dma_channels",
        &peripheral_dma_channels_table,
    );
    make_table(&mut data, "peripheral_rcc", &peripheral_rcc_table);
    make_table(&mut data, "gpio_rcc", &gpio_rcc_table);
    make_table(&mut data, "dma_channels", &dma_channels_table);
    make_table(&mut data, "dbgmcu", &dbgmcu_table);
    make_peripheral_counts(&mut data, &peripheral_counts);
    make_dma_channel_counts(&mut data, &dma_channel_counts);

    let mut file = File::create(chip_dir.join("mod.rs")).unwrap();
    file.write_all(data.as_bytes()).unwrap();

    // ==============================
    // generate device.x

    File::create(chip_dir.join("device.x"))
        .unwrap()
        .write_all(device_x.as_bytes())
        .unwrap();

    // ==============================
    // generate default memory.x
    gen_memory_x(&chip_dir, &chip);
}

fn load_chip(options: &Options, name: &str) -> Chip {
    let chip_path = options
        .data_dir
        .join("chips")
        .join(&format!("{}.yaml", name));
    let chip = fs::read(chip_path).expect(&format!("Could not load chip {}", name));
    serde_yaml::from_slice(&chip).unwrap()
}

fn gen_opts() -> generate::Options {
    generate::Options {
        common_module: CommonModule::External(TokenStream::from_str("crate::common").unwrap()),
    }
}

pub fn gen(options: Options) {
    fs::create_dir_all(options.out_dir.join("src/peripherals")).unwrap();
    fs::create_dir_all(options.out_dir.join("src/chips")).unwrap();

    let mut all_peripheral_versions: HashSet<(String, String)> = HashSet::new();
    let mut chip_core_names: Vec<String> = Vec::new();

    for chip_name in &options.chips {
        let chip = load_chip(&options, chip_name);
        for (core_index, core) in chip.cores.iter().enumerate() {
            let chip_core_name = match chip.cores.len() {
                1 => chip_name.clone(),
                _ => format!("{}-{}", chip_name, core.name),
            };

            chip_core_names.push(chip_core_name.clone());
            gen_chip(
                &options,
                &chip_core_name,
                &chip,
                core,
                core_index,
                &mut all_peripheral_versions,
            )
        }
    }

    for (module, version) in all_peripheral_versions {
        println!("loading {} {}", module, version);

        let regs_path = Path::new(&options.data_dir)
            .join("registers")
            .join(&format!("{}_{}.yaml", module, version));

        let mut ir: ir::IR = serde_yaml::from_reader(File::open(regs_path).unwrap()).unwrap();

        transform::expand_extends::ExpandExtends {}
            .run(&mut ir)
            .unwrap();

        transform::map_names(&mut ir, |k, s| match k {
            transform::NameKind::Block => *s = format!("{}", s),
            transform::NameKind::Fieldset => *s = format!("regs::{}", s),
            transform::NameKind::Enum => *s = format!("vals::{}", s),
            _ => {}
        });

        transform::sort::Sort {}.run(&mut ir).unwrap();
        transform::Sanitize {}.run(&mut ir).unwrap();

        let items = generate::render(&ir, &gen_opts()).unwrap();
        let mut file = File::create(
            options
                .out_dir
                .join("src/peripherals")
                .join(format!("{}_{}.rs", module, version)),
        )
        .unwrap();
        let data = items.to_string().replace("] ", "]\n");

        // Remove inner attributes like #![no_std]
        let re = Regex::new("# *! *\\[.*\\]").unwrap();
        let data = re.replace_all(&data, "");
        file.write_all(data.as_bytes()).unwrap();
    }

    // Generate src/lib_inner.rs
    const PATHS_MARKER: &[u8] = b"// GEN PATHS HERE";
    let librs = include_bytes!("assets/lib_inner.rs");
    let i = bytes_find(librs, PATHS_MARKER).unwrap();
    let mut paths = String::new();

    for name in chip_core_names {
        let x = name.to_ascii_lowercase();
        write!(
            &mut paths,
            "#[cfg_attr(feature=\"{}\", path = \"chips/{}/mod.rs\")]",
            x, x
        )
        .unwrap();
    }
    let mut contents: Vec<u8> = Vec::new();
    contents.extend(&librs[..i]);
    contents.extend(paths.as_bytes());
    contents.extend(&librs[i + PATHS_MARKER.len()..]);
    fs::write(options.out_dir.join("src").join("lib_inner.rs"), &contents).unwrap();

    // Generate src/lib.rs
    const CUT_MARKER: &[u8] = b"// GEN CUT HERE";
    let librs = include_bytes!("../../stm32-metapac/src/lib.rs");
    let i = bytes_find(librs, CUT_MARKER).unwrap();
    let mut contents: Vec<u8> = Vec::new();
    contents.extend(&librs[..i]);
    contents.extend(b"include!(\"lib_inner.rs\");\n");
    fs::write(options.out_dir.join("src").join("lib.rs"), contents).unwrap();

    // Generate src/common.rs
    fs::write(
        options.out_dir.join("src").join("common.rs"),
        generate::COMMON_MODULE,
    )
    .unwrap();

    // Generate Cargo.toml
    const BUILDDEP_BEGIN: &[u8] = b"# BEGIN BUILD DEPENDENCIES";
    const BUILDDEP_END: &[u8] = b"# END BUILD DEPENDENCIES";

    let mut contents = include_bytes!("../../stm32-metapac/Cargo.toml").to_vec();
    let begin = bytes_find(&contents, BUILDDEP_BEGIN).unwrap();
    let end = bytes_find(&contents, BUILDDEP_END).unwrap() + BUILDDEP_END.len();
    contents.drain(begin..end);
    fs::write(options.out_dir.join("Cargo.toml"), contents).unwrap();

    // Generate build.rs
    fs::write(
        options.out_dir.join("build.rs"),
        include_bytes!("assets/build.rs"),
    )
    .unwrap();
}

fn bytes_find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn gen_memory_x(out_dir: &PathBuf, chip: &Chip) {
    let mut memory_x = String::new();

    let flash_bytes = chip
        .flash
        .regions
        .get("BANK_1")
        .unwrap()
        .bytes
        .unwrap_or(chip.flash.bytes);
    let flash_origin = chip.flash.regions.get("BANK_1").unwrap().base;

    let ram_bytes = chip
        .ram
        .regions
        .get("SRAM")
        .unwrap()
        .bytes
        .unwrap_or(chip.ram.bytes);
    let ram_origin = chip.ram.regions.get("SRAM").unwrap().base;

    write!(memory_x, "MEMORY\n{{\n").unwrap();
    write!(
        memory_x,
        "    FLASH : ORIGIN = 0x{:x}, LENGTH = {}\n",
        flash_origin, flash_bytes
    )
    .unwrap();
    write!(
        memory_x,
        "    RAM : ORIGIN = 0x{:x}, LENGTH = {}\n",
        ram_origin, ram_bytes
    )
    .unwrap();
    write!(memory_x, "}}").unwrap();

    fs::create_dir_all(out_dir.join("memory_x")).unwrap();
    let mut file = File::create(out_dir.join("memory_x").join("memory.x")).unwrap();
    file.write_all(memory_x.as_bytes()).unwrap();
}
