//! Pio backed uart drivers

use core::convert::Infallible;

use embedded_io_async::{ErrorType, Read, Write};
use fixed::traits::ToFixed;

use crate::clocks::clk_sys_freq;
use crate::gpio::Level;
use crate::pio::{
    Common, Config, Direction as PioDirection, FifoJoin, Instance, LoadedProgram, PioPin, ShiftDirection, StateMachine,
};
use crate::Peri;

/// This struct represents a uart tx program loaded into pio instruction memory.
pub struct PioUartTxProgram<'d, PIO: Instance> {
    prg: LoadedProgram<'d, PIO>,
}

impl<'d, PIO: Instance> PioUartTxProgram<'d, PIO> {
    /// Load the uart tx program into the given pio
    pub fn new(common: &mut Common<'d, PIO>) -> Self {
        let prg = pio::pio_asm!(
            r#"
                .side_set 1 opt

                ; An 8n1 UART transmit program.
                ; OUT pin 0 and side-set pin 0 are both mapped to UART TX pin.

                    pull       side 1 [7]  ; Assert stop bit, or stall with line in idle state
                    set x, 7   side 0 [7]  ; Preload bit counter, assert start bit for 8 clocks
                bitloop:                   ; This loop will run 8 times (8n1 UART)
                    out pins, 1            ; Shift 1 bit from OSR to the first OUT pin
                    jmp x-- bitloop   [6]  ; Each loop iteration is 8 cycles.
            "#
        );

        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}

/// PIO backed Uart transmitter
pub struct PioUartTx<'d, PIO: Instance, const SM: usize> {
    sm_tx: StateMachine<'d, PIO, SM>,
}

impl<'d, PIO: Instance, const SM: usize> PioUartTx<'d, PIO, SM> {
    /// Configure a pio state machine to use the loaded tx program.
    pub fn new(
        baud: u32,
        common: &mut Common<'d, PIO>,
        mut sm_tx: StateMachine<'d, PIO, SM>,
        tx_pin: Peri<'d, impl PioPin>,
        program: &PioUartTxProgram<'d, PIO>,
    ) -> Self {
        let tx_pin = common.make_pio_pin(tx_pin);
        sm_tx.set_pins(Level::High, &[&tx_pin]);
        sm_tx.set_pin_dirs(PioDirection::Out, &[&tx_pin]);

        let mut cfg = Config::default();

        cfg.set_out_pins(&[&tx_pin]);
        cfg.use_program(&program.prg, &[&tx_pin]);
        cfg.shift_out.auto_fill = false;
        cfg.shift_out.direction = ShiftDirection::Right;
        cfg.fifo_join = FifoJoin::TxOnly;
        cfg.clock_divider = (clk_sys_freq() / (8 * baud)).to_fixed();
        sm_tx.set_config(&cfg);
        sm_tx.set_enable(true);

        Self { sm_tx }
    }

    /// Write a single u8
    pub async fn write_u8(&mut self, data: u8) {
        self.sm_tx.tx().wait_push(data as u32).await;
    }
}

impl<PIO: Instance, const SM: usize> ErrorType for PioUartTx<'_, PIO, SM> {
    type Error = Infallible;
}

impl<PIO: Instance, const SM: usize> Write for PioUartTx<'_, PIO, SM> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Infallible> {
        for byte in buf {
            self.write_u8(*byte).await;
        }
        Ok(buf.len())
    }
}

/// This struct represents a Uart Rx program loaded into pio instruction memory.
pub struct PioUartRxProgram<'d, PIO: Instance> {
    prg: LoadedProgram<'d, PIO>,
}

impl<'d, PIO: Instance> PioUartRxProgram<'d, PIO> {
    /// Load the uart rx program into the given pio
    pub fn new(common: &mut Common<'d, PIO>) -> Self {
        let prg = pio::pio_asm!(
            r#"
                ; Slightly more fleshed-out 8n1 UART receiver which handles framing errors and
                ; break conditions more gracefully.
                ; IN pin 0 and JMP pin are both mapped to the GPIO used as UART RX.

                start:
                    wait 0 pin 0        ; Stall until start bit is asserted
                    set x, 7    [10]    ; Preload bit counter, then delay until halfway through
                rx_bitloop:             ; the first data bit (12 cycles incl wait, set).
                    in pins, 1          ; Shift data bit into ISR
                    jmp x-- rx_bitloop [6] ; Loop 8 times, each loop iteration is 8 cycles
                    jmp pin good_rx_stop   ; Check stop bit (should be high)

                    irq 4 rel           ; Either a framing error or a break. Set a sticky flag,
                    wait 1 pin 0        ; and wait for line to return to idle state.
                    jmp start           ; Don't push data if we didn't see good framing.

                good_rx_stop:           ; No delay before returning to start; a little slack is
                    in null 24
                    push                ; important in case the TX clock is slightly too fast.
            "#
        );

        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}

/// PIO backed Uart reciever
pub struct PioUartRx<'d, PIO: Instance, const SM: usize> {
    sm_rx: StateMachine<'d, PIO, SM>,
}

impl<'d, PIO: Instance, const SM: usize> PioUartRx<'d, PIO, SM> {
    /// Configure a pio state machine to use the loaded rx program.
    pub fn new(
        baud: u32,
        common: &mut Common<'d, PIO>,
        mut sm_rx: StateMachine<'d, PIO, SM>,
        rx_pin: Peri<'d, impl PioPin>,
        program: &PioUartRxProgram<'d, PIO>,
    ) -> Self {
        let mut cfg = Config::default();
        cfg.use_program(&program.prg, &[]);

        let rx_pin = common.make_pio_pin(rx_pin);
        sm_rx.set_pins(Level::High, &[&rx_pin]);
        cfg.set_in_pins(&[&rx_pin]);
        cfg.set_jmp_pin(&rx_pin);
        sm_rx.set_pin_dirs(PioDirection::In, &[&rx_pin]);

        cfg.clock_divider = (clk_sys_freq() / (8 * baud)).to_fixed();
        cfg.shift_in.auto_fill = false;
        cfg.shift_in.direction = ShiftDirection::Right;
        cfg.shift_in.threshold = 32;
        cfg.fifo_join = FifoJoin::RxOnly;
        sm_rx.set_config(&cfg);
        sm_rx.set_enable(true);

        Self { sm_rx }
    }

    /// Wait for a single u8
    pub async fn read_u8(&mut self) -> u8 {
        self.sm_rx.rx().wait_pull().await as u8
    }
}

impl<PIO: Instance, const SM: usize> ErrorType for PioUartRx<'_, PIO, SM> {
    type Error = Infallible;
}

impl<PIO: Instance, const SM: usize> Read for PioUartRx<'_, PIO, SM> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Infallible> {
        let mut i = 0;
        while i < buf.len() {
            buf[i] = self.read_u8().await;
            i += 1;
        }
        Ok(i)
    }
}
