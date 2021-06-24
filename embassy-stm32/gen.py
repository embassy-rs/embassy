import sys
import yaml
import re
import os
import re

try:
    from yaml import CSafeLoader as SafeLoader
except ImportError:
    from yaml import SafeLoader


abspath = os.path.abspath(__file__)
dname = os.path.dirname(abspath)
os.chdir(dname)

data_path = '../stm32-data/data'

try:
    _, chip_name, output_file = sys.argv
except:
    raise Exception("Usage: gen.py STM32F429ZI_CM0 path/to/generated.rs")

c = chip_name.split('_', 1)

chip_name = c[0].upper()
core_name = None

if len(c) > 1:
    core_name = c[1].lower()

# ======= load chip
with open(f'{data_path}/chips/{chip_name}.yaml', 'r') as f:
    chip = yaml.load(f, Loader=SafeLoader)

# ======= Generate!
with open(output_file, 'w') as f:
    singletons = []  # USART1, PA5, EXTI8
    exti_interrupts = []  # EXTI IRQs, EXTI0, EXTI4_15 etc.
    pins = set()  # set of all present pins. PA4, PA5...

    # ========= peripherals

    singletons.extend((f'EXTI{x}' for x in range(16)))
    num_dmas = 0

    core = chip['cores'][0]
    if core_name != None:
        for c in chip['cores']:
            if core_name == c['name']:
                core = c

    for (name, peri) in core['peripherals'].items():
        if 'block' not in peri:
            continue

        block = peri['block']
        block_mod, block_name_unparsed = block.rsplit('/')
        block_mod, block_version = block_mod.rsplit('_')
        block_name = ''
        for b in block_name_unparsed.split('_'):
            block_name += b.capitalize()

        custom_singletons = False

        if block_mod == 'gpio':
            custom_singletons = True
            port = name[4:]
            port_num = ord(port) - ord('A')

            for pin_num in range(16):
                pin = f'P{port}{pin_num}'
                pins.add(pin)
                singletons.append(pin)

        if block_mod == 'dma':
            custom_singletons = True
            for ch_num in range(8):
                channel = f'{name}_CH{ch_num}'
                singletons.append(channel)

        if not custom_singletons:
            singletons.append(name)

    f.write(f"embassy_extras::peripherals!({','.join(singletons)});")
