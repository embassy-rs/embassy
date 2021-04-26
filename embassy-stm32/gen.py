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
        chip = yaml.load(f, Loader=yaml.SafeLoader)
    chip['name'] = chip['name'].lower()
    print(chip['name'])
    chips[chip['name']] = chip

# ======= load GPIO AF
gpio_afs = {}
for f in sorted(glob('stm32-data/data/gpio_af/*.yaml')):
    name = f.split('/')[-1].split('.')[0]
    with open(f, 'r') as f:
        af = yaml.load(f, Loader=yaml.SafeLoader)
    gpio_afs[name] = af

# ========= Update chip/mod.rs

with open('src/chip/mod.rs', 'w') as f:
    for chip in chips.values():
        f.write(
            f'#[cfg_attr(feature="{chip["name"]}", path="{chip["name"]}.rs")]\n')
    f.write('mod chip;\n')
    f.write('pub use chip::*;\n')

# ========= Update Cargo features

features = {name: [] for name, chip in chips.items()}

SEPARATOR_START = '# BEGIN GENERATED FEATURES\n'
SEPARATOR_END = '# END GENERATED FEATURES\n'

with open('Cargo.toml', 'r') as f:
    cargo = f.read()
before, cargo = cargo.split(SEPARATOR_START, maxsplit=1)
_, after = cargo.split(SEPARATOR_END, maxsplit=1)
cargo = before + SEPARATOR_START + toml.dumps(features) + SEPARATOR_END + after
with open('Cargo.toml', 'w') as f:
    f.write(cargo)

# ========= Generate per-chip mod

for chip in chips.values():
    print(f'generating {chip["name"]}')
    af = gpio_afs[chip['gpio_af']]
    peripherals = []
    impls = []
    pins = set()

    # TODO this should probably come from the yamls?
    # We don't want to hardcode the EXTI peripheral addr
    peripherals.extend((f'EXTI{x}' for x in range(16)))

    gpio_base = chip['peripherals']['GPIOA']['address']
    gpio_stride = 0x400

    for (name, peri) in chip['peripherals'].items():
        if name.startswith('GPIO'):
            port = name[4:]
            port_num = ord(port) - ord('A')

            assert peri['address'] == gpio_base + gpio_stride*port_num

            for pin_num in range(16):
                pin = f'P{port}{pin_num}'
                pins.add(pin)
                peripherals.append(pin)
                impls.append(f'impl_gpio_pin!({pin}, {port_num}, {pin_num}, EXTI{pin_num});')
            continue

        # TODO maybe we should only autogenerate the known ones...??
        peripherals.append(name)

        if 'block' not in peri:
            continue

        if peri['block'] == 'usart_v1/USART':
            impls.append(f'impl_usart!({name}, 0x{peri["address"]:x});')
            for pin, funcs in af.items():
                if pin in pins:
                    if func := funcs.get(f'{name}_RX'):
                        impls.append(f'impl_usart_pin!({name}, RxPin, {pin}, {func});')
                    if func := funcs.get(f'{name}_TX'):
                        impls.append(f'impl_usart_pin!({name}, TxPin, {pin}, {func});')
                    if func := funcs.get(f'{name}_CTS'):
                        impls.append(f'impl_usart_pin!({name}, CtsPin, {pin}, {func});')
                    if func := funcs.get(f'{name}_RTS'):
                        impls.append(f'impl_usart_pin!({name}, RtsPin, {pin}, {func});')
                    if func := funcs.get(f'{name}_CK'):
                        impls.append(f'impl_usart_pin!({name}, CkPin, {pin}, {func});')

        if peri['block'] == 'rng_v1/RNG':
            impls.append(f'impl_rng!(0x{peri["address"]:x});')

    with open(f'src/chip/{chip["name"]}.rs', 'w') as f:
        # TODO uart etc
        # TODO import the right GPIO AF map mod
        # TODO impl traits for the periperals

        f.write(f"""
        use embassy_extras::peripherals;
        peripherals!({','.join(peripherals)});
        pub const GPIO_BASE: usize = 0x{gpio_base:x};
        pub const GPIO_STRIDE: usize = 0x{gpio_stride:x};
        """)
        for i in impls:
            f.write(i)


# TODO generate GPIO AF map mods


# format
os.system('rustfmt src/chip/*')
