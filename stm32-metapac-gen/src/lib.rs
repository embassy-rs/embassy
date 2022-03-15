use chiptool::generate::CommonModule;
use chiptool::{generate, ir, transform};
use proc_macro2::TokenStream;
use regex::Regex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::{Debug, Write as _};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

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

pub struct Options {
    pub chips: Vec<String>,
    pub out_dir: PathBuf,
    pub data_dir: PathBuf,
}

pub struct Gen {
    opts: Options,
    all_peripheral_versions: HashSet<(String, String)>,
    metadata_dedup: HashMap<String, String>,
}

impl Gen {
    pub fn new(opts: Options) -> Self {
        Self {
            opts,
            all_peripheral_versions: HashSet::new(),
            metadata_dedup: HashMap::new(),
        }
    }

    fn gen_chip(&mut self, chip_core_name: &str, chip: &Chip, core: &Core, core_index: usize) {
        let mut ir = ir::IR::new();

        let mut dev = ir::Device {
            interrupts: Vec::new(),
            peripherals: Vec::new(),
        };

        let mut peripheral_versions: BTreeMap<String, String> = BTreeMap::new();

        let gpio_base = core
            .peripherals
            .iter()
            .find(|p| p.name == "GPIOA")
            .unwrap()
            .address as u32;
        let gpio_stride = 0x400;

        for p in &core.peripherals {
            let mut ir_peri = ir::Peripheral {
                name: p.name.clone(),
                array: None,
                base_address: p.address,
                block: None,
                description: None,
                interrupts: HashMap::new(),
            };

            if let Some(bi) = &p.registers {
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

                if bi.kind == "gpio" {
                    assert_eq!(0, (p.address as u32 - gpio_base) % gpio_stride);
                }
            }

            dev.peripherals.push(ir_peri);
        }

        for irq in &core.interrupts {
            dev.interrupts.push(ir::Interrupt {
                name: irq.name.clone(),
                description: None,
                value: irq.number,
            });
        }

        ir.devices.insert("".to_string(), dev);

        let mut extra = format!(
            "pub fn GPIO(n: usize) -> gpio::Gpio {{
            gpio::Gpio(({} + {}*n) as _)
        }}",
            gpio_base, gpio_stride,
        );

        for (module, version) in &peripheral_versions {
            self.all_peripheral_versions
                .insert((module.clone(), version.clone()));
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

        let chip_dir = self
            .opts
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
        // generate metadata.rs

        // (peripherals, interrupts, dma_channels) are often equal across multiple chips.
        // To reduce bloat, deduplicate them.
        let mut data = String::new();
        write!(
            &mut data,
            "
                const PERIPHERALS: &'static [Peripheral] = {};
                const INTERRUPTS: &'static [Interrupt] = {};
                const DMA_CHANNELS: &'static [DmaChannel] = {};
            ",
            stringify(&core.peripherals),
            stringify(&core.interrupts),
            stringify(&core.dma_channels),
        )
        .unwrap();

        let out_dir = self.opts.out_dir.clone();
        let n = self.metadata_dedup.len();
        let deduped_file = self.metadata_dedup.entry(data.clone()).or_insert_with(|| {
            let file = format!("metadata_{:04}.rs", n);
            let path = out_dir.join("src/chips").join(&file);
            fs::write(path, data).unwrap();

            file
        });

        let data = format!(
            "include!(\"../{}\");
            pub const METADATA: Metadata = Metadata {{
                name: {:?},
                family: {:?},
                line: {:?},
                memory: {},
                peripherals: PERIPHERALS,
                interrupts: INTERRUPTS,
                dma_channels: DMA_CHANNELS,
            }};",
            deduped_file,
            &chip.name,
            &chip.family,
            &chip.line,
            stringify(&chip.memory),
        );

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

    fn load_chip(&mut self, name: &str) -> Chip {
        let chip_path = self
            .opts
            .data_dir
            .join("chips")
            .join(&format!("{}.json", name));
        let chip = fs::read(chip_path).expect(&format!("Could not load chip {}", name));
        serde_yaml::from_slice(&chip).unwrap()
    }

    pub fn gen(&mut self) {
        fs::create_dir_all(self.opts.out_dir.join("src/peripherals")).unwrap();
        fs::create_dir_all(self.opts.out_dir.join("src/chips")).unwrap();

        let mut chip_core_names: Vec<String> = Vec::new();

        for chip_name in &self.opts.chips.clone() {
            println!("Generating {}...", chip_name);

            let mut chip = self.load_chip(chip_name);

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
                self.gen_chip(&chip_core_name, &chip, core, core_index)
            }
        }

        for (module, version) in &self.all_peripheral_versions {
            println!("loading {} {}", module, version);

            let regs_path = Path::new(&self.opts.data_dir)
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
                self.opts
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

        // Generate Cargo.toml
        const BUILDDEP_BEGIN: &[u8] = b"# BEGIN BUILD DEPENDENCIES";
        const BUILDDEP_END: &[u8] = b"# END BUILD DEPENDENCIES";

        let mut contents = include_bytes!("../../stm32-metapac/Cargo.toml").to_vec();
        let begin = bytes_find(&contents, BUILDDEP_BEGIN).unwrap();
        let end = bytes_find(&contents, BUILDDEP_END).unwrap() + BUILDDEP_END.len();
        contents.drain(begin..end);
        fs::write(self.opts.out_dir.join("Cargo.toml"), contents).unwrap();

        // copy misc files
        fs::write(
            self.opts.out_dir.join("build.rs"),
            include_bytes!("../../stm32-metapac/build_pregenerated.rs"),
        )
        .unwrap();
        fs::write(
            self.opts.out_dir.join("src/lib.rs"),
            include_bytes!("../../stm32-metapac/src/lib.rs"),
        )
        .unwrap();
        fs::write(
            self.opts.out_dir.join("src/common.rs"),
            chiptool::generate::COMMON_MODULE,
        )
        .unwrap();
        fs::write(
            self.opts.out_dir.join("src/metadata.rs"),
            include_bytes!("../../stm32-metapac/src/metadata.rs"),
        )
        .unwrap();
    }
}

fn bytes_find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn stringify<T: Debug>(metadata: T) -> String {
    let mut metadata = format!("{:?}", metadata);
    if metadata.starts_with('[') {
        metadata = format!("&{}", metadata);
    }
    metadata = metadata.replace(": [", ": &[");
    metadata = metadata.replace("kind: Ram", "kind: MemoryRegionKind::Ram");
    metadata = metadata.replace("kind: Flash", "kind: MemoryRegionKind::Flash");
    metadata
}

fn gen_opts() -> generate::Options {
    generate::Options {
        common_module: CommonModule::External(TokenStream::from_str("crate::common").unwrap()),
    }
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
