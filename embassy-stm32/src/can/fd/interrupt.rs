use core::{marker::PhantomData, sync::atomic::Ordering};

use bit_field::BitField;
use stm32_metapac::can::regs::Ir;

use crate::{can::Instance, interrupt};

use super::State;

/// Interrupt handler channel 0.
pub struct IT0InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

// == Exceptions
// 29 ARA  : Triggered on Access to Reserved Address
// 28 PED  : Protocol Error in Data Phase Detected
// 27 PEA  : Protocol Error in Arbitration Phase Detected
// 26 WDI  : Watchdog Interrupt
// 25 BO   : Bus_Off status changed
// 24 EW   : Error_Warning status changed
// 23 EP   : Error_Passive status changed
// 22 ELO  : Error Logging Counter Overflow occurred
// - 21 BEU  : Uncorrected Bit error detected when reading from Message RAM, CCCR.INIT set to 1
// - 20 BEC  : Corrected Bit error detected and corrected when reading from Message RAM
// 17 MRAF : Message RAM access failure

// == Operational Errors
// 18 TOO  : Timeout reached
// 16 TSW  : Timestamp counter Wraparound occurred

// == Tx Event
// 15 TEFL : Tx Event FIFO Element Lost
// 14 TEFF : Tx Event FIFO is full
// 13 TEFW : Tx Event FIFO Watermark Reached
// 12 TEFN : Tx Event FIFO New Entry

// == Tx
// 11 TFE  : Tx FIFO Empty
// 10 TCF  : Tx cancel request finished
//  9 TC   : Tx completed

// == Rx
//  8 HPM  : A high priority message has been received
// 19 DRX  : At least one message stored to Dedicated Rx Buffer

// == Rx Fifo 1
//  7 RF1L : Rx FIFO 1 Message Lost
//  6 RF1F : Rx FIFO 1 Full
//  5 RF1W : Rx FIFO 1 Watermark reached
//  4 RF1N : Rx FIFO 1 New message

// == Rx Fifo 0
//  3 RF0L : Rx FIFO 0 Message Lost
//  2 RF0F : Rx FIFO 0 Full
//  1 RF0W : Rx FIFO 0 Watermark reached
//  0 RF0N : Rx FIFO 0 New Message

// We use IT0 for everything currently
impl<T: Instance> interrupt::typelevel::Handler<T::IT0Interrupt> for IT0InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::info().low.regs;
        let ir = regs.ir().read();

        let mut ir_clear: Ir = Ir(0);

        // Sync status + wake on either Tx complete or Tx cancel.
        if ir.tc() || ir.tcf() {
            ir_clear.set_tc(true);
            ir_clear.set_tcf(true);

            T::state().sync_tx_status();
            T::state().tx_done_waker.wake();
        }

        // TX event FIFO new element
        if ir.tefn() {
            ir_clear.set_tefn(true);

            // TODO wake
        }

        // RX FIFO new element
        if ir.rfn(0) {
            ir_clear.set_rfn(0, true);
            T::state().rx_fifo_waker[0].wake();
        }
        if ir.rfn(1) {
            ir_clear.set_rfn(1, true);
            T::state().rx_fifo_waker[1].wake();
        }
        // RX Dedicated stored
        if ir.drx() {
            ir_clear.set_drx(true);
            T::state().rx_dedicated_waker.wake();
        }

        // Bus_Off
        if ir.bo() {
            ir_clear.set_bo(true);
            if regs.psr().read().bo() {
                let state = T::state();

                let settings_flags = state.settings_flags.load(Ordering::Relaxed);
                if settings_flags.get_bit(State::FLAG_AUTO_RECOVER_BUS_OFF) {
                    // Initiate bus-off recovery sequence by resetting CCCR.INIT
                    regs.cccr().modify(|w| w.set_init(false));
                } else {
                    state
                        .state_flags
                        .fetch_or(1 << State::STATE_FLAG_BUS_OFF, Ordering::Relaxed);
                }

                state.err_waker.wake();
                if settings_flags.get_bit(State::FLAG_PROPAGATE_ERRORS_TO_RX) {
                    state.rx_dedicated_waker.wake();
                    state.rx_fifo_waker[0].wake();
                    state.rx_fifo_waker[1].wake();
                }
                if settings_flags.get_bit(State::FLAG_PROPAGATE_ERRORS_TO_TX) {
                    state.tx_done_waker.wake();
                }
            }
            // Bus Off status flag cleared by error recovery code.
        }

        regs.ir().write_value(ir_clear);
    }
}

/// Interrupt handler channel 1.
pub struct IT1InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::IT1Interrupt> for IT1InterruptHandler<T> {
    unsafe fn on_interrupt() {}
}
