# Booting the N6 from flash

Note: A copy of this is located here: https://github.com/embassy-rs/embassy/wiki/Booting-the-N6-from-flash

A Guide to how I currently boot from flash on the N6. My repo is laid out as a workspace with a bootloader and firmware. Unfortunately I'm not able to share that repo, but I can provide relevant snippets.

# Build instructions

1. Build the bootloader as a binary and sign it
```console
cd bootloader
cargo objcopy -- -O binary ../fsbl.bin
/usr/local/STMicroelectronics/STM32Cube/STM32CubeProgrammer/bin/STM32_SigningTool_CLI -bin ../fsbl.bin -nk -of 0x80000000 -t fsbl -o ../fsbl-trusted.bin -hv 2.3 -dump ../fsbl-trusted.bin -s
```
2. Use the STM32_Programmer or STM32_Programmer_CLI to flash `fsbl-trusted.bin` to 0x70000000
3. Build the firmware as a binary and sign it
```console
cd firmware
cargo objcopy -- -O binary ../app.bin
/usr/local/STMicroelectronics/STM32Cube/STM32CubeProgrammer/bin/STM32_SigningTool_CLI -bin ../app.bin -nk -of 0x80000000 -t fsbl -o ../app-trusted.bin -hv 2.3 -dump ../app-trusted.bin -s
```
4. Use the STM32_Programmer or STM32_Programmer_CLI to flash `app-trusted.bin` to 0x70100000
5. Configure the boot switches to both be low (if on the Disco kit) and click the reset button to boot into your code

# Memory layout

**Bootloader**

I have yet to test updating (if that's even supported for the N6), but this is how it is so far
```x
/* DFU must be at least one page (4K) more than Active */
MEMORY
{
  RAM                         (rwx) : ORIGIN = 0x34100000, LENGTH = 513K
  FLASH                             : ORIGIN = 0x34180400, LENGTH = 507K
  BOOTLOADER_STATE                  : ORIGIN = 0x341ff000, LENGTH = 4K
  BOOTLOADER                        : ORIGIN = 0x70000000, LENGTH = 1024K
  ACTIVE                            : ORIGIN = 0x70100400, LENGTH = 2044K
  DFU                               : ORIGIN = 0x70300400, LENGTH = 2048K
}
SECTIONS
{
  .application_flash (NOLOAD) :
    {
      *(.application_flash);
    } > ACTIVE
}

__bootloader_flash_address = ORIGIN(FLASH);
__bootloader_active_address = ORIGIN(ACTIVE);

__bootloader_state_start = ORIGIN(BOOTLOADER_STATE) - ORIGIN(FLASH);
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE) - ORIGIN(FLASH);

__bootloader_active_start = ORIGIN(ACTIVE) - ORIGIN(FLASH);
__bootloader_active_end = ORIGIN(ACTIVE) + LENGTH(ACTIVE) - ORIGIN(FLASH);

__bootloader_dfu_start = ORIGIN(DFU) - ORIGIN(DFU);
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU) - ORIGIN(DFU);
```

**Firmware**
```x
MEMORY
{
  BACKUP_RAM      : ORIGIN = 0x3C000000,   LENGTH = 8K
  RAM             : ORIGIN = 0x34100000,   LENGTH = 2816K
  FLASH           : ORIGIN = 0x70100400,   LENGTH = 128M
  XSPI_RAM        : ORIGIN = 0x90000000,   LENGTH = 32M
}
SECTIONS
{
  .external_ram (NOLOAD) :
    {
      *(.external_ram);
    } > XSPI_RAM

  .external_flash (NOLOAD) :
    {
      *(.external_flash);
    } > XSPI_FLASH
}

__backup_ram = ORIGIN(BACKUP_RAM);
```

# Bootloader snippets

The bootloader is built using embassy-boot-stm32. Not all steps may be relevant, but this is what the bootloader does:
1. Enable debugging mode
```Rust
unsafe {
    // Open the debug access port to the Cortex-M55
    BSEC.as_ptr()
        .cast::<u32>()
        .byte_add(0xE90)
        .write(0xB451_B400);
    // Enable the non-secure/secure debug
    BSEC.as_ptr()
        .cast::<u32>()
        .byte_add(0xE8C)
        .write(0xB451_B400);
};
```
2. Set the vector table (otherwise it can hardfault)
```Rust
let mut core_peri = unsafe { cortex_m::Peripherals::steal() };
unsafe { core_peri.SCB.vtor.write(0x34180400) }; // __bootloader_flash_address
```
3. Configure the clocks
```Rust
// Configure the system without any PLL, dropping to 64 MHz to allow the firmware to configure
// its own clocks
let mut config = Config::default();
// The initial settings must match what PLL1 is already set to. We'll disable it later
config.rcc.pll1 = Some(rcc::Pll::Oscillator {
    source: Pllsel::HSI,
    divm: Plldivm::DIV4,
    fractional: 0,
    divn: 75,
    divp1: Pllpdiv::DIV1,
    divp2: Pllpdiv::DIV1,
});
config.rcc.sys = SysClk::Hsi;
config.rcc.cpu = CpuClk::Hsi;
// Place the flash on the HSI bus to allow the firmware to configure the clocks without causing issues
config.rcc.mux.xspi2sel = Xspisel::PER;
let p = embassy_stm32::init(config);
// Disable PLL1
RCC.pllcfgr3(0).modify(|w| w.set_pllpdiven(false));
RCC.ccr().write(|w| w.set_pllonc(0, true));
while RCC.sr().read().pllrdy(0) {} // wait till disabled
RCC.pllcfgr1(0).modify(|w| w.set_pllbyp(false)); // clear bypass mode
```
4. Initialize external flash in memory map mode
5. Enable all RAM banks
6. Load firmware
```Rust
core_peri.SCB.invalidate_icache(); // Clear instruction cache
unsafe {
    core_peri.SCB.vtor.write(0x70100400); // Set new vector table (__bootloader_active_address)
    cortex_m::asm::bootload(0x70100400 as *const u32); // Jump
}
```

# Firmware snippets

Nothing too special here, I just reconfigure the clocks for peripherals and speed

```Rust
let mut config = Config::default();
// Configure the clocks as desired...
let mut periph = unsafe { Peripherals::steal() };
rcc::reinit(config, &mut periph.RCC);
```