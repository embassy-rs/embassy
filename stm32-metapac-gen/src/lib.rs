use regex::Regex;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use chiptool::{generate, ir, transform};
use chiptool::util::ToSanitizedSnakeCase;

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct Chip {
    pub name: String,
    pub family: String,
    pub line: String,
    pub cores: Vec<Core>,
    pub flash: u32,
    pub ram: u32,
    pub packages: Vec<Package>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct Core {
    pub name: String,
    pub peripherals: HashMap<String, Peripheral>,
    pub interrupts: HashMap<String, u32>,
    pub dma_channels: HashMap<String, DmaChannel>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct Package {
    pub name: String,
    pub package: String,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct Peripheral {
    pub address: u32,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub block: Option<String>,
    #[serde(default)]
    pub clock: Option<String>,
    #[serde(default)]
    pub pins: Vec<Pin>,
    #[serde(default)]
    pub dma_channels: HashMap<String, Vec<PeripheralDmaChannel>>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct Pin {
    pub pin: String,
    pub signal: String,
    pub af: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct DmaChannel {
    pub dma: String,
    pub channel: u32,
    pub dmamux: Option<String>,
    pub dmamux_channel: Option<u32>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Hash)]
pub struct PeripheralDmaChannel {
    pub channel: Option<String>,
    pub dmamux: Option<String>,
    pub request: Option<u32>,
}

struct BlockInfo {
    /// usart_v1/USART -> usart
    module: String,
    /// usart_v1/USART -> v1
    version: String,
    /// usart_v1/USART -> USART
    block: String,
}

impl BlockInfo {
    fn parse(s: &str) -> Self {
        let mut s = s.split("/");
        let module = s.next().unwrap();
        let block = s.next().unwrap();
        assert!(s.next().is_none());
        let mut s = module.split("_");
        let module = s.next().unwrap();
        let version = s.next().unwrap();
        assert!(s.next().is_none());
        Self {
            module: module.to_string(),
            version: version.to_string(),
            block: block.to_string(),
        }
    }
}

fn find_reg_for_field<'c>(
    rcc: &'c ir::IR,
    reg_prefix: &str,
    field_name: &str,
) -> Option<(&'c str, &'c str)> {
    rcc.fieldsets.iter().find_map(|(name, fieldset)| {
        // Workaround for some families that prefix register aliases with C1_, which does
        // not help matching for clock name.
        if name.starts_with("C1") || name.starts_with("C2") {
            None
        } else if name.starts_with(reg_prefix) {
            fieldset
                .fields
                .iter()
                .find_map(|field| {
                    if field_name == field.name {
                        return Some(field.name.as_str());
                    } else {
                        None
                    }
                })
                .map(|n| (name.as_str(), n))
        } else {
            None
        }
    })
}

fn make_peripheral_counts(out: &mut String, data: &HashMap<String, u8>) {
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

fn make_dma_channel_counts(out: &mut String, data: &HashMap<String, u8>) {
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

pub fn gen(options: Options) {
    let generate_opts = generate::Options {
        common_path: syn::parse_str("crate::common").unwrap(),
    };

    let out_dir = options.out_dir;
    let data_dir = options.data_dir;

    fs::create_dir_all(out_dir.join("src/peripherals")).unwrap();
    fs::create_dir_all(out_dir.join("src/chips")).unwrap();

    println!("cwd: {:?}", env::current_dir());

    let mut all_peripheral_versions: HashSet<(String, String)> = HashSet::new();
    let mut chip_cores: HashMap<String, Option<String>> = HashMap::new();

    for chip_name in &options.chips {
        let mut s = chip_name.split('_');
        let chip_name: &str = s.next().unwrap();
        let core_name: Option<&str> = s.next();

        chip_cores.insert(
            chip_name.to_string(),
            core_name.map(|s| s.to_ascii_lowercase().to_string()),
        );

        let chip_path = data_dir.join("chips").join(&format!("{}.yaml", chip_name));
        println!("chip_path: {:?}", chip_path);
        let chip = fs::read(chip_path).unwrap();
        let chip: Chip = serde_yaml::from_slice(&chip).unwrap();

        println!("looking for core {:?}", core_name);
        let core: Option<&Core> = if let Some(core_name) = core_name {
            let core_name = core_name.to_ascii_lowercase();
            let mut c = None;
            for core in chip.cores.iter() {
                if core.name == core_name {
                    c = Some(core);
                    break;
                }
            }
            c
        } else {
            Some(&chip.cores[0])
        };

        let core = core.unwrap();
        let core_name = &core.name;

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
                    let dbgmcu_reg_path = data_dir
                        .join("registers")
                        .join(&format!("{}_{}.yaml", bi.module, bi.version));
                    serde_yaml::from_reader(File::open(dbgmcu_reg_path).unwrap()).unwrap()
                })
            } else {
                None
            }
        });

        // Load RCC register for chip
        let rcc = core.peripherals.iter().find_map(|(name, p)| {
            if name == "RCC" {
                p.block.as_ref().map(|block| {
                    let bi = BlockInfo::parse(block);
                    let rcc_reg_path = data_dir
                        .join("registers")
                        .join(&format!("{}_{}.yaml", bi.module, bi.version));
                    serde_yaml::from_reader(File::open(rcc_reg_path).unwrap()).unwrap()
                })
            } else {
                None
            }
        });

        let mut peripheral_versions: HashMap<String, String> = HashMap::new();
        let mut pin_table: Vec<Vec<String>> = Vec::new();
        let mut interrupt_table: Vec<Vec<String>> = Vec::new();
        let mut peripherals_table: Vec<Vec<String>> = Vec::new();
        let mut peripheral_pins_table: Vec<Vec<String>> = Vec::new();
        let mut peripheral_rcc_table: Vec<Vec<String>> = Vec::new();
        let mut dma_channels_table: Vec<Vec<String>> = Vec::new();
        let mut peripheral_dma_channels_table: Vec<Vec<String>> = Vec::new();
        let mut peripheral_counts: HashMap<String, u8> = HashMap::new();
        let mut dma_channel_counts: HashMap<String, u8> = HashMap::new();
        let mut dbgmcu_table: Vec<Vec<String>> = Vec::new();
        let mut gpio_rcc_table: Vec<Vec<String>> = Vec::new();
        let mut gpio_regs: HashSet<String> = HashSet::new();

        let gpio_base = core.peripherals.get(&"GPIOA".to_string()).unwrap().address;
        let gpio_stride = 0x400;

        let number_suffix_re = Regex::new("^(.*?)[0-9]*$").unwrap();

        if let Some(ref mut reg) = dbgmcu {
            if let Some(ref cr) = reg.fieldsets.get("CR")  {
                for field in cr.fields.iter().filter(|e| e.name.contains("DBG")) {
                    let mut fn_name = String::new();
                    fn_name.push_str("set_");
                    fn_name.push_str( &field.name.to_sanitized_snake_case() );
                    dbgmcu_table.push( vec!( "cr".into(), fn_name ));
                }
            }
        }

        let mut has_bdma = false;
        let mut has_dma = false;

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

                if bi.module == "bdma" {
                    has_bdma = true
                } else if bi.module == "dma" {
                    has_dma = true
                }

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

                for (request, dma_channels) in &p.dma_channels {
                    for channel in dma_channels.iter() {
                        let mut row = Vec::new();
                        row.push(name.clone());
                        row.push(bi.module.clone());
                        row.push(bi.block.clone());
                        row.push(request.clone());
                        if let Some(channel) = &channel.channel {
                            row.push(format!("{{channel: {}}}", channel));
                        } else if let Some(dmamux) = &channel.dmamux {
                            row.push(format!("{{dmamux: {}}}", dmamux));
                        } else {
                            unreachable!();
                        }
                        if let Some(request) = channel.request {
                            row.push(request.to_string());
                        } else {
                            row.push("()".to_string());
                        }
                        peripheral_dma_channels_table.push(row);
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
                        let port_num = port_letter as u32 - 'A' as u32;
                        assert_eq!(p.address, gpio_base + gpio_stride * port_num);

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

                if let Some(rcc) = &rcc {
                    let clock_prefix: Option<&str> = if let Some(clock) = &p.clock {
                        Some(clock)
                    } else if name.starts_with("TIM") {
                        // Not all peripherals like timers the clock hint due to insufficient information from
                        // chip definition. If clock is not specified, the first matching register with the
                        // expected field will be used.
                        Some("")
                    } else {
                        None
                    };

                    if let Some(clock_prefix) = clock_prefix {
                        // Workaround for clock registers being split on some chip families. Assume fields are
                        // named after peripheral and look for first field matching and use that register.
                        let mut en = find_reg_for_field(&rcc, clock_prefix, &format!("{}EN", name));
                        let mut rst =
                            find_reg_for_field(&rcc, clock_prefix, &format!("{}RST", name));

                        if en.is_none() && name.ends_with("1") {
                            en = find_reg_for_field(
                                &rcc,
                                clock_prefix,
                                &format!("{}EN", &name[..name.len() - 1]),
                            );
                            rst = find_reg_for_field(
                                &rcc,
                                clock_prefix,
                                &format!("{}RST", &name[..name.len() - 1]),
                            );
                        }

                        match (en, rst) {
                            (Some((enable_reg, enable_field)), reset_reg_field) => {
                                let clock = if clock_prefix.is_empty() {
                                    let re = Regex::new("([A-Z]+\\d*).*").unwrap();
                                    if !re.is_match(enable_reg) {
                                        panic!(
                                            "unable to derive clock name from register name {}",
                                            enable_reg
                                        );
                                    } else {
                                        let caps = re.captures(enable_reg).unwrap();
                                        caps.get(1).unwrap().as_str()
                                    }
                                } else {
                                    clock_prefix
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
                                    row.push(format!(
                                        "set_{}",
                                        enable_field.to_ascii_lowercase()
                                    ));
                                    row.push(format!(
                                        "set_{}",
                                        reset_field.to_ascii_lowercase()
                                    ));
                                } else {
                                    row.push(format!(
                                        "set_{}",
                                        enable_field.to_ascii_lowercase()
                                    ));
                                }

                                if !name.starts_with("GPIO") {
                                    peripheral_rcc_table.push(row);
                                } else {
                                    gpio_rcc_table.push(row);
                                    gpio_regs.insert( enable_reg.to_ascii_lowercase() );
                                }
                            }
                            (None, Some(_)) => {
                                print!("Unable to find enable register for {}", name)
                            }
                            (None, None) => {
                                print!("Unable to find enable and reset register for {}", name)
                            }
                        }
                    }
                }
            }

            dev.peripherals.push(ir_peri);
        }

        for reg in gpio_regs {
            gpio_rcc_table.push( vec!( reg ) );
        }

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

            if name.starts_with("DMA1_") || name.starts_with("DMA2_") || name.contains("_DMA") {
                if has_dma {
                    interrupt_table.push(vec!["DMA".to_string(), name.clone()]);
                } else if has_bdma {
                    interrupt_table.push(vec!["BDMA".to_string(), name.clone()]);
                }
            }

            if name.starts_with("BDMA_") || name.starts_with("BDMA1_") || name.starts_with("BDMA2_")
            {
                interrupt_table.push(vec!["BDMA".to_string(), name.clone()]);
            }

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

        let peripheral_version_table = peripheral_versions
            .iter()
            .map(|(kind, version)| vec![kind.clone(), version.clone()])
            .collect();

        make_table(&mut extra, "pins", &pin_table);
        make_table(&mut extra, "interrupts", &interrupt_table);
        make_table(&mut extra, "peripherals", &peripherals_table);
        make_table(&mut extra, "peripheral_versions", &peripheral_version_table);
        make_table(&mut extra, "peripheral_pins", &peripheral_pins_table);
        make_table(
            &mut extra,
            "peripheral_dma_channels",
            &peripheral_dma_channels_table,
        );
        make_table(&mut extra, "peripheral_rcc", &peripheral_rcc_table);
        make_table(&mut extra, "gpio_rcc", &gpio_rcc_table);
        make_table(&mut extra, "dma_channels", &dma_channels_table);
        make_table(&mut extra, "dbgmcu", &dbgmcu_table);
        make_peripheral_counts(&mut extra, &peripheral_counts);
        make_dma_channel_counts(&mut extra, &dma_channel_counts);

        for (module, version) in peripheral_versions {
            all_peripheral_versions.insert((module.clone(), version.clone()));
            write!(
                &mut extra,
                "#[path=\"../../peripherals/{}_{}.rs\"] pub mod {};\n",
                module, version, module
            )
            .unwrap();
        }

        // Cleanups!
        transform::sort::Sort {}.run(&mut ir).unwrap();
        transform::Sanitize {}.run(&mut ir).unwrap();

        let chip_dir = if chip.cores.len() > 1 {
            out_dir.join("src/chips").join(format!(
                "{}_{}",
                chip_name.to_ascii_lowercase(),
                core_name.to_ascii_lowercase()
            ))
        } else {
            out_dir
                .join("src/chips")
                .join(chip_name.to_ascii_lowercase())
        };
        fs::create_dir_all(&chip_dir).unwrap();

        let items = generate::render(&ir, &generate_opts).unwrap();
        let mut file = File::create(chip_dir.join("pac.rs")).unwrap();
        let data = items.to_string().replace("] ", "]\n");

        // Remove inner attributes like #![no_std]
        let re = Regex::new("# *! *\\[.*\\]").unwrap();
        let data = re.replace_all(&data, "");
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

        File::create(chip_dir.join("device.x"))
            .unwrap()
            .write_all(device_x.as_bytes())
            .unwrap();
    }

    for (module, version) in all_peripheral_versions {
        println!("loading {} {}", module, version);

        let regs_path = Path::new(&data_dir)
            .join("registers")
            .join(&format!("{}_{}.yaml", module, version));

        let mut ir: ir::IR = serde_yaml::from_reader(File::open(regs_path).unwrap()).unwrap();

        transform::expand_extends::ExpandExtends {}
            .run(&mut ir)
            .unwrap();

        transform::map_names(&mut ir, |s, k| match k {
            transform::NameKind::Block => format!("{}", s),
            transform::NameKind::Fieldset => format!("regs::{}", s),
            transform::NameKind::Enum => format!("vals::{}", s),
            _ => s.to_string(),
        })
        .unwrap();

        transform::sort::Sort {}.run(&mut ir).unwrap();
        transform::Sanitize {}.run(&mut ir).unwrap();

        let items = generate::render(&ir, &generate_opts).unwrap();
        let mut file = File::create(
            out_dir
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

    for (chip, cores) in chip_cores.iter() {
        let x = chip.to_ascii_lowercase();
        if let Some(c) = cores {
            write!(
                &mut paths,
                "#[cfg_attr(feature=\"{}_{}\", path = \"chips/{}_{}/pac.rs\")]",
                x, c, x, c
            )
            .unwrap();
        } else {
            write!(
                &mut paths,
                "#[cfg_attr(feature=\"{}\", path = \"chips/{}/pac.rs\")]",
                x, x
            )
            .unwrap();
        }
    }
    let mut contents: Vec<u8> = Vec::new();
    contents.extend(&librs[..i]);
    contents.extend(paths.as_bytes());
    contents.extend(&librs[i + PATHS_MARKER.len()..]);
    fs::write(out_dir.join("src").join("lib_inner.rs"), &contents).unwrap();

    // Generate src/lib.rs
    const CUT_MARKER: &[u8] = b"// GEN CUT HERE";
    let librs = include_bytes!("../../stm32-metapac/src/lib.rs");
    let i = bytes_find(librs, CUT_MARKER).unwrap();
    let mut contents: Vec<u8> = Vec::new();
    contents.extend(&librs[..i]);
    contents.extend(b"include!(\"lib_inner.rs\");\n");
    fs::write(out_dir.join("src").join("lib.rs"), contents).unwrap();

    // Generate src/common.rs
    fs::write(
        out_dir.join("src").join("common.rs"),
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
    fs::write(out_dir.join("Cargo.toml"), contents).unwrap();

    // Generate build.rs
    fs::write(out_dir.join("build.rs"), include_bytes!("assets/build.rs")).unwrap();
}

fn bytes_find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
