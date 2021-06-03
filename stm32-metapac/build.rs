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

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct Chip {
    pub name: String,
    pub family: String,
    pub line: String,
    pub core: String,
    pub flash: u32,
    pub ram: u32,
    pub gpio_af: String,
    pub packages: Vec<Package>,
    pub peripherals: HashMap<String, Peripheral>,
    pub interrupts: HashMap<String, u32>,
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
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct Pin {
    pub pin: String,
    pub signal: String,
    pub af: Option<String>,
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

fn main() {
    let dir = "../stm32-data/data";

    println!("cwd: {:?}", env::current_dir());

    let chip_name = env::vars_os()
        .map(|(a, _)| a.to_string_lossy().to_string())
        .find(|x| x.starts_with("CARGO_FEATURE_STM32"))
        .expect("No stm32xx Cargo feature enabled")
        .strip_prefix("CARGO_FEATURE_")
        .unwrap()
        .to_ascii_uppercase();

    let chip_path = Path::new(&dir)
        .join("chips")
        .join(&format!("{}.yaml", chip_name));
    let chip = fs::read(chip_path).unwrap();
    let chip: Chip = serde_yaml::from_slice(&chip).unwrap();

    let mut ir = ir::IR::new();

    let mut dev = ir::Device {
        interrupts: Vec::new(),
        peripherals: Vec::new(),
    };

    let mut peripheral_versions: HashMap<String, String> = HashMap::new();
    let mut cfgs: HashSet<String> = HashSet::new();
    let mut pin_table: Vec<Vec<String>> = Vec::new();
    let mut interrupt_table: Vec<Vec<String>> = Vec::new();
    let mut peripherals_table: Vec<Vec<String>> = Vec::new();
    let mut peripheral_pins_table: Vec<Vec<String>> = Vec::new();

    let dma_base = chip
        .peripherals
        .get(&"DMA".to_string())
        .unwrap_or_else(|| chip.peripherals.get(&"DMA1".to_string()).unwrap())
        .address;
    let dma_stride = 0x400;

    let gpio_base = chip.peripherals.get(&"GPIOA".to_string()).unwrap().address;
    let gpio_stride = 0x400;

    cfgs.insert(chip.family.to_ascii_lowercase().replace("+", "plus"));

    for (name, p) in &chip.peripherals {
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

            cfgs.insert(bi.module.clone());
            cfgs.insert(format!("{}_{}", bi.module, bi.version));
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
                "dma" => {
                    let dma_num = if name == "DMA" {
                        0
                    } else {
                        let dma_letter = name.chars().skip(3).next().unwrap();
                        dma_letter as u32 - '1' as u32
                    };
                    assert_eq!(p.address, dma_base + dma_stride * dma_num);
                }
                _ => {}
            }
        }

        dev.peripherals.push(ir_peri);
    }

    for (name, &num) in &chip.interrupts {
        dev.interrupts.push(ir::Interrupt {
            name: name.clone(),
            description: None,
            value: num,
        });

        interrupt_table.push(vec![name.to_ascii_uppercase()]);
    }

    ir.devices.insert("".to_string(), dev);

    let mut extra = format!(
        "pub fn GPIO(n: usize) -> gpio::Gpio {{
            gpio::Gpio(({} + {}*n) as _)
        }}
        pub fn DMA(n: usize) -> dma::Dma {{
            dma::Dma(({} + {}*n) as _)
        }}",
        gpio_base, gpio_stride, dma_base, dma_stride,
    );

    let peripheral_version_table = peripheral_versions
        .iter()
        .map(|(kind, version)| vec![kind.clone(), version.clone()])
        .collect();

    let exti_interrupt_table = &interrupt_table
        .iter()
        .filter(|row| row[0].contains("EXTI"))
        .map(|row| row.clone())
        .collect();

    make_table(&mut extra, "pins", &pin_table);
    make_table(&mut extra, "interrupts", &interrupt_table);
    make_table(&mut extra, "exti_interrupts", &exti_interrupt_table);
    make_table(&mut extra, "peripherals", &peripherals_table);
    make_table(&mut extra, "peripheral_versions", &peripheral_version_table);
    make_table(&mut extra, "peripheral_pins", &peripheral_pins_table);

    for (module, version) in peripheral_versions {
        println!("loading {} {}", module, version);

        let regs_path = Path::new(&dir)
            .join("registers")
            .join(&format!("{}_{}.yaml", module, version));

        let mut peri: ir::IR = serde_yaml::from_reader(File::open(regs_path).unwrap()).unwrap();

        transform::expand_extends::ExpandExtends {}
            .run(&mut peri)
            .unwrap();

        let prefix = module;
        transform::map_names(&mut peri, |s, k| match k {
            transform::NameKind::Block => format!("{}::{}", prefix, s),
            transform::NameKind::Fieldset => format!("{}::regs::{}", prefix, s),
            transform::NameKind::Enum => format!("{}::vals::{}", prefix, s),
            _ => s.to_string(),
        })
        .unwrap();

        ir.merge(peri);
    }

    // Cleanups!
    transform::sort::Sort {}.run(&mut ir).unwrap();
    transform::Sanitize {}.run(&mut ir).unwrap();

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let items = generate::render(&ir).unwrap();
    let mut file = File::create(out.join("pac.rs")).unwrap();
    let data = items.to_string().replace("] ", "]\n");

    // Remove inner attributes like #![no_std]
    let re = Regex::new("# *! *\\[.*\\]").unwrap();
    let data = re.replace_all(&data, "");
    file.write_all(data.as_bytes()).unwrap();
    file.write_all(extra.as_bytes()).unwrap();

    let mut device_x = String::new();

    for (name, _) in &chip.interrupts {
        write!(
            &mut device_x,
            "PROVIDE({} = DefaultHandler);\n",
            name.to_ascii_uppercase()
        )
        .unwrap();
    }

    File::create(out.join("device.x"))
        .unwrap()
        .write_all(device_x.as_bytes())
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rerun-if-changed=build.rs");

    println!(
        "cargo:cfgs={}",
        cfgs.into_iter().collect::<Vec<_>>().join(",")
    );
}
