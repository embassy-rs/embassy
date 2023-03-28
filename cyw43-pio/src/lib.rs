#![no_std]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

use core::slice;

use cyw43::SpiBusCyw43;
use embassy_rp::dma::Channel;
use embassy_rp::gpio::{Drive, Output, Pin, Pull, SlewRate};
use embassy_rp::pio::{PioStateMachine, ShiftDirection};
use embassy_rp::relocate::RelocatedProgram;
use embassy_rp::{pio_instr_util, Peripheral};
use pio::Wrap;
use pio_proc::pio_asm;

pub struct PioSpi<CS: Pin, SM, DMA> {
    cs: Output<'static, CS>,
    sm: SM,
    dma: DMA,
    wrap_target: u8,
}

impl<CS, SM, DMA> PioSpi<CS, SM, DMA>
where
    SM: PioStateMachine,
    DMA: Channel,
    CS: Pin,
{
    pub fn new<DIO, CLK>(mut sm: SM, cs: Output<'static, CS>, dio: DIO, clk: CLK, dma: DMA) -> Self
    where
        DIO: Pin,
        CLK: Pin,
    {
        let program = pio_asm!(
            ".side_set 1"

            ".wrap_target"
            // write out x-1 bits
            "lp:",
            "out pins, 1    side 0"
            "jmp x-- lp     side 1"
            // switch directions
            "set pindirs, 0 side 0"
            // these nops seem to be necessary for fast clkdiv
            "nop            side 1"
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

        let relocated = RelocatedProgram::new(&program.program);

        let mut pin_io = sm.make_pio_pin(dio);
        pin_io.set_pull(Pull::Down);
        pin_io.set_schmitt(true);
        pin_io.set_input_sync_bypass(true);

        let mut pin_clk = sm.make_pio_pin(clk);
        pin_clk.set_drive_strength(Drive::_12mA);
        pin_clk.set_slew_rate(SlewRate::Fast);

        sm.write_instr(relocated.origin() as usize, relocated.code());

        // theoretical maximum according to data sheet, 100Mhz Pio => 50Mhz SPI Freq
        // does not work yet,
        // sm.set_clkdiv(0x0140);

        // same speed as pico-sdk, 62.5Mhz
        sm.set_clkdiv(0x0200);

        // 32 Mhz
        // sm.set_clkdiv(0x03E8);

        // 16 Mhz
        // sm.set_clkdiv(0x07d0);

        // 8Mhz
        // sm.set_clkdiv(0x0a_00);

        // 1Mhz
        // sm.set_clkdiv(0x7d_00);

        // slowest possible
        // sm.set_clkdiv(0xffff_00);

        sm.set_autopull(true);
        // sm.set_pull_threshold(32);
        sm.set_autopush(true);
        // sm.set_push_threshold(32);

        sm.set_out_pins(&[&pin_io]);
        sm.set_in_base_pin(&pin_io);

        sm.set_set_pins(&[&pin_clk]);
        pio_instr_util::set_pindir(&mut sm, 0b1);
        sm.set_set_pins(&[&pin_io]);
        pio_instr_util::set_pindir(&mut sm, 0b1);

        sm.set_sideset_base_pin(&pin_clk);
        sm.set_sideset_count(1);

        sm.set_out_shift_dir(ShiftDirection::Left);
        sm.set_in_shift_dir(ShiftDirection::Left);

        let Wrap { source, target } = relocated.wrap();
        sm.set_wrap(source, target);

        // pull low for startup
        pio_instr_util::set_pin(&mut sm, 0);

        Self {
            cs,
            sm,
            dma,
            wrap_target: target,
        }
    }

    pub async fn write(&mut self, write: &[u32]) -> u32 {
        self.sm.set_enable(false);
        let write_bits = write.len() * 32 - 1;
        let read_bits = 31;

        defmt::trace!("write={} read={}", write_bits, read_bits);

        let mut dma = Peripheral::into_ref(&mut self.dma);
        pio_instr_util::set_x(&mut self.sm, write_bits as u32);
        pio_instr_util::set_y(&mut self.sm, read_bits as u32);
        pio_instr_util::set_pindir(&mut self.sm, 0b1);
        pio_instr_util::exec_jmp(&mut self.sm, self.wrap_target);

        self.sm.set_enable(true);

        self.sm.dma_push(dma.reborrow(), write).await;

        let mut status = 0;
        self.sm.dma_pull(dma.reborrow(), slice::from_mut(&mut status)).await;
        status
    }

    pub async fn cmd_read(&mut self, cmd: u32, read: &mut [u32]) -> u32 {
        self.sm.set_enable(false);
        let write_bits = 31;
        let read_bits = read.len() * 32 + 32 - 1;

        defmt::trace!("write={} read={}", write_bits, read_bits);

        let mut dma = Peripheral::into_ref(&mut self.dma);
        pio_instr_util::set_y(&mut self.sm, read_bits as u32);
        pio_instr_util::set_x(&mut self.sm, write_bits as u32);
        pio_instr_util::set_pindir(&mut self.sm, 0b1);
        pio_instr_util::exec_jmp(&mut self.sm, self.wrap_target);
        // self.cs.set_low();
        self.sm.set_enable(true);

        self.sm.dma_push(dma.reborrow(), slice::from_ref(&cmd)).await;
        self.sm.dma_pull(dma.reborrow(), read).await;

        let mut status = 0;
        self.sm.dma_pull(dma.reborrow(), slice::from_mut(&mut status)).await;
        status
    }
}

impl<CS, SM, DMA> SpiBusCyw43 for PioSpi<CS, SM, DMA>
where
    CS: Pin,
    SM: PioStateMachine,
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
        self.sm.wait_irq(0).await;
        self.sm.clear_irq(0);
    }
}
