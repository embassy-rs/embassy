import sys
import yaml
import re
import os
import re

abspath = os.path.abspath(__file__)
dname = os.path.dirname(abspath)
os.chdir(dname)

data_path = '../stm32-data/data'

try:
    _, chip_name, output_file = sys.argv
except:
    raise Exception("Usage: gen.py STM32F429ZI path/to/generated.rs")

# ======= load chip
chip_name = chip_name.upper()
with open(f'{data_path}/chips/{chip_name}.yaml', 'r') as f:
    chip = yaml.load(f, Loader=yaml.CSafeLoader)

# ======= load GPIO AF
with open(f'{data_path}/gpio_af/{chip["gpio_af"]}.yaml', 'r') as f:
    af = yaml.load(f, Loader=yaml.CSafeLoader)

# ======= Generate!
with open(output_file, 'w') as f:
    singletons = []  # USART1, PA5, EXTI8
    exti_interrupts = []  # EXTI IRQs, EXTI0, EXTI4_15 etc.
    pins = set()  # set of all present pins. PA4, PA5...

    # ========= peripherals

    singletons.extend((f'EXTI{x}' for x in range(16)))
    num_dmas = 0

    for (name, peri) in chip['peripherals'].items():
        if 'block' not in peri:
            continue

        block = peri['block']
        block_mod, block_name_unparsed = block.rsplit('/')
        block_mod, block_version = block_mod.rsplit('_')
        block_name = ''
        for b in block_name_unparsed.split('_'):
            block_name += b.capitalize()

        custom_singletons = False

        if block_mod == 'usart':
            f.write(f'impl_usart!({name});')
            for pin, funcs in af.items():
                if pin in pins:
                    if (func := funcs.get(f'{name}_RX')) != None:
                        f.write(f'impl_usart_pin!({name}, RxPin, {pin}, {func});')
                    if (func := funcs.get(f'{name}_TX')) != None:
                        f.write(f'impl_usart_pin!({name}, TxPin, {pin}, {func});')
                    if (func := funcs.get(f'{name}_CTS')) != None:
                        f.write(f'impl_usart_pin!({name}, CtsPin, {pin}, {func});')
                    if (func := funcs.get(f'{name}_RTS')) != None:
                        f.write(f'impl_usart_pin!({name}, RtsPin, {pin}, {func});')
                    if (func := funcs.get(f'{name}_CK')) != None:
                        f.write(f'impl_usart_pin!({name}, CkPin, {pin}, {func});')

        if block_mod == 'rng':
            for irq in chip['interrupts']:
                if re.search('RNG', irq):
                    f.write(f'impl_rng!({name}, {irq});')

        if block_mod == 'spi':
            if 'clock' in peri:
                clock = peri['clock']
                f.write(f'impl_spi!({name}, {clock});')
                for pin, funcs in af.items():
                    if pin in pins:
                        if (func := funcs.get(f'{name}_SCK')) != None:
                            f.write(f'impl_spi_pin!({name}, SckPin, {pin}, {func});')
                        if (func := funcs.get(f'{name}_MOSI')) != None:
                            f.write(f'impl_spi_pin!({name}, MosiPin, {pin}, {func});')
                        if (func := funcs.get(f'{name}_MISO')) != None:
                            f.write(f'impl_spi_pin!({name}, MisoPin, {pin}, {func});')

        if block_mod == 'i2c':
            f.write(f'impl_i2c!({name});')
            for pin, funcs in af.items():
                if pin in pins:
                    if func := funcs.get(f'{name}_SCL'):
                        f.write(f'impl_i2c_pin!({name}, SclPin, {pin}, {func});')
                    if func := funcs.get(f'{name}_SDA'):
                        f.write(f'impl_i2c_pin!({name}, SdaPin, {pin}, {func});')

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
            num_dmas += 1
            dma_num = int(name[3:])-1  # substract 1 because we want DMA1=0, DMA2=1

            for ch_num in range(8):
                channel = f'{name}_CH{ch_num}'
                singletons.append(channel)

                f.write(f'impl_dma_channel!({channel}, {dma_num}, {ch_num});')

        if peri['block'] == 'sdmmc_v2/SDMMC':
            f.write(f'impl_sdmmc!({name});')
            for pin, funcs in af.items():
                if pin in pins:
                    if (func := funcs.get(f'{name}_CK')) != None:
                        f.write(f'impl_sdmmc_pin!({name}, CkPin, {pin}, {func});')
                    if (func := funcs.get(f'{name}_CMD')) != None:
                        f.write(f'impl_sdmmc_pin!({name}, CmdPin, {pin}, {func});')
                    if (func := funcs.get(f'{name}_D0')) != None:
                        f.write(f'impl_sdmmc_pin!({name}, D0Pin, {pin}, {func});')
                    if (func := funcs.get(f'{name}_D1')) != None:
                        f.write(f'impl_sdmmc_pin!({name}, D1Pin, {pin}, {func});')
                    if (func := funcs.get(f'{name}_D2')) != None:
                        f.write(f'impl_sdmmc_pin!({name}, D2Pin, {pin}, {func});')
                    if (func := funcs.get(f'{name}_D3')) != None:
                        f.write(f'impl_sdmmc_pin!({name}, D3Pin, {pin}, {func});')
                    if (func := funcs.get(f'{name}_D4')) != None:
                        f.write(f'impl_sdmmc_pin!({name}, D4Pin, {pin}, {func});')
                    if (func := funcs.get(f'{name}_D5')) != None:
                        f.write(f'impl_sdmmc_pin!({name}, D5Pin, {pin}, {func});')
                    if (func := funcs.get(f'{name}_D6')) != None:
                        f.write(f'impl_sdmmc_pin!({name}, D6Pin, {pin}, {func});')
                    if (func := funcs.get(f'{name}_D7')) != None:
                        f.write(f'impl_sdmmc_pin!({name}, D7Pin, {pin}, {func});')

        if block_name == 'TimGp16':
            if re.match('TIM[2345]$', name):
                f.write(f'impl_timer!({name});')

        if block_mod == 'exti':
            for irq in chip['interrupts']:
                if re.match('EXTI', irq):
                    exti_interrupts.append(irq)

        if not custom_singletons:
            singletons.append(name)

    f.write(f"embassy_extras::peripherals!({','.join(singletons)});")

    # ========= exti interrupts
    f.write(f"impl_exti_irq!({','.join(exti_interrupts)});")
