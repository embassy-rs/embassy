#![no_std]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use core::slice;

use cyw43::SpiBusCyw43;
use embassy_rp::dma::Channel;
use embassy_rp::gpio::{Drive, Level, Output, Pull, SlewRate};
use embassy_rp::pio::{instr, Common, Config, Direction, Instance, Irq, PioPin, ShiftDirection, StateMachine};
use embassy_rp::{Peripheral, PeripheralRef};
use fixed::FixedU32;
use pio_proc::pio_asm;

/// SPI comms driven by PIO.
pub struct PioSpi<'d, PIO: Instance, const SM: usize, DMA> {
    cs: Output<'d>,
    sm: StateMachine<'d, PIO, SM>,
    irq: Irq<'d, PIO, 0>,
    dma: PeripheralRef<'d, DMA>,
    wrap_target: u8,
}

impl<'d, PIO, const SM: usize, DMA> PioSpi<'d, PIO, SM, DMA>
where
    DMA: Channel,
    PIO: Instance,
{
    /// Create a new instance of PioSpi.
    pub fn new<DIO, CLK>(
        common: &mut Common<'d, PIO>,
        mut sm: StateMachine<'d, PIO, SM>,
        irq: Irq<'d, PIO, 0>,
        cs: Output<'d>,
        dio: DIO,
        clk: CLK,
        dma: impl Peripheral<P = DMA> + 'd,
    ) -> Self
    where
        DIO: PioPin,
        CLK: PioPin,
    {
        #[cfg(feature = "overclock")]
        let program = pio_asm!(
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
        #[cfg(not(feature = "overclock"))]
        let program = pio_asm!(
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
        let loaded_program = common.load_program(&program.program);
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

        #[cfg(feature = "overclock")]
        {
            // 125mhz Pio => 62.5Mhz SPI Freq. 25% higher than theoretical maximum according to
            // data sheet, but seems to work fine.
            cfg.clock_divider = FixedU32::from_bits(0x0100);
        }

        #[cfg(not(feature = "overclock"))]
        {
            // same speed as pico-sdk, 62.5Mhz
            // This is actually the fastest we can go without overclocking.
            // According to data sheet, the theoretical maximum is 100Mhz Pio => 50Mhz SPI Freq.
            // However, the PIO uses a fractional divider, which works by introducing jitter when
            // the divider is not an integer. It does some clocks at 125mhz and others at 62.5mhz
            // so that it averages out to the desired frequency of 100mhz. The 125mhz clock cycles
            // violate the maximum from the data sheet.
            cfg.clock_divider = FixedU32::from_bits(0x0200);
        }

        sm.set_config(&cfg);

        sm.set_pin_dirs(Direction::Out, &[&pin_clk, &pin_io]);
        sm.set_pins(Level::Low, &[&pin_clk, &pin_io]);

        Self {
            cs,
            sm,
            irq,
            dma: dma.into_ref(),
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
            instr::set_x(&mut self.sm, write_bits as u32);
            instr::set_y(&mut self.sm, read_bits as u32);
            instr::set_pindir(&mut self.sm, 0b1);
            instr::exec_jmp(&mut self.sm, self.wrap_target);
        }

        self.sm.set_enable(true);

        self.sm.tx().dma_push(self.dma.reborrow(), write).await;

        let mut status = 0;
        self.sm
            .rx()
            .dma_pull(self.dma.reborrow(), slice::from_mut(&mut status))
            .await;
        status
    }

    /// Send command and read response into buffer.
    pub async fn cmd_read(&mut self, cmd: u32, read: &mut [u32]) -> u32 {
        self.sm.set_enable(false);
        let write_bits = 31;
        let read_bits = read.len() * 32 + 32 - 1;

        #[cfg(feature = "defmt")]
        defmt::trace!("write={} read={}", write_bits, read_bits);

        unsafe {
            instr::set_y(&mut self.sm, read_bits as u32);
            instr::set_x(&mut self.sm, write_bits as u32);
            instr::set_pindir(&mut self.sm, 0b1);
            instr::exec_jmp(&mut self.sm, self.wrap_target);
        }

        // self.cs.set_low();
        self.sm.set_enable(true);

        self.sm.tx().dma_push(self.dma.reborrow(), slice::from_ref(&cmd)).await;
        self.sm.rx().dma_pull(self.dma.reborrow(), read).await;

        let mut status = 0;
        self.sm
            .rx()
            .dma_pull(self.dma.reborrow(), slice::from_mut(&mut status))
            .await;
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
