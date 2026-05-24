use core::pin::pin;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::Poll;
use core::{mem, ptr, slice};

use cortex_m::peripheral::SCB;
use embassy_futures::poll_once;
use embassy_stm32::ipcc::{Ipcc, IpccRxChannel, IpccTxChannel};

use crate::evt::EvtPacket;
use crate::tables::{
    THREAD_CLI_CMD_BUFFER, THREAD_CLI_NOT_BUFFER, THREAD_CMD_BUFFER, THREAD_NOTIF_RSP_EVT_BUFFER, TL_THREAD_TABLE,
    TL_TRACES_TABLE, TRACES_EVT_QUEUE, ThreadTable, TracesTable,
};
use crate::wb::channels::cpu1::IPCC_THREAD_OT_CMD_RSP_CHANNEL;
use crate::wb::cmd::{CmdPacket, CmdSerialStub};
use crate::wb::consts::TlPacketType;
use crate::wb::unsafe_linked_list::LinkedListNode;

pub struct Thread<'a> {
    _thread_cmd_rsp_channel: IpccTxChannel<'a>,
    thread_notification_ack_channel: IpccRxChannel<'a>,
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
            thread_notification_ack_channel,
        }
    }
}

pub struct Runner<'a> {
    thread_notification_ack_channel: IpccRxChannel<'a>,
}

impl<'a> Runner<'a> {
    pub fn new(thread: Thread<'a>) -> Self {
        Self {
            thread_notification_ack_channel: thread.thread_notification_ack_channel,
        }
    }

    pub async fn run_stack(&mut self) -> ! {
        unsafe extern "C" {
            unsafe fn OpenThread_CallBack_Processing();
        }

        // This future never returns because None is always returned, therefore more data is always fetched
        self.thread_notification_ack_channel
            .receive::<()>(|| unsafe {
                OpenThread_CallBack_Processing();

                None
            })
            .await;

        loop {}
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
                /* Structure of the messages exchanged between M0 and M4 (inside of the payload) */
                // #define OT_CMD_BUFFER_SIZE 20U
                // typedef PACKED_STRUCT
                // {
                // uint32_t  ID;
                // uint32_t  Size;
                // uint32_t  Data[OT_CMD_BUFFER_SIZE];
                // }Thread_OT_Cmd_Request_t;

                let (payload, len) = CmdPacket::read_payload(THREAD_CMD_BUFFER.as_ptr());

                compiler_fence(Ordering::Acquire);

                let payload = slice::from_raw_parts(payload, len);
                let l_size = u32::from_le_bytes(payload[4..8].try_into().unwrap()) * 4 + 8;

                /* OpenThread OT command cmdcode range 0x280 .. 0x3DF = 352 */

                // p_thread_otcmdbuffer->cmdserial.cmd.cmdcode = 0x280U;
                // p_thread_otcmdbuffer->cmdserial.cmd.plen = l_size;

                CmdPacket::write_stub(
                    THREAD_CMD_BUFFER.as_mut_ptr(),
                    CmdSerialStub {
                        ty: TlPacketType::OtCmd as u8,
                        cmd_code: 0x280u16,
                        payload_len: l_size as u8,
                    },
                );
            })
            .await;

            Ipcc::flush(IPCC_THREAD_OT_CMD_RSP_CHANNEL as u8).await;
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

/// Embassy Runtime -> rust-openthread -> stm32-bindings -> embassy-stm32-wpan
///
/// This approach effectively blocks embassy while an OT command is executing, but
/// propagating async up the stack to rust-openthread is not worth it for now.

#[unsafe(no_mangle)]
extern "C" fn Pre_OtCmdProcessing() {
    cmd_flush();
}

#[unsafe(no_mangle)]
extern "C" fn THREAD_Get_OTCmdPayloadBuffer() -> *mut u8 {
    unsafe { &mut (*THREAD_CMD_BUFFER.as_mut_ptr()).cmdserial.cmd.payload as *mut _ }
}

#[unsafe(no_mangle)]
extern "C" fn THREAD_Get_OTCmdRspPayloadBuffer() -> *mut u8 {
    unsafe {
        let p_event_packet = THREAD_CMD_BUFFER.as_mut_ptr() as *mut EvtPacket;

        &mut ((*p_event_packet).evt_serial.evt.payload) as *mut u8
    }
}

#[unsafe(no_mangle)]
extern "C" fn Ot_Cmd_Transfer() {
    cmd_transfer();
}

#[unsafe(no_mangle)]
extern "C" fn Ot_Cmd_TransferWithNotif() {
    // This cmd will also send a notification, that we will deal with after we finish processing
    // the cmd. I don't think think any further special handling is required in this case.
    cmd_transfer();
}

#[unsafe(no_mangle)]
extern "C" fn Post_OtCmdProcessing() {
    cmd_flush();
}

#[unsafe(no_mangle)]
extern "C" fn THREAD_Get_NotificationPayloadBuffer() -> *mut u8 {
    unsafe {
        let p_event_packet = THREAD_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut EvtPacket;

        &mut ((*p_event_packet).evt_serial.evt.payload) as *mut u8
    }
}

#[unsafe(no_mangle)]
extern "C" fn THREAD_Get_RCPPayloadBuffer() -> *mut u8 {
    unsafe {
        let p_event_packet = THREAD_NOTIF_RSP_EVT_BUFFER.as_mut_ptr() as *mut EvtPacket;

        &mut ((*p_event_packet).evt_serial.evt.payload) as *mut u8
    }
}
