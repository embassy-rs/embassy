import xmltodict
import yaml
import re
import json
import os
import toml
from collections import OrderedDict
from glob import glob

abspath = os.path.abspath(__file__)
dname = os.path.dirname(abspath)
os.chdir(dname)

# ======= load chips
chips = {}
for f in sorted(glob('stm32-data/data/chips/*.yaml')):
    if 'STM32F4' not in f and 'STM32L4' not in f:
        continue
    with open(f, 'r') as f:
        chip = yaml.load(f, Loader=yaml.CSafeLoader)
    chip['name'] = chip['name'].lower()
    chip['features'] = set()
    family = chip["family"].lower().replace('+', 'p')
    chip['features'].add(f'_{family}')
    print(chip['name'])
    chips[chip['name']] = chip

# ======= load GPIO AF
gpio_afs = {}
for f in sorted(glob('stm32-data/data/gpio_af/*.yaml')):
    name = f.split('/')[-1].split('.')[0]
    with open(f, 'r') as f:
        af = yaml.load(f, Loader=yaml.CSafeLoader)
    gpio_afs[name] = af

# ========= Generate pac/mod.rs

with open('src/pac/mod.rs', 'w') as f:
    for chip in chips.values():
        f.write(
            f'#[cfg_attr(feature="{chip["name"]}", path="{chip["name"]}.rs")]\n')
    f.write('mod chip;\n')
    f.write('pub use chip::*;\n')

# ========= Generate pac/stm32xxx.rs

