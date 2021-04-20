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

chips = {}
for f in sorted(glob('stm32-data/data/chips/*.yaml')):
    if 'STM32F4' not in f:
        continue
    with open(f, 'r') as f:
        chip = yaml.load(f, Loader=yaml.SafeLoader)
    chip['name'] = chip['name'].lower()
    print(chip['name'])
    chips[chip['name']] = chip

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
    peripherals = []
    peripherals.extend((f'EXTI{x}' for x in range(16)))

    # TODO get the number of ports from the YAML when stm32-data includes it
    for port in 'ABCD':
        peripherals.extend((f'P{port}{x}' for x in range(16)))

    with open(f'src/chip/{chip["name"]}.rs', 'w') as f:
        # TODO uart etc
        # TODO import the right GPIO AF map mod
        # TODO impl traits for the periperals

        f.write(f"""
        use embassy_extras::peripherals;
        peripherals!({','.join(peripherals)});
        """)


# TODO generate GPIO AF map mods
