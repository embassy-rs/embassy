#![no_std]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use core::slice;

use cyw43::SpiBusCyw43;
use embassy_rp::Peri;
use embassy_rp::clocks::clk_sys_freq;
use embassy_rp::dma::Channel;
use embassy_rp::gpio::{Drive, Level, Output, Pull, SlewRate};
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{Common, Config, Direction, Instance, Irq, PioPin, ShiftDirection, StateMachine};
use fixed::FixedU32;
use fixed::types::extra::U8;

/// SPI comms driven by PIO.
pub struct PioSpi<'d, PIO: Instance, const SM: usize, DMA: Channel> {
    cs: Output<'d>,
    sm: StateMachine<'d, PIO, SM>,
    irq: Irq<'d, PIO, 0>,
    dma: Peri<'d, DMA>,
    wrap_target: u8,
}

/// Clock divider used for most applications
/// With default core clock configuration:
/// RP2350: 150Mhz / 2 = 75Mhz pio clock -> 37.5Mhz GSPI clock
/// RP2040: 133Mhz / 2 = 66.5Mhz pio clock -> 33.25Mhz GSPI clock
pub const DEFAULT_CLOCK_DIVIDER: FixedU32<U8> = FixedU32::from_bits(0x0200);

/// Clock divider used to overclock the cyw43
/// With default core clock configuration:
/// RP2350: 150Mhz / 1 = 150Mhz pio clock -> 75Mhz GSPI clock (50% greater that manufacturer
/// recommended 50Mhz)
/// RP2040: 133Mhz / 1 = 133Mhz pio clock -> 66.5Mhz GSPI clock (33% greater that manufacturer
/// recommended 50Mhz)
pub const OVERCLOCK_CLOCK_DIVIDER: FixedU32<U8> = FixedU32::from_bits(0x0100);

/// Clock divider used with the RM2
/// With default core clock configuration:
/// RP2350: 150Mhz / 3 = 50Mhz pio clock -> 25Mhz GSPI clock
/// RP2040: 133Mhz / 3 = 44.33Mhz pio clock -> 22.16Mhz GSPI clock
pub const RM2_CLOCK_DIVIDER: FixedU32<U8> = FixedU32::from_bits(0x0300);

