use chiptool::generate::CommonModule;
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

#[derive(Debug, Eq, PartialEq, Clone)]
struct Metadata<'a> {
    name: &'a str,
    family: &'a str,
    line: &'a str,
    memory: &'a [MemoryRegion],
    peripherals: &'a [Peripheral],
    interrupts: &'a [Interrupt],
    dma_channels: &'a [DmaChannel],
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
    let mut dbgmcu: Option<ir::IR> = core.peripherals.iter().find_map(|p| {
        if p.name == "DBGMCU" {
            p.registers.as_ref().map(|bi| {
                let dbgmcu_reg_path = options
                    .data_dir
                    .join("registers")
                    .join(&format!("{}_{}.yaml", bi.kind, bi.version));
                serde_yaml::from_reader(File::open(dbgmcu_reg_path).unwrap()).unwrap()
            })
        } else {
            None
        }
    });

    let mut peripheral_versions: BTreeMap<String, String> = BTreeMap::new();
    let mut pin_table: Vec<Vec<String>> = Vec::new();
    let mut interrupt_table: Vec<Vec<String>> = Vec::new();
    let mut peripherals_table: Vec<Vec<String>> = Vec::new();
    let mut dma_channels_table: Vec<Vec<String>> = Vec::new();
    let mut peripheral_counts: BTreeMap<String, u8> = BTreeMap::new();
    let mut dma_channel_counts: BTreeMap<String, u8> = BTreeMap::new();
    let mut dbgmcu_table: Vec<Vec<String>> = Vec::new();

    let gpio_base = core
        .peripherals
        .iter()
        .find(|p| p.name == "GPIOA")
        .unwrap()
        .address as u32;
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

    for p in &core.peripherals {
        let captures = number_suffix_re.captures(&p.name).unwrap();
        let root_peri_name = captures.get(1).unwrap().as_str().to_string();
        peripheral_counts.insert(
            root_peri_name.clone(),
            peripheral_counts.get(&root_peri_name).map_or(1, |v| v + 1),
        );
        let mut ir_peri = ir::Peripheral {
            name: p.name.clone(),
            array: None,
            base_address: p.address,
            block: None,
            description: None,
            interrupts: HashMap::new(),
        };

        if let Some(bi) = &p.registers {
            peripheral_counts.insert(
                bi.kind.clone(),
                peripheral_counts.get(&bi.kind).map_or(1, |v| v + 1),
            );

            for irq in &p.interrupts {
                let mut row = Vec::new();
                row.push(p.name.clone());
                row.push(bi.kind.clone());
                row.push(bi.block.clone());
                row.push(irq.signal.clone());
                row.push(irq.interrupt.to_ascii_uppercase());
                interrupt_table.push(row)
            }

            let mut peripheral_row = Vec::new();
            peripheral_row.push(bi.kind.clone());
            peripheral_row.push(p.name.clone());
            peripherals_table.push(peripheral_row);

            if let Some(old_version) =
                peripheral_versions.insert(bi.kind.clone(), bi.version.clone())
            {
                if old_version != bi.version {
                    panic!(
                        "Peripheral {} has multiple versions: {} and {}",
                        bi.kind, old_version, bi.version
                    );
                }
            }
            ir_peri.block = Some(format!("{}::{}", bi.kind, bi.block));

            match bi.kind.as_str() {
                "gpio" => {
                    let port_letter = p.name.chars().skip(4).next().unwrap();
                    assert_eq!(0, (p.address as u32 - gpio_base) % gpio_stride);
                    let port_num = (p.address as u32 - gpio_base) / gpio_stride;

                    for pin_num in 0u32..16 {
                        let pin_name = format!("P{}{}", port_letter, pin_num);
                        pin_table.push(vec![
                            pin_name.clone(),
                            p.name.clone(),
                            port_num.to_string(),
                            pin_num.to_string(),
                            format!("EXTI{}", pin_num),
                        ]);
                    }
                }
                _ => {}
            }
        }

        dev.peripherals.push(ir_peri);
    }

    for ch in &core.dma_channels {
        let mut row = Vec::new();
        let dma_peri = core.peripherals.iter().find(|p| p.name == ch.dma).unwrap();
        let bi = dma_peri.registers.as_ref().unwrap();

        row.push(ch.name.clone());
        row.push(ch.dma.clone());
        row.push(bi.kind.clone());
        row.push(ch.channel.to_string());
        if let Some(dmamux) = &ch.dmamux {
            let dmamux_channel = ch.dmamux_channel.unwrap();
            row.push(format!(
                "{{dmamux: {}, dmamux_channel: {}}}",
                dmamux, dmamux_channel
            ));
        } else {
            row.push("{}".to_string());
        }

        dma_channels_table.push(row);

        let dma_peri_name = ch.dma.clone();
        dma_channel_counts.insert(
            dma_peri_name.clone(),
            dma_channel_counts.get(&dma_peri_name).map_or(1, |v| v + 1),
        );
    }

    for irq in &core.interrupts {
        dev.interrupts.push(ir::Interrupt {
            name: irq.name.clone(),
            description: None,
            value: irq.number,
        });

        let name = irq.name.to_ascii_uppercase();

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

    for irq in &core.interrupts {
        write!(&mut device_x, "PROVIDE({} = DefaultHandler);\n", irq.name).unwrap();
    }

    // ==============================
    // generate mod.rs

    let mut data = String::new();

    write!(&mut data, "#[cfg(feature=\"metadata\")] pub mod metadata;").unwrap();
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
    make_table(&mut data, "dma_channels", &dma_channels_table);
    make_table(&mut data, "dbgmcu", &dbgmcu_table);
    make_peripheral_counts(&mut data, &peripheral_counts);
    make_dma_channel_counts(&mut data, &dma_channel_counts);

    let mut file = File::create(chip_dir.join("mod.rs")).unwrap();
    file.write_all(data.as_bytes()).unwrap();

    // ==============================
    // generate metadata.rs

    let metadata = Metadata {
        name: &chip.name,
        family: &chip.family,
        line: &chip.line,
        memory: &chip.memory,
        peripherals: &core.peripherals,
        interrupts: &core.interrupts,
        dma_channels: &core.dma_channels,
    };
    let metadata = format!("{:#?}", metadata);
    let metadata = metadata.replace("[\n", "&[\n");
    let metadata = metadata.replace("[],\n", "&[],\n");

    let mut data = String::new();

    write!(
        &mut data,
        "
            include!(\"../../metadata.rs\");
            use MemoryRegionKind::*;
            pub const METADATA: Metadata = {};    
        ",
        metadata
    )
    .unwrap();

    let mut file = File::create(chip_dir.join("metadata.rs")).unwrap();
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
        .join(&format!("{}.json", name));
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
        println!("Generating {}...", chip_name);

        let mut chip = load_chip(&options, chip_name);

        // Cleanup
        for core in &mut chip.cores {
            for irq in &mut core.interrupts {
                irq.name = irq.name.to_ascii_uppercase();
            }
            for p in &mut core.peripherals {
                for irq in &mut p.interrupts {
                    irq.interrupt = irq.interrupt.to_ascii_uppercase();
                }
            }
        }

        // Generate
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

    // Generate src/metadata.rs
    fs::write(
        options.out_dir.join("src").join("metadata.rs"),
        include_bytes!("assets/metadata.rs"),
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

    let flash = chip.memory.iter().find(|r| r.name == "BANK_1").unwrap();
    let ram = chip.memory.iter().find(|r| r.name == "SRAM").unwrap();

    write!(memory_x, "MEMORY\n{{\n").unwrap();
    write!(
        memory_x,
        "    FLASH : ORIGIN = 0x{:x}, LENGTH = {}\n",
        flash.address, flash.size,
    )
    .unwrap();
    write!(
        memory_x,
        "    RAM : ORIGIN = 0x{:x}, LENGTH = {}\n",
        ram.address, ram.size,
    )
    .unwrap();
    write!(memory_x, "}}").unwrap();

    fs::create_dir_all(out_dir.join("memory_x")).unwrap();
    let mut file = File::create(out_dir.join("memory_x").join("memory.x")).unwrap();
    file.write_all(memory_x.as_bytes()).unwrap();
}
