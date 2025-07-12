#![no_std]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use core::slice;

use cyw43::SpiBusCyw43;
use embassy_rp::dma::Channel;
use embassy_rp::gpio::{Drive, Level, Output, Pull, SlewRate};
use embassy_rp::pio::program::pio_asm;
use embassy_rp::pio::{Common, Config, Direction, Instance, Irq, PioPin, ShiftDirection, StateMachine};
use embassy_rp::Peri;
use fixed::types::extra::U8;
use fixed::FixedU32;

/// SPI comms driven by PIO.
pub struct PioSpi<'d, PIO: Instance, const SM: usize, DMA: Channel> {
    cs: Output<'d>,
    sm: StateMachine<'d, PIO, SM>,
    irq: Irq<'d, PIO, 0>,
    dma: Peri<'d, DMA>,
    wrap_target: u8,
}

/// The default clock divider that works for Pico 1 and 2 W. As well as the RM2 on rp2040 devices.
/// same speed as pico-sdk, 62.5Mhz
/// This is actually the fastest we can go without overclocking.
/// According to data sheet, the theoretical maximum is 100Mhz Pio => 50Mhz SPI Freq.
/// However, the PIO uses a fractional divider, which works by introducing jitter when
/// the divider is not an integer. It does some clocks at 125mhz and others at 62.5mhz
/// so that it averages out to the desired frequency of 100mhz. The 125mhz clock cycles
/// violate the maximum from the data sheet.
pub const DEFAULT_CLOCK_DIVIDER: FixedU32<U8> = FixedU32::from_bits(0x0200);

/// The overclock clock divider for the Pico 1 W. Does not work on any known RM2 devices.
/// 125mhz Pio => 62.5Mhz SPI Freq. 25% higher than theoretical maximum according to
/// data sheet, but seems to work fine.
pub const OVERCLOCK_CLOCK_DIVIDER: FixedU32<U8> = FixedU32::from_bits(0x0100);

/// The clock divider for the RM2 module. Found to be needed for the Pimoroni Pico Plus 2 W,
/// Pico Plus 2 Non w with the RM2 breakout module, and the Pico 2 with the RM2 breakout module.
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
        let loaded_program = if clock_divider < DEFAULT_CLOCK_DIVIDER {
            let overclock_program = pio_asm!(
                ".side_set 1"

                ".wrap_target"
                // write out x-1 bits
                "lp:"
                "out pins, 1    side 0"
                "jmp x-- lp     side 1"
                // switch directions
                "set pindirs, 0 side 0"
                "nop            side 1"  // necessary for clkdiv=1.
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
        } else {
            let default_program = pio_asm!(
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
            common.load_program(&default_program.program)
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