impl<'d, PIO, const SM: usize, DMA> PioSpi<'d, PIO, SM, DMA>
where
    DMA: Channel,
    PIO: Instance,
{
    /// Create a new instance of PioSpi.
    pub fn new(
        common: &mut Common<'d, PIO>,
        mut sm: StateMachine<'d, PIO, SM>,
        clock_divider: FixedU32<U8>,
        irq: Irq<'d, PIO, 0>,
        cs: Output<'d>,
        dio: Peri<'d, impl PioPin>,
        clk: Peri<'d, impl PioPin>,
        dma: Peri<'d, DMA>,
    ) -> Self {
        let effective_pio_frequency = (clk_sys_freq() as f32 / clock_divider.to_num::<f32>()) as u32;

        #[cfg(feature = "defmt")]
        defmt::trace!("Effective pio frequency: {}Hz", effective_pio_frequency);

        // Non-integer pio clock dividers are achieved by introducing clock jitter resulting in a
        // combination of long and short cycles. The long and short cycles average to achieve the
        // requested clock speed.
        // This can be a problem for peripherals that expect a consistent clock / have a clock
        // speed upper bound that is violated by the short cycles. The cyw43 seems to handle the
        // jitter well, but we emit a warning to recommend an integer divider anyway.
        if clock_divider.frac() != FixedU32::<U8>::ZERO {
            #[cfg(feature = "defmt")]
            defmt::trace!(
                "Configured clock divider is not a whole number. Some clock cycles may violate the maximum recommended GSPI speed. Use at your own risk."
            );
        }

        // Different pio programs must be used for different pio clock speeds.
        // The programs used below are based on the pico SDK: https://github.com/raspberrypi/pico-sdk/blob/master/src/rp2_common/pico_cyw43_driver/cyw43_bus_pio_spi.pio
        // The clock speed cutoff for each program has been determined experimentally:
        // > 100Mhz -> Overclock program
        // [75Mhz, 100Mhz] -> High speed program
        // [0, 75Mhz) -> Low speed program
        let loaded_program = if effective_pio_frequency > 100_000_000 {
            // Any frequency > 100Mhz is overclocking the chip (manufacturer recommends max 50Mhz GSPI
            // clock)
            // Example:
            // * RP2040 @ 133Mhz (stock) with OVERCLOCK_CLOCK_DIVIDER (133MHz)
            #[cfg(feature = "defmt")]
            defmt::trace!(
                "Configured clock divider results in a GSPI frequency greater than the manufacturer recommendation (50Mhz). Use at your own risk."
            );

            let overclock_program = pio_asm!(
                ".side_set 1"

                ".wrap_target"
                // write out x-1 bits
                "lp:"
                "out pins, 1    side 0"
                "jmp x-- lp     side 1"
                // switch directions
                "set pindirs, 0 side 0"
                "nop            side 1"
                "nop            side 0"
                // read in y-1 bits
                "lp2:"
                "in pins, 1     side 1"
                "jmp y-- lp2    side 0"

                // wait for event and irq host
                "wait 1 pin 0   side 0"
                "irq 0          side 0"

                ".wrap"
            );
            common.load_program(&overclock_program.program)
        } else if effective_pio_frequency >= 75_000_000 {
            // Experimentally determined cutoff.
            // Notably includes the stock RP2350 configured with clk_div of 2 (150Mhz base clock / 2 = 75Mhz)
            // but does not include stock RP2040 configured with clk_div of 2 (133Mhz base clock / 2 = 66.5Mhz)
            // Example:
            // * RP2350 @ 150Mhz (stock) with DEFAULT_CLOCK_DIVIDER (75Mhz)
            // * RP2XXX @ 200Mhz with DEFAULT_CLOCK_DIVIDER (100Mhz)
            #[cfg(feature = "defmt")]
            defmt::trace!("Using high speed pio program.");
            let high_speed_program = pio_asm!(
                ".side_set 1"

                ".wrap_target"
                // write out x-1 bits
                "lp:"
                "out pins, 1    side 0"
                "jmp x-- lp     side 1"
                // switch directions
                "set pindirs, 0 side 0"
                "nop            side 1"
                // read in y-1 bits
                "lp2:"
                "in pins, 1     side 0"
                "jmp y-- lp2    side 1"

                // wait for event and irq host
                "wait 1 pin 0   side 0"
                "irq 0          side 0"

                ".wrap"
            );
            common.load_program(&high_speed_program.program)
        } else {
            // Low speed
            // Examples:
            // * RP2040 @ 133Mhz (stock) with DEFAULT_CLOCK_DIVIDER (66.5Mhz)
            // * RP2040 @ 133Mhz (stock) with RM2_CLOCK_DIVIDER (44.3Mhz)
            // * RP2350 @ 150Mhz (stock) with RM2_CLOCK_DIVIDER (50Mhz)
            #[cfg(feature = "defmt")]
            defmt::trace!("Using low speed pio program.");
            let low_speed_program = pio_asm!(
                ".side_set 1"

                ".wrap_target"
                // write out x-1 bits
                "lp:"
                "out pins, 1    side 0"
                "jmp x-- lp     side 1"
                // switch directions
                "set pindirs, 0 side 0"
                "nop            side 0"
                // read in y-1 bits
                "lp2:"
                "in pins, 1     side 1"
                "jmp y-- lp2    side 0"

                // wait for event and irq host
                "wait 1 pin 0   side 0"
                "irq 0          side 0"

                ".wrap"
            );
            common.load_program(&low_speed_program.program)
        };

        let mut pin_io: embassy_rp::pio::Pin<PIO> = common.make_pio_pin(dio);
        pin_io.set_pull(Pull::None);
        pin_io.set_schmitt(true);
        pin_io.set_input_sync_bypass(true);
        pin_io.set_drive_strength(Drive::_12mA);
        pin_io.set_slew_rate(SlewRate::Fast);

        let mut pin_clk = common.make_pio_pin(clk);
        pin_clk.set_drive_strength(Drive::_12mA);
        pin_clk.set_slew_rate(SlewRate::Fast);

        let mut cfg = Config::default();
        cfg.use_program(&loaded_program, &[&pin_clk]);
        cfg.set_out_pins(&[&pin_io]);
        cfg.set_in_pins(&[&pin_io]);
        cfg.set_set_pins(&[&pin_io]);
        cfg.shift_out.direction = ShiftDirection::Left;
        cfg.shift_out.auto_fill = true;
        //cfg.shift_out.threshold = 32;
        cfg.shift_in.direction = ShiftDirection::Left;
        cfg.shift_in.auto_fill = true;
        //cfg.shift_in.threshold = 32;
        cfg.clock_divider = clock_divider;

        sm.set_config(&cfg);

        sm.set_pin_dirs(Direction::Out, &[&pin_clk, &pin_io]);
        sm.set_pins(Level::Low, &[&pin_clk, &pin_io]);

        Self {
            cs,
            sm,
            irq,
            dma: dma,
            wrap_target: loaded_program.wrap.target,
        }
    }

    /// Write data to peripheral and return status.
    pub async fn write(&mut self, write: &[u32]) -> u32 {
        self.sm.set_enable(false);
        let write_bits = write.len() * 32 - 1;
        let read_bits = 31;

        #[cfg(feature = "defmt")]
        defmt::trace!("write={} read={}", write_bits, read_bits);

        unsafe {
            self.sm.set_x(write_bits as u32);
            self.sm.set_y(read_bits as u32);
            self.sm.set_pindir(0b1);
            self.sm.exec_jmp(self.wrap_target);
        }

        self.sm.set_enable(true);

        self.sm.tx().dma_push(self.dma.reborrow(), write, false).await;

        let mut status = 0;
        self.sm
            .rx()
            .dma_pull(self.dma.reborrow(), slice::from_mut(&mut status), false)
            .await;
        status
    }

    /// Send command and read response into buffer.
    pub async fn cmd_read(&mut self, cmd: u32, read: &mut [u32]) -> u32 {
        self.sm.set_enable(false);
        let write_bits = 31;
        let read_bits = read.len() * 32 + 32 - 1;

        #[cfg(feature = "defmt")]
        defmt::trace!("cmd_read write={} read={}", write_bits, read_bits);

        #[cfg(feature = "defmt")]
        defmt::trace!("cmd_read cmd = {:02x} len = {}", cmd, read.len());

        unsafe {
            self.sm.set_y(read_bits as u32);
            self.sm.set_x(write_bits as u32);
            self.sm.set_pindir(0b1);
            self.sm.exec_jmp(self.wrap_target);
        }

        // self.cs.set_low();
        self.sm.set_enable(true);

        self.sm
            .tx()
            .dma_push(self.dma.reborrow(), slice::from_ref(&cmd), false)
            .await;
        self.sm.rx().dma_pull(self.dma.reborrow(), read, false).await;

        let mut status = 0;
        self.sm
            .rx()
            .dma_pull(self.dma.reborrow(), slice::from_mut(&mut status), false)
            .await;

        #[cfg(feature = "defmt")]
        defmt::trace!("cmd_read cmd = {:02x} len = {} read = {:08x}", cmd, read.len(), read);

        status
    }
}

impl<'d, PIO, const SM: usize, DMA> SpiBusCyw43 for PioSpi<'d, PIO, SM, DMA>
where
    PIO: Instance,
    DMA: Channel,
{
    async fn cmd_write(&mut self, write: &[u32]) -> u32 {
        self.cs.set_low();
        let status = self.write(write).await;
        self.cs.set_high();
        status
    }

    async fn cmd_read(&mut self, write: u32, read: &mut [u32]) -> u32 {
        self.cs.set_low();
        let status = self.cmd_read(write, read).await;
        self.cs.set_high();
        status
    }

    async fn wait_for_event(&mut self) {
        self.irq.wait().await;
    }
}
