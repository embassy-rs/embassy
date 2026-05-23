use core::pin::pin;
use core::task::Poll;
use core::{mem, ptr};

use cortex_m::peripheral::SCB;
use embassy_futures::poll_once;
use embassy_stm32::ipcc::{Ipcc, IpccRxChannel, IpccTxChannel};
use openthread_radio::{Capabilities, MacCapabilities, Radio, RadioErrorKind};

use crate::tables::{
    THREAD_CLI_CMD_BUFFER, THREAD_CLI_NOT_BUFFER, THREAD_CMD_BUFFER, THREAD_NOTIF_RSP_EVT_BUFFER, TL_THREAD_TABLE,
    TL_TRACES_TABLE, TRACES_EVT_QUEUE, ThreadTable, TracesTable,
};
use crate::wb::channels::cpu1::IPCC_THREAD_OT_CMD_RSP_CHANNEL;
use crate::wb::unsafe_linked_list::LinkedListNode;

pub struct Thread<'a> {
    _thread_cmd_rsp_channel: IpccTxChannel<'a>,
    _thread_notification_ack_channel: IpccRxChannel<'a>,
}

impl<'a> Thread<'a> {
    pub(crate) fn new(
        thread_cmd_rsp_channel: IpccTxChannel<'a>,
        thread_notification_ack_channel: IpccRxChannel<'a>,
    ) -> Self {
        unsafe {
            LinkedListNode::init_head(TRACES_EVT_QUEUE.as_mut_ptr() as *mut _);

            TL_TRACES_TABLE.as_mut_ptr().write_volatile(TracesTable {
                traces_queue: TRACES_EVT_QUEUE.as_ptr() as *const _,
            });

            TL_THREAD_TABLE.as_mut_ptr().write_volatile(ThreadTable {
                nostack_buffer: THREAD_NOTIF_RSP_EVT_BUFFER.as_ptr() as *const _,
                clicmdrsp_buffer: THREAD_CLI_CMD_BUFFER.as_ptr() as *const _,
                otcmdrsp_buffer: THREAD_NOTIF_RSP_EVT_BUFFER.as_ptr() as *const _,
                clinot_buffer: THREAD_CLI_NOT_BUFFER.as_ptr() as *const _,
            });
        };

        Self {
            _thread_cmd_rsp_channel: thread_cmd_rsp_channel,
            _thread_notification_ack_channel: thread_notification_ack_channel,
        }
    }
}

pub struct ControllerAdapter<'a> {
    _thread: Thread<'a>,
}

impl<'a> ControllerAdapter<'a> {
    pub fn new(thread: Thread<'a>) -> Self {
        Self { _thread: thread }
    }
}

impl<'a> Radio for ControllerAdapter<'a> {
    type Error = RadioErrorKind;

    const CAPS: Capabilities = Capabilities::empty();

    const MAC_CAPS: MacCapabilities = MacCapabilities::empty();

    async fn receive(&mut self, _psdu_buf: &mut [u8]) -> Result<openthread_radio::PsduMeta, Self::Error> {
        unreachable!()
    }

    async fn transmit(
        &mut self,
        _psdu: &[u8],
        _cca: bool,
        _ack_psdu_buf: Option<&mut [u8]>,
    ) -> Result<Option<openthread_radio::PsduMeta>, Self::Error> {
        unreachable!()
    }

    async fn set_config(&mut self, _config: &openthread_radio::Config) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// A tiny executor
///
/// This provides power savings but does not allow other tasks to run concurrently
fn tiny_exec(f: impl Future) {
    unsafe extern "Rust" {
        unsafe fn __pender(_context: *mut ());
    }

    // Pin the future
    let mut f = pin!(f);

    // Clear any sleep flags in the event that they are set
    let mut scb: SCB = unsafe { mem::transmute(()) };
    scb.clear_sleepdeep();

    // Poll it !
    while matches!(poll_once(&mut f), Poll::Pending) {
        // We only run on cortex-m in this crate
        cortex_m::asm::wfe();
    }

    // Make sure that embassy-executor runs again after we're done
    unsafe { __pender(ptr::null_mut()) };
}

fn cmd_transfer() {
    tiny_exec(async {
        unsafe {
            Ipcc::send(IPCC_THREAD_OT_CMD_RSP_CHANNEL as u8, || {
                /* OpenThread OT command cmdcode range 0x280 .. 0x3DF = 352 */
                // p_thread_otcmdbuffer->cmdserial.cmd.cmdcode = 0x280U;
                /* Size = otCmdBuffer->Size (Number of OT cmd arguments : 1 arg = 32bits so multiply by 4 to get size in bytes)
                 * + ID (4 bytes) + Size (4 bytes) */
                // uint32_t l_size = ((Thread_OT_Cmd_Request_t*)(p_thread_otcmdbuffer->cmdserial.cmd.payload))->Size * 4U + 8U;
                // p_thread_otcmdbuffer->cmdserial.cmd.plen = l_size;
            })
        }
    });
}

fn cmd_flush() {
    tiny_exec(async {
        unsafe {
            Ipcc::flush(IPCC_THREAD_OT_CMD_RSP_CHANNEL as u8).await;
        }
    });
}

/// Embassy Runtime -> Openthread Rust -> stm32-bindings -> embassy-stm32-wpan

#[unsafe(no_mangle)]
extern "C" fn Pre_OtCmdProcessing() {
    cmd_flush();
}

#[unsafe(no_mangle)]
extern "C" fn THREAD_Get_OTCmdPayloadBuffer() -> *mut u8 {
    unsafe { THREAD_CMD_BUFFER.as_mut_ptr() as *mut _ }
}

#[unsafe(no_mangle)]
extern "C" fn Ot_Cmd_Transfer() {
    cmd_transfer();
}

#[unsafe(no_mangle)]
extern "C" fn Ot_Cmd_TransferWithNotif() {
    cmd_transfer();
}

#[unsafe(no_mangle)]
extern "C" fn Post_OtCmdProcessing() {}