for chip in chips.values():
    print(f'generating {chip["name"]}')
    with open(f'src/pac/{chip["name"]}.rs', 'w') as f:

        f.write("""
            #![allow(dead_code)]
            #![allow(unused_imports)]
            #![allow(non_snake_case)]
        """)

        af = gpio_afs[chip['gpio_af']]
        peripheral_names = []  # USART1, PA5, EXTI8
        peripheral_versions = {}  # usart -> v1, syscfg -> f4
        pins = set()  # set of all present pins. PA4, PA5...

        # TODO this should probably come from the yamls?
        # We don't want to hardcode the EXTI peripheral addr

        gpio_base = chip['peripherals']['GPIOA']['address']
        gpio_stride = 0x400
        f.write(f"""
            pub fn GPIO(n: usize) -> gpio::Gpio {{
                gpio::Gpio((0x{gpio_base:x} + 0x{gpio_stride:x}*n) as _)
            }}
        """)

        # ========= GPIO

        peripheral_names.extend((f'EXTI{x}' for x in range(16)))

        for (name, peri) in chip['peripherals'].items():
            if not name.startswith('GPIO'):
                continue

            port = name[4:]
            port_num = ord(port) - ord('A')

            assert peri['address'] == gpio_base + gpio_stride*port_num

            for pin_num in range(16):
                pin = f'P{port}{pin_num}'
                pins.add(pin)
                peripheral_names.append(pin)
                f.write(f'impl_gpio_pin!({pin}, {port_num}, {pin_num}, EXTI{pin_num});')

        # ========= peripherals

        for (name, peri) in chip['peripherals'].items():
            if 'block' not in peri:
                continue

            if not name.startswith('GPIO'):
                peripheral_names.append(name)

            block = peri['block']
            block_mod, block_name = block.rsplit('/')
            block_mod, block_version = block_mod.rsplit('_')
            block_name = block_name.capitalize()

            # Check all peripherals have the same version: it's not OK for the same chip to use both usart_v1 and usart_v2
            if old_version := peripheral_versions.get(block_mod):
                if old_version != block_version:
                    raise Exception(f'Peripheral {block_mod} has two versions: {old_version} and {block_version}')
            peripheral_versions[block_mod] = block_version

            # Set features
            chip['features'].add(f'_{block_mod}')
            chip['features'].add(f'_{block_mod}_{block_version}')

            f.write(f'pub const {name}: {block_mod}::{block_name} = {block_mod}::{block_name}(0x{peri["address"]:x} as _);')

            if peri['block'] in ('usart_v1/USART', 'usart_v1/UART'):
                f.write(f'impl_usart!({name});')
                for pin, funcs in af.items():
                    if pin in pins:
                        if func := funcs.get(f'{name}_RX'):
                            f.write(f'impl_usart_pin!({name}, RxPin, {pin}, {func});')
                        if func := funcs.get(f'{name}_TX'):
                            f.write(f'impl_usart_pin!({name}, TxPin, {pin}, {func});')
                        if func := funcs.get(f'{name}_CTS'):
                            f.write(f'impl_usart_pin!({name}, CtsPin, {pin}, {func});')
                        if func := funcs.get(f'{name}_RTS'):
                            f.write(f'impl_usart_pin!({name}, RtsPin, {pin}, {func});')
                        if func := funcs.get(f'{name}_CK'):
                            f.write(f'impl_usart_pin!({name}, CkPin, {pin}, {func});')

            if peri['block'] == 'rng_v1/RNG':
                f.write(f'impl_rng!({name});')

        for mod, version in peripheral_versions.items():
            f.write(f'pub use regs::{mod}_{version} as {mod};')

        f.write(f"""
            mod regs;
            pub use regs::generic;
            use embassy_extras::peripherals;
            peripherals!({','.join(peripheral_names)});
        """)

        # ========= interrupts

        irq_variants = []
        irq_vectors = []
        irq_fns = []
        irq_declares = []

        irqs = {num: name for name, num in chip['interrupts'].items()}
        irq_count = max(irqs.keys()) + 1
        for num, name in irqs.items():
            irq_variants.append(f'{name} = {num},')
            irq_fns.append(f'fn {name}();')
            irq_declares.append(f'declare!({name});')
        for num in range(irq_count):
            if name := irqs.get(num):
                irq_vectors.append(f'Vector {{ _handler: {name} }},')
            else:
                irq_vectors.append(f'Vector {{ _reserved: 0 }},')

        f.write(f"""
            pub mod interrupt {{
                pub use cortex_m::interrupt::{{CriticalSection, Mutex}};
                pub use embassy::interrupt::{{declare, take, Interrupt}};
                pub use embassy_extras::interrupt::Priority4 as Priority;

                #[derive(Copy, Clone, Debug, PartialEq, Eq)]
                #[allow(non_camel_case_types)]
                enum InterruptEnum {{
                    {''.join(irq_variants)}
                }}
                unsafe impl cortex_m::interrupt::InterruptNumber for InterruptEnum {{
                    #[inline(always)]
                    fn number(self) -> u16 {{
                        self as u16
                    }}
                }}

                {''.join(irq_declares)}
            }}
            mod interrupt_vector {{
                extern "C" {{
                    {''.join(irq_fns)} 
                }}
                pub union Vector {{
                    _handler: unsafe extern "C" fn(),
                    _reserved: u32,
                }}
                #[link_section = ".vector_table.interrupts"]
                #[no_mangle]
                pub static __INTERRUPTS: [Vector; {irq_count}] = [
                    {''.join(irq_vectors)}
                ];
            }}
        """)


# ========= Update Cargo features

feature_optional_deps = {}
feature_optional_deps['_rng'] = ['rand_core']

features = {}
extra_features = set()
for name, chip in chips.items():
    features[name] = sorted(list(chip['features']))
    for feature in chip['features']:
        extra_features.add(feature)
for feature in sorted(list(extra_features)):
    features[feature] = feature_optional_deps.get(feature) or []

SEPARATOR_START = '# BEGIN GENERATED FEATURES\n'
SEPARATOR_END = '# END GENERATED FEATURES\n'

with open('Cargo.toml', 'r') as f:
    cargo = f.read()
before, cargo = cargo.split(SEPARATOR_START, maxsplit=1)
_, after = cargo.split(SEPARATOR_END, maxsplit=1)
cargo = before + SEPARATOR_START + toml.dumps(features) + SEPARATOR_END + after
with open('Cargo.toml', 'w') as f:
    f.write(cargo)

# ========= Generate pac/regs.rs
os.system('cargo run --manifest-path ../../svd2rust/Cargo.toml -- generate --dir stm32-data/data/registers')
os.system('mv lib.rs src/pac/regs.rs')

# ========= Update Cargo features
os.system('rustfmt src/pac/*')
