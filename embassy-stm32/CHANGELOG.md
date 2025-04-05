# Changelog for embassy-stm32

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased
- Modify BufferedUart initialization to take pins before interrupts ([#3983](https://github.com/embassy-rs/embassy/pull/3983))

## 0.2.0 - 2025-01-10

Starting 2025 strong with a release packed with new, exciting good stuff! 🚀

### New chips

This release adds support for many newly-released STM32 chips.

- STM32H7[RS] "bootflash line" ([#2898](https://github.com/embassy-rs/embassy/pull/2898))
- STM32U0 ([#2809](https://github.com/embassy-rs/embassy/pull/2809) [#2813](https://github.com/embassy-rs/embassy/pull/2813))
- STM32H5[23] ([#2892](https://github.com/embassy-rs/embassy/pull/2892))
- STM32U5[FG] ([#2892](https://github.com/embassy-rs/embassy/pull/2892))
- STM32WBA5[045] ([#2892](https://github.com/embassy-rs/embassy/pull/2892))

### Simpler APIs with less generics

Many HAL APIs have been simplified thanks to reducing the amount of generic parameters. This helps with creating arrays of pins or peripherals, and for calling the same code with different pins/peripherals without incurring in code size penalties d

For GPIO, the pins have been eliminated. `Output<'_, PA4>` is now `Output<'_>`.

For peripherals, both pins and DMA channels have been eliminated. Peripherals now have a "mode" generic param that specifies whether it's capable of async operation. For example, `I2c<'_, I2C2, NoDma, NoDma>` is now `I2c<'_, Blocking>` and `I2c<'_, I2C2, DMA2_CH1, DMA2_CH2>` is now `I2c<'_, Async>`.

- Removed DMA channel generic params for UART ([#2821](https://github.com/embassy-rs/embassy/pull/2821)), I2C ([#2820](https://github.com/embassy-rs/embassy/pull/2820)), SPI ([#2819](https://github.com/embassy-rs/embassy/pull/2819)), QSPI ([#2982](https://github.com/embassy-rs/embassy/pull/2982)), OSPI ([#2941](https://github.com/embassy-rs/embassy/pull/2941)).
- Removed peripheral generic params for GPIO ([#2471](https://github.com/embassy-rs/embassy/pull/2471)), UART ([#2836](https://github.com/embassy-rs/embassy/pull/2836)), I2C ([#2974](https://github.com/embassy-rs/embassy/pull/2974)), SPI ([#2835](https://github.com/embassy-rs/embassy/pull/2835))
- Remove generics in CAN ([#3012](https://github.com/embassy-rs/embassy/pull/3012), [#3020](https://github.com/embassy-rs/embassy/pull/3020), [#3032](https://github.com/embassy-rs/embassy/pull/3032), [#3033](https://github.com/embassy-rs/embassy/pull/3033))

### More complete and consistent RCC

RCC support has been vastly expanded and improved.
- The API is now consistent across all STM32 families. Previously in some families you'd configure the desired target frequencies for `sysclk` and the buses and `embassy-stm32` would try to calculate dividers and muxes to hit them as close as possible. This has proved to be intractable in the general case and hard to extend to more exotic RCC configurations. So, we have standardized on an API where the user specifies the settings for dividers and muxes directly. It's lower level but gices more control to the user, supports all edge case exotic configurations, and makes it easier to translate a configuration from the STM32CubeMX tool. ([Tracking issue](https://github.com/embassy-rs/embassy/issues/2515). [#2624](https://github.com/embassy-rs/embassy/pull/2624). F0, F1 [#2564](https://github.com/embassy-rs/embassy/pull/2564), F3 [#2560](https://github.com/embassy-rs/embassy/pull/2560), U5 [#2617](https://github.com/embassy-rs/embassy/pull/2617), [#3514](https://github.com/embassy-rs/embassy/pull/3514), [#3513](https://github.com/embassy-rs/embassy/pull/3513), G4 [#2579](https://github.com/embassy-rs/embassy/pull/2579), [#2618](https://github.com/embassy-rs/embassy/pull/2618), WBA [#2520](https://github.com/embassy-rs/embassy/pull/2520), G0, C0 ([#2656](https://github.com/embassy-rs/embassy/pull/2656)).
- Added support for configuring all per-peripheral clock muxes (CCIPRx, DCKCFGRx registers) in `config.rcc.mux`. This was previously handled in an ad-hoc way in some drivers (e.g. USB) and not at all in others (causing e.g. wrong SPI frequency) ([#2521](https://github.com/embassy-rs/embassy/pull/2521), [#2583](https://github.com/embassy-rs/embassy/pull/2583), [#2634](https://github.com/embassy-rs/embassy/pull/2634), [#2626](https://github.com/embassy-rs/embassy/pull/2626), [#2815](https://github.com/embassy-rs/embassy/pull/2815), [#2517](https://github.com/embassy-rs/embassy/pull/2517)).
- Switch to a safe configuration before configuring RCC. This helps avoid crashes when RCC has been already configured previously (for example by a bootloader). (F2, F4, F7 [#2829](https://github.com/embassy-rs/embassy/pull/2829), C0, F0, F1, F3, G0, G4, H5, H7[#3008](https://github.com/embassy-rs/embassy/pull/3008))
- Some new nice features:
    - Expose RCC enable and disable in public API. ([#2807](https://github.com/embassy-rs/embassy/pull/2807))
    - Add `unchecked-overclocking` feature that disables all asserts, allowing running RCC out of spec. ([#3574](https://github.com/embassy-rs/embassy/pull/3574))
- Many fixes:
    - Workaround H5 errata that accidentally clears RAM on backup domain reset. ([#2616](https://github.com/embassy-rs/embassy/pull/2616))
    - Reset RTC on L0 ([#2597](https://github.com/embassy-rs/embassy/pull/2597))
    - Fix H7 to use correct unit in vco clock check ([#2537](https://github.com/embassy-rs/embassy/pull/2537))
    - Fix incorrect D1CPRE max for STM32H7 RM0468 ([#2518](https://github.com/embassy-rs/embassy/pull/2518))
    - WBA's high speed external clock has to run at 32 MHz ([#2511](https://github.com/embassy-rs/embassy/pull/2511))
    - Take into account clock propagation delay to peripherals after enabling a clock. ([#2677](https://github.com/embassy-rs/embassy/pull/2677))
    - Fix crash caused by using higher MSI range as sysclk on STM32WL ([#2786](https://github.com/embassy-rs/embassy/pull/2786))
    - fix using HSI48 as SYSCLK on F0 devices with CRS ([#3652](https://github.com/embassy-rs/embassy/pull/3652))
    - compute LSE and LSI frequency for STM32L and STM32U0 series ([#3554](https://github.com/embassy-rs/embassy/pull/3554))
    - Add support for LSESYS, used to pass LSE clock to peripherals ([#3518](https://github.com/embassy-rs/embassy/pull/3518))
    - H5: LSE low drive mode is not functional ([#2738](https://github.com/embassy-rs/embassy/pull/2738))

### New peripheral drivers

- Dual-core support. First core initializes RCC and writes a struct into shared memory that the second core uses, ensuring no conflicts. ([#3158](https://github.com/embassy-rs/embassy/pull/3158), [#3263](https://github.com/embassy-rs/embassy/pull/3263), [#3687](https://github.com/embassy-rs/embassy/pull/3687))
- USB Type-C/USB Power Delivery Interface (UCPD) ([#2652](https://github.com/embassy-rs/embassy/pull/2652), [#2683](https://github.com/embassy-rs/embassy/pull/2683), [#2701](https://github.com/embassy-rs/embassy/pull/2701), [#2925](https://github.com/embassy-rs/embassy/pull/2925), [#3084](https://github.com/embassy-rs/embassy/pull/3084), [#3271](https://github.com/embassy-rs/embassy/pull/3271), [#3678](https://github.com/embassy-rs/embassy/pull/3678), [#3714](https://github.com/embassy-rs/embassy/pull/3714))
- Touch sensing controller (TSC) ([#2853](https://github.com/embassy-rs/embassy/pull/2853), [#3111](https://github.com/embassy-rs/embassy/pull/3111), [#3163](https://github.com/embassy-rs/embassy/pull/3163), [#3274](https://github.com/embassy-rs/embassy/pull/3274))
- Display Serial Interface (DSI) [#2903](https://github.com/embassy-rs/embassy/pull/2903), ([#3082](https://github.com/embassy-rs/embassy/pull/3082))
- LCD/TFT Display Controller (LTDC) ([#3126](https://github.com/embassy-rs/embassy/pull/3126), [#3458](https://github.com/embassy-rs/embassy/pull/3458))
- SPDIF receiver (SPDIFRX) ([#3280](https://github.com/embassy-rs/embassy/pull/3280))
- CORDIC math accelerator ([#2697](https://github.com/embassy-rs/embassy/pull/2697))
- Digital Temperature Sensor (DTS) ([#3717](https://github.com/embassy-rs/embassy/pull/3717))
- HMAC accelerator ([#2565](https://github.com/embassy-rs/embassy/pull/2565))
- Hash accelerator ([#2528](https://github.com/embassy-rs/embassy/pull/2528))
- Crypto accelerator ([#2619](https://github.com/embassy-rs/embassy/pull/2619), [#2691](https://github.com/embassy-rs/embassy/pull/2691))
- Semaphore (HSEM) ([#2777](https://github.com/embassy-rs/embassy/pull/2777), [#3161](https://github.com/embassy-rs/embassy/pull/3161))

### Improvements to existing drivers

GPIO:
- Generate singletons only for pins that actually exist. ([#3738](https://github.com/embassy-rs/embassy/pull/3738))
- Add `set_as_analog` to Flex ([#3017](https://github.com/embassy-rs/embassy/pull/3017))
- Add `embedded-hal` v0.2 `InputPin` impls for `OutputOpenDrain`. ([#2716](https://github.com/embassy-rs/embassy/pull/2716))
- Add a config option to make the VDDIO2 supply line valid ([#2737](https://github.com/embassy-rs/embassy/pull/2737))
- Refactor AfType ([#3031](https://github.com/embassy-rs/embassy/pull/3031))
- Gpiov1: Do not call set_speed for AFType::Input ([#2996](https://github.com/embassy-rs/embassy/pull/2996))

UART: 
- Add embedded-io impls ([#2739](https://github.com/embassy-rs/embassy/pull/2739))
- Add support for changing baud rate ([#3512](https://github.com/embassy-rs/embassy/pull/3512))
- Add split_ref ([#3500](https://github.com/embassy-rs/embassy/pull/3500))
- Add data bit selection ([#3595](https://github.com/embassy-rs/embassy/pull/3595))
- Add RX Pull configuration option ([#3415](https://github.com/embassy-rs/embassy/pull/3415))
- Add async flush ([#3379](https://github.com/embassy-rs/embassy/pull/3379))
- Add support for sending breaks ([#3286](https://github.com/embassy-rs/embassy/pull/3286))
- Disconnect pins on drop ([#3006](https://github.com/embassy-rs/embassy/pull/3006))
- Half-duplex improvements
    - Add half-duplex for all USART versions ([#2833](https://github.com/embassy-rs/embassy/pull/2833))
    - configurable readback for half-duplex. ([#3679](https://github.com/embassy-rs/embassy/pull/3679))
    - Convert uart half_duplex to use user configurable IO ([#3233](https://github.com/embassy-rs/embassy/pull/3233))
    - Fix uart::flush with FIFO at Half-Duplex mode ([#2895](https://github.com/embassy-rs/embassy/pull/2895))
    - Fix Half-Duplex sequential reads and writes ([#3089](https://github.com/embassy-rs/embassy/pull/3089))
    - disable transmitter during during half-duplex flush ([#3299](https://github.com/embassy-rs/embassy/pull/3299))
- Buffered UART improvements
    - Add embedded-io ReadReady impls ([#3179](https://github.com/embassy-rs/embassy/pull/3179), [#3451](https://github.com/embassy-rs/embassy/pull/3451))
    - Add constructors for RS485 ([#3441](https://github.com/embassy-rs/embassy/pull/3441))
    - Fix RingBufferedUartRx hard-resetting DMA after initial error ([#3356](https://github.com/embassy-rs/embassy/pull/3356))
    - Don't teardown during reconfigure ([#2989](https://github.com/embassy-rs/embassy/pull/2989))
    - Wake receive task for each received byte ([#2722](https://github.com/embassy-rs/embassy/pull/2722))
    - Fix dma and idle line detection in ringbuffereduartrx ([#3319](https://github.com/embassy-rs/embassy/pull/3319))

SPI: 
- Add MISO pullup configuration option ([#2943](https://github.com/embassy-rs/embassy/pull/2943))
- Add slew rate configuration options ([#3669](https://github.com/embassy-rs/embassy/pull/3669))
- Fix blocking_write on nosck spi. ([#3035](https://github.com/embassy-rs/embassy/pull/3035))
- Restrict txonly_nosck to SPIv1, it hangs in other versions. ([#3028](https://github.com/embassy-rs/embassy/pull/3028))
- Fix non-u8 word sizes. ([#3363](https://github.com/embassy-rs/embassy/pull/3363))
- Issue correct DMA word length when reading to prevent hang. ([#3362](https://github.com/embassy-rs/embassy/pull/3362))
- Add proper rxonly support for spi_v3 and force tx dma stream requirements. ([#3007](https://github.com/embassy-rs/embassy/pull/3007))

I2C:
- Implement asynchronous transactions ([#2742](https://github.com/embassy-rs/embassy/pull/2742))
- Implement blocking transactions ([#2713](https://github.com/embassy-rs/embassy/pull/2713))
- Disconnect pins on drop ([#3006](https://github.com/embassy-rs/embassy/pull/3006))
- Ensure bus is free before master-write operation ([#3104](https://github.com/embassy-rs/embassy/pull/3104))
- Add workaround for STM32 i2cv1 errata ([#2887](https://github.com/embassy-rs/embassy/pull/2887))
- Fix disabling pullup accidentally enabling pulldown ([#3410](https://github.com/embassy-rs/embassy/pull/3410))

Flash:
- Add L5 support ([#3423](https://github.com/embassy-rs/embassy/pull/3423))
- Add H5 support ([#3305](https://github.com/embassy-rs/embassy/pull/3305))
- add F2 support ([#3303](https://github.com/embassy-rs/embassy/pull/3303))
- Add U5 support ([#2591](https://github.com/embassy-rs/embassy/pull/2591), [#2792](https://github.com/embassy-rs/embassy/pull/2792))
- Add H50x support ([#2600](https://github.com/embassy-rs/embassy/pull/2600), [#2808](https://github.com/embassy-rs/embassy/pull/2808))
- Fix flash erase on F3 ([#3744](https://github.com/embassy-rs/embassy/pull/3744))
- Support G0 second flash bank ([#3711](https://github.com/embassy-rs/embassy/pull/3711))
- F1, F3: wait for BSY flag to clear before flashing ([#3217](https://github.com/embassy-rs/embassy/pull/3217))
- H7: enhance resilience to program sequence errors (pgserr) ([#2539](https://github.com/embassy-rs/embassy/pull/2539))

ADC:
- Add `AnyAdcChannel` type. You can obtain it from a pin with `.degrade_adc()`. Useful for making arrays of ADC pins. ([#2985](https://github.com/embassy-rs/embassy/pull/2985))
- Add L0 support ([#2544](https://github.com/embassy-rs/embassy/pull/2544))
- Add U5 support ([#3688](https://github.com/embassy-rs/embassy/pull/3688))
- Add H5 support ([#2613](https://github.com/embassy-rs/embassy/pull/2613), [#3557](https://github.com/embassy-rs/embassy/pull/3557))
- Add G4 async support ([#3566](https://github.com/embassy-rs/embassy/pull/3566))
- Add G4 support for calibrating differential inputs ([#3735](https://github.com/embassy-rs/embassy/pull/3735))
- Add oversampling and differential support for G4 ([#3169](https://github.com/embassy-rs/embassy/pull/3169))
- Add DMA support for ADC v2 ([#3116](https://github.com/embassy-rs/embassy/pull/3116))
- Add DMA support for ADC v3 and v4 ([#3128](https://github.com/embassy-rs/embassy/pull/3128))
- Unify naming `blocking_read` for blocking, `read` for async. ([#3148](https://github.com/embassy-rs/embassy/pull/3148))
- Fix channel count for the STM32G4 ADCs. ([#2828](https://github.com/embassy-rs/embassy/pull/2828))
- Fix blocking_delay_us() overflowing when sys freq is high ([#2825](https://github.com/embassy-rs/embassy/pull/2825))
- Remove need for taking a `Delay` impl. ([#2797](https://github.com/embassy-rs/embassy/pull/2797))
- H5: set OR.OP0 to 1 when ADCx_INP0 is selected, per RM ([#2776](https://github.com/embassy-rs/embassy/pull/2776))
- Add oversampling support ([#3124](https://github.com/embassy-rs/embassy/pull/3124))
- Adc averaging support for ADC v4. ([#3110](https://github.com/embassy-rs/embassy/pull/3110))
- F2 ADC fixes ([#2513](https://github.com/embassy-rs/embassy/pull/2513))

DAC:
- Fix new_internal not setting mode as documented ([#2886](https://github.com/embassy-rs/embassy/pull/2886))

OPAMP:
- Add missing opamp external outputs for STM32G4 ([#3636](https://github.com/embassy-rs/embassy/pull/3636))
- Add extra lifetime to opamp-using structs ([#3207](https://github.com/embassy-rs/embassy/pull/3207))
- Make OpAmp usable in follower configuration for internal DAC channel ([#3021](https://github.com/embassy-rs/embassy/pull/3021))

CAN:
- Add FDCAN support. ([#2475](https://github.com/embassy-rs/embassy/pull/2475), [#2571](https://github.com/embassy-rs/embassy/pull/2571), [#2623](https://github.com/embassy-rs/embassy/pull/2623), [#2631](https://github.com/embassy-rs/embassy/pull/2631), [#2635](https://github.com/embassy-rs/embassy/pull/2635), [#2637](https://github.com/embassy-rs/embassy/pull/2637), [#2645](https://github.com/embassy-rs/embassy/pull/2645), [#2647](https://github.com/embassy-rs/embassy/pull/2647), [#2658](https://github.com/embassy-rs/embassy/pull/2658), [#2703](https://github.com/embassy-rs/embassy/pull/2703), [#3364](https://github.com/embassy-rs/embassy/pull/3364))
- Simplify BXCAN API, make BXCAN and FDCAN APIs consistent. ([#2760](https://github.com/embassy-rs/embassy/pull/2760), [#2693](https://github.com/embassy-rs/embassy/pull/2693), [#2744](https://github.com/embassy-rs/embassy/pull/2744))
- Add buffered mode support ([#2588](https://github.com/embassy-rs/embassy/pull/2588))
- Add support for modifying the receiver filters from `BufferedCan`, `CanRx`, and `BufferedCanRx` ([#3733](https://github.com/embassy-rs/embassy/pull/3733))
- Add support for optional FIFO scheduling for outgoing frames ([#2988](https://github.com/embassy-rs/embassy/pull/2988))
- fdcan: Properties for common runtime get/set operations ([#2840](https://github.com/embassy-rs/embassy/pull/2840))
- fdcan: implement bus-off recovery ([#2832](https://github.com/embassy-rs/embassy/pull/2832))
- Add BXCAN sleep/wakeup functionality ([#2854](https://github.com/embassy-rs/embassy/pull/2854))
- Fix BXCAN hangs ([#3468](https://github.com/embassy-rs/embassy/pull/3468))
- add RTR flag if it is remote frame ([#3421](https://github.com/embassy-rs/embassy/pull/3421))
- Fix log storm when no CAN is connected ([#3284](https://github.com/embassy-rs/embassy/pull/3284))
- Fix error handling ([#2850](https://github.com/embassy-rs/embassy/pull/2850))
- Give CAN a kick when writing into TX buffer via sender. ([#2646](https://github.com/embassy-rs/embassy/pull/2646))
- Preseve the RTR flag in messages. ([#2745](https://github.com/embassy-rs/embassy/pull/2745))

FMC:
- Add 13bit address sdram constructors ([#3189](https://github.com/embassy-rs/embassy/pull/3189))

xSPI:
- Add OCTOSPI support ([#2672](https://github.com/embassy-rs/embassy/pull/2672))
- Add OCTOSPIM support ([#3102](https://github.com/embassy-rs/embassy/pull/3102))
- Add HEXADECASPI support ([#3667](https://github.com/embassy-rs/embassy/pull/3667))
- Add memory mapping support for QSPI ([#3725](https://github.com/embassy-rs/embassy/pull/3725))
- Add memory mapping support for OCTOSPI ([#3456](https://github.com/embassy-rs/embassy/pull/3456))
- Add async support for QSPI ([#3475](https://github.com/embassy-rs/embassy/pull/3475))
- Fix QSPI synchronous read operation hangs when FIFO is not full ([#3724](https://github.com/embassy-rs/embassy/pull/3724))
- Stick to `blocking_*` naming convention for QSPI, OSPI ([#3661](https://github.com/embassy-rs/embassy/pull/3661))

SDMMC:
- Add `block-device-driver` impl for use with `embedded-fatfs` ([#2607](https://github.com/embassy-rs/embassy/pull/2607))
- Allow cmd block to be passed in for sdmmc dma transfers ([#3188](https://github.com/embassy-rs/embassy/pull/3188))

ETH:
- Fix reception of multicast packets ([#3488](https://github.com/embassy-rs/embassy/pull/3488), [#3707](https://github.com/embassy-rs/embassy/pull/3707))
- Add support for executing custom SMI commands ([#3355](https://github.com/embassy-rs/embassy/pull/3355))
- Add support for MII interface ([#2465](https://github.com/embassy-rs/embassy/pull/2465))

USB:
- Assert correct clock on init. ([#2711](https://github.com/embassy-rs/embassy/pull/2711))
- Set PWR_CR2 USV on STM32L4 ([#2605](https://github.com/embassy-rs/embassy/pull/2605))
- USBD driver improvements:
    - Add ISO endpoint support ([#3314](https://github.com/embassy-rs/embassy/pull/3314))
    - Add support for L1. ([#2452](https://github.com/embassy-rs/embassy/pull/2452))
    - set USB initialization delay to 1µs ([#3700](https://github.com/embassy-rs/embassy/pull/3700))
- OTG driver improvements:
    - Add ISO endpoint support ([#3314](https://github.com/embassy-rs/embassy/pull/3314))
    - Add support for U595, U5A5 ([#3613](https://github.com/embassy-rs/embassy/pull/3613))
    - Add support for STM32H7R/S ([#3337](https://github.com/embassy-rs/embassy/pull/3337))
    - Add support for full-speed ULPI mode ([#3281](https://github.com/embassy-rs/embassy/pull/3281))
    - Make max EP count configurable ([#2881](https://github.com/embassy-rs/embassy/pull/2881))
    - fix corruption in CONTROL OUT transfers in stm32f4. ([#3565](https://github.com/embassy-rs/embassy/pull/3565))
    - Extract Synopsys USB OTG driver to a separate crate ([#2871](https://github.com/embassy-rs/embassy/pull/2871))
    - Add critical sections to avoid USB OTG corruption Errata ([#2823](https://github.com/embassy-rs/embassy/pull/2823))
    - Fix support for OTG_HS in FS mode. ([#2805](https://github.com/embassy-rs/embassy/pull/2805))

I2S:
- Add SPIv3 support. ([#2992](https://github.com/embassy-rs/embassy/pull/2992))
- Add full-duplex support. ([#2992](https://github.com/embassy-rs/embassy/pull/2992))
- Add I2S ringbuffered DMA support ([#3023](https://github.com/embassy-rs/embassy/pull/3023))
- Fix STM32F4 I2S clock calculations ([#3716](https://github.com/embassy-rs/embassy/pull/3716))

SAI:
- Add a function that waits for any SAI/ringbuffer write error ([#3545](https://github.com/embassy-rs/embassy/pull/3545))
- Disallow start without an initial write ([#3541](https://github.com/embassy-rs/embassy/pull/3541))
- Flush FIFO on init and disable ([#3538](https://github.com/embassy-rs/embassy/pull/3538))
- Fix MCKDIV for SAI v3/v4 ([#2710](https://github.com/embassy-rs/embassy/pull/2710))
- Pull down clock and data lines in receive mode ([#3326](https://github.com/embassy-rs/embassy/pull/3326))
- Add function to check if SAI is muted ([#3282](https://github.com/embassy-rs/embassy/pull/3282))

Low-power support:
- Update `embassy-executor` to v0.7.
- Add support for U0 ([#3556](https://github.com/embassy-rs/embassy/pull/3556))
- Add support for U5 ([#3496](https://github.com/embassy-rs/embassy/pull/3496))
- Add support for H5 ([#2877](https://github.com/embassy-rs/embassy/pull/2877))
- Add support for L4 ([#3213](https://github.com/embassy-rs/embassy/pull/3213))
- Fix low-power EXTI IRQ handler dropped edges ([#3404](https://github.com/embassy-rs/embassy/pull/3404))
- Fix alarms not triggering in some cases ([#3592](https://github.com/embassy-rs/embassy/pull/3592))

Timer:
- Add Input Capture high-level driver ([#2912](https://github.com/embassy-rs/embassy/pull/2912))
- Add PWM Input high-level driver ([#3014](https://github.com/embassy-rs/embassy/pull/3014))
- Add support for splitting `SimplePwm` into channels ([#3317](https://github.com/embassy-rs/embassy/pull/3317))
- Fix `SimplePwm` not enabling output pin in some stm32 families ([#2670](https://github.com/embassy-rs/embassy/pull/2670))
- Add LPTIM low-level driver. ([#3310](https://github.com/embassy-rs/embassy/pull/3310))
- Low-level TIM driver improvements:
    - Simplify traits, convert from trait methods to struct. ([#2728](https://github.com/embassy-rs/embassy/pull/2728))
    - Add `low_level::Timer::get_clock_frequency()` ([#2908](https://github.com/embassy-rs/embassy/pull/2908))
    - Fix 32bit timer off by one ARR error ([#2876](https://github.com/embassy-rs/embassy/pull/2876))
    - Avoid max_compare_value >= u16::MAX ([#3549](https://github.com/embassy-rs/embassy/pull/3549))

DMA:
- Add `AnyChannel` type. Similar to `AnyPin`, it allows representing any DMA channel at runtime without needing generics. ([#2606](https://github.com/embassy-rs/embassy/pull/2606))
, Add support for BDMA on H7 ([#2606](https://github.com/embassy-rs/embassy/pull/2606))
- Add async `stop()` function to BDMA, DMA ([#2757](https://github.com/embassy-rs/embassy/pull/2757))
- Add configuration option for DMA Request Priority ([#2680](https://github.com/embassy-rs/embassy/pull/2680))
- Rewrite DMA ringbuffers ([#3336](https://github.com/embassy-rs/embassy/pull/3336))
- Enable half transfer IRQ when constructing a ReadableDmaRingBuffer ([#3093](https://github.com/embassy-rs/embassy/pull/3093))
- Right-align `write_immediate()` in ring buffers ([#3588](https://github.com/embassy-rs/embassy/pull/3588))

`embassy-time` driver:
- Update to `embassy-time` v0.4, `embassy-time-driver` v0.2. ([#3593](https://github.com/embassy-rs/embassy/pull/3593))
- Change preference order of `time-driver-any` to pick less-featureful timers first. ([#2570](https://github.com/embassy-rs/embassy/pull/2570))
- Allow using more TIMx timers for the time driver  of TIM1 ([#2570](https://github.com/embassy-rs/embassy/pull/2570), [#2614](https://github.com/embassy-rs/embassy/pull/2614))
- Correctly gate `time` feature of embassy-embedded-hal in embassy-stm32 ([#3359](https://github.com/embassy-rs/embassy/pull/3359))
- adds timer-driver for tim21 and tim22 (on L0) ([#2450](https://github.com/embassy-rs/embassy/pull/2450))

WDG:
- Allow higher PSC value for iwdg_v3 ... ([#2628](https://github.com/embassy-rs/embassy/pull/2628))

Misc:
- Allow `bind_interrupts!` to accept conditional compilation attrs ([#3444](https://github.com/embassy-rs/embassy/pull/3444))

## 0.1.0 - 2024-01-12

First release.
