#[cfg(feature = "bt-hci")]
use core::cell::RefCell;
#[cfg(feature = "bt-hci")]
use core::future::poll_fn;
use core::ptr;
#[cfg(feature = "bt-hci")]
use core::sync::atomic::{AtomicBool, Ordering};

use embassy_stm32::ipcc::{Ipcc, IpccRxChannel, IpccTxChannel};
#[cfg(feature = "bt-hci")]
use embassy_sync::blocking_mutex;
#[cfg(feature = "bt-hci")]
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
#[cfg(feature = "bt-hci")]
use embassy_sync::mutex::Mutex;
#[cfg(feature = "bt-hci")]
use embassy_sync::signal::Signal;
#[cfg(feature = "bt-hci")]
use embassy_sync::waitqueue::AtomicWaker;

use crate::sub::mm;
use crate::util::Flag;
use crate::wb::channels::cpu1::IPCC_HCI_ACL_DATA_CHANNEL;
use crate::wb::cmd::CmdPacket;
use crate::wb::consts::{TL_BLEEVT_CC_OPCODE, TL_BLEEVT_CS_OPCODE, TlPacketType};
use crate::wb::evt;
use crate::wb::evt::{EvtBox, EvtPacket};
use crate::wb::tables::{BLE_CMD_BUFFER, BleTable, CS_BUFFER, EVT_QUEUE, HCI_ACL_DATA_BUFFER, TL_BLE_TABLE};
use crate::wb::unsafe_linked_list::LinkedListNode;

static ACL_EVT_OUT: Flag = Flag::new(false);

/// A guard that, once constructed, may be used to send BLE commands to CPU2.
///
/// It is the responsibility of the caller to ensure that they have awaited an event via
/// [crate::sub::sys::Sys::read] before sending any of these commands, and to call
/// [crate::sub::sys::Sys::shci_c2_ble_init] and await the HCI_COMMAND_COMPLETE_EVENT before
/// sending any other commands.
///
/// # Example
///
/// ```
/// # embassy_stm32::bind_interrupts!(struct Irqs{
/// #     IPCC_C1_RX => ReceiveInterruptHandler;
/// #     IPCC_C1_TX => TransmitInterruptHandler;
/// # });
/// #
/// # let p = embassy_stm32::init(embassy_stm32::Config::default());
/// # let mut mbox = embassy_stm32_wpan::TlMbox::init(p.IPCC, Irqs, embassy_stm32::ipcc::Config::default());
/// #
/// # let sys_event = mbox.sys_subsystem.read().await;
/// # let _command_status = mbox.sys_subsystem.shci_c2_ble_init(Default::default());
/// # // BLE commands may now be sent
/// #
/// # mbox.ble_subsystem.reset().await;
/// # let _reset_response = mbox.ble_subsystem.read().await;
/// ```
pub struct Ble<'a> {
    hw_ipcc_ble_cmd_channel: IpccTxChannel<'a>,
    ipcc_ble_event_channel: IpccRxChannel<'a>,
    ipcc_hci_acl_tx_data_channel: IpccTxChannel<'a>,
    ipcc_hci_acl_rx_data_channel: IpccRxChannel<'a>,
}

/// BLE for only sending commands to CPU2
pub struct BleTx<'a> {
    hw_ipcc_ble_cmd_channel: IpccTxChannel<'a>,
    ipcc_hci_acl_tx_data_channel: IpccTxChannel<'a>,
}

/// BLE for only receive commands from CPU2
pub struct BleRx<'a> {
    ipcc_ble_event_channel: IpccRxChannel<'a>,
    ipcc_hci_acl_rx_data_channel: IpccRxChannel<'a>,
}

impl<'a> Ble<'a> {
    /// Constructs a guard that allows for BLE commands to be sent to CPU2.
    ///
    /// This takes the place of `TL_BLE_Init`, completing that step as laid out in AN5289, Fig 66.
    pub(crate) fn new(
        hw_ipcc_ble_cmd_channel: IpccTxChannel<'a>,
        ipcc_ble_event_channel: IpccRxChannel<'a>,
        ipcc_hci_acl_tx_data_channel: IpccTxChannel<'a>,
        ipcc_hci_acl_rx_data_channel: IpccRxChannel<'a>,
    ) -> Self {
        unsafe {
            LinkedListNode::init_head(EVT_QUEUE.as_mut_ptr());

            TL_BLE_TABLE.as_mut_ptr().write_volatile(BleTable {
                pcmd_buffer: BLE_CMD_BUFFER.as_mut_ptr().cast(),
                pcs_buffer: CS_BUFFER.as_ptr().cast(),
                pevt_queue: EVT_QUEUE.as_ptr().cast(),
                phci_acl_data_buffer: HCI_ACL_DATA_BUFFER.as_mut_ptr().cast(),
            });
        }

        Self {
            hw_ipcc_ble_cmd_channel,
            ipcc_ble_event_channel,
            ipcc_hci_acl_tx_data_channel,
            ipcc_hci_acl_rx_data_channel,
        }
    }

    /// Split current BLE into BleTx and BleRx
    pub fn split(self) -> (BleTx<'a>, BleRx<'a>) {
        (
            BleTx {
                hw_ipcc_ble_cmd_channel: self.hw_ipcc_ble_cmd_channel,
                ipcc_hci_acl_tx_data_channel: self.ipcc_hci_acl_tx_data_channel,
            },
            BleRx {
                ipcc_ble_event_channel: self.ipcc_ble_event_channel,
                ipcc_hci_acl_rx_data_channel: self.ipcc_hci_acl_rx_data_channel,
            },
        )
    }

    /// `HW_IPCC_BLE_EvtNot`
    pub async fn tl_read(&mut self) -> EvtBox<Self> {
        self.ipcc_ble_event_channel
            .receive(|| unsafe {
                if let Some(node_ptr) =
                    critical_section::with(|cs| LinkedListNode::remove_head(cs, EVT_QUEUE.as_mut_ptr()))
                {
                    Some(EvtBox::new(node_ptr.cast()))
                } else {
                    None
                }
            })
            .await
    }

    /// `TL_BLE_SendCmd`
    pub async fn tl_write(&mut self, opcode: u16, payload: &[u8]) {
        self.hw_ipcc_ble_cmd_channel
            .send(|| unsafe {
                CmdPacket::write_into(BLE_CMD_BUFFER.as_mut_ptr(), TlPacketType::BleCmd, opcode, payload);
            })
            .await;
    }

    /// `TL_BLE_SendAclData`
    pub async fn acl_write(&mut self, handle: u16, payload: &[u8]) {
        self.ipcc_hci_acl_tx_data_channel
            .send_exclusive(|| unsafe {
                CmdPacket::write_into(
                    HCI_ACL_DATA_BUFFER.as_mut_ptr() as *mut _,
                    TlPacketType::AclData,
                    handle,
                    payload,
                );
            })
            .await;
    }

    /// `TL_BLE_AclNot`
    pub async fn acl_read(&mut self) -> EvtBox<Self> {
        ACL_EVT_OUT.wait_for_low().await;
        self.ipcc_hci_acl_rx_data_channel
            .receive(|| unsafe { Some(EvtBox::new(HCI_ACL_DATA_BUFFER.as_mut_ptr() as *mut _)) })
            .await
    }
}

impl<'a> evt::MemoryManager for Ble<'a> {
    unsafe fn new_event_packet(evt: *mut EvtPacket) {
        if ptr::eq(evt, HCI_ACL_DATA_BUFFER.as_mut_ptr() as *mut _) {
            ACL_EVT_OUT.set_high();
        }
    }

    /// SAFETY: passing a pointer to something other than a managed event packet is UB
    unsafe fn drop_event_packet(evt: *mut EvtPacket) {
        if ptr::eq(evt, HCI_ACL_DATA_BUFFER.as_mut_ptr() as *mut _) {
            Ipcc::clear(IPCC_HCI_ACL_DATA_CHANNEL as u8);
            ACL_EVT_OUT.set_low();

            return;
        }

        let stub = unsafe { EvtBox::read_stub(evt) };
        if !(stub.evt_code == TL_BLEEVT_CS_OPCODE || stub.evt_code == TL_BLEEVT_CC_OPCODE) {
            mm::MemoryManager::drop_event_packet(evt);
        }
    }
}

#[cfg(feature = "wb-hci")]
impl<'a> stm32wb_hci::Controller for Ble<'a> {
    async fn controller_write(&mut self, opcode: stm32wb_hci::Opcode, payload: &[u8]) {
        self.tl_write(opcode.0, payload).await;
    }

    async fn controller_read_into(&mut self, buf: &mut [u8]) {
        let evt_box = self.tl_read().await;
        let evt_serial = evt_box.serial();

        buf[..evt_serial.len()].copy_from_slice(evt_serial);
    }
}

impl<'a> BleTx<'a> {
    /// `TL_BLE_SendCmd`
    pub async fn tl_write(&mut self, opcode: u16, payload: &[u8]) {
        self.hw_ipcc_ble_cmd_channel
            .send(|| unsafe {
                CmdPacket::write_into(BLE_CMD_BUFFER.as_mut_ptr(), TlPacketType::BleCmd, opcode, payload);
            })
            .await;
    }

    /// `TL_BLE_SendAclData`
    pub async fn acl_write(&mut self, handle: u16, payload: &[u8]) {
        self.ipcc_hci_acl_tx_data_channel
            .send(|| unsafe {
                CmdPacket::write_into(
                    HCI_ACL_DATA_BUFFER.as_mut_ptr() as *mut _,
                    TlPacketType::AclData,
                    handle,
                    payload,
                );
            })
            .await;
    }
}

impl<'a> BleRx<'a> {
    /// `HW_IPCC_BLE_EvtNot`
    pub async fn tl_read(&mut self) -> EvtBox<Ble<'a>> {
        self.ipcc_ble_event_channel
            .receive(|| unsafe {
                if let Some(node_ptr) =
                    critical_section::with(|cs| LinkedListNode::remove_head(cs, EVT_QUEUE.as_mut_ptr()))
                {
                    Some(EvtBox::new(node_ptr.cast()))
                } else {
                    None
                }
            })
            .await
    }

    /// `TL_BLE_AclNot`
    pub async fn acl_read(&mut self) -> EvtBox<Ble<'a>> {
        ACL_EVT_OUT.wait_for_low().await;
        self.ipcc_hci_acl_rx_data_channel
            .receive(|| unsafe { Some(EvtBox::new(HCI_ACL_DATA_BUFFER.as_mut_ptr() as *mut _)) })
            .await
    }
}

#[cfg(feature = "wb-hci")]
/// Implement Controller for TX (Write only)
impl<'a> stm32wb_hci::Controller for BleTx<'a> {
    async fn controller_write(&mut self, opcode: stm32wb_hci::Opcode, payload: &[u8]) {
        self.tl_write(opcode.0, payload).await;
    }

    async fn controller_read_into(&mut self, _buf: &mut [u8]) {
        panic!("BleTx cannot read!");
    }
}

#[cfg(feature = "wb-hci")]
/// Implement Controller for RX (Read only)
impl<'a> stm32wb_hci::Controller for BleRx<'a> {
    async fn controller_write(&mut self, _opcode: stm32wb_hci::Opcode, _payload: &[u8]) {
        panic!("BleRx cannot write!");
    }

    async fn controller_read_into(&mut self, buf: &mut [u8]) {
        let evt_box = self.tl_read().await;
        let evt_serial = evt_box.serial();
        buf[..evt_serial.len()].copy_from_slice(evt_serial);
    }
}

#[cfg(feature = "bt-hci")]
pub struct ControllerAdapter<'d> {
    hw_ipcc_ble_cmd_channel: Mutex<NoopRawMutex, IpccTxChannel<'d>>,
    ipcc_ble_event_channel: Mutex<NoopRawMutex, IpccRxChannel<'d>>,
    ipcc_hci_acl_tx_data_channel: Mutex<NoopRawMutex, IpccTxChannel<'d>>,
    ipcc_hci_acl_rx_data_channel: Mutex<NoopRawMutex, IpccRxChannel<'d>>,
    slot: blocking_mutex::NoopMutex<RefCell<Option<bt_hci::cmd::Opcode>>>,
    signal: Signal<NoopRawMutex, Option<EvtBox<Ble<'d>>>>,
    waker: AtomicWaker,
    pending_evt: blocking_mutex::NoopMutex<RefCell<Option<EvtBox<Ble<'d>>>>>,
    cc_no_status: AtomicBool,
}

#[cfg(feature = "bt-hci")]
impl<'d> embedded_io::ErrorType for ControllerAdapter<'d> {
    type Error = embedded_io::ErrorKind;
}

#[cfg(feature = "bt-hci")]
struct SlotGuard<'d, 'a> {
    controller: &'a ControllerAdapter<'d>,
}

#[cfg(feature = "bt-hci")]
impl<'d, 'a> Drop for SlotGuard<'d, 'a> {
    fn drop(&mut self) {
        self.controller.slot.borrow().borrow_mut().take();
        self.controller.signal.reset();
        self.controller.waker.wake();
    }
}

#[cfg(feature = "bt-hci")]
impl<'d> ControllerAdapter<'d> {
    pub fn new(controller: Ble<'d>) -> Self {
        Self {
            hw_ipcc_ble_cmd_channel: Mutex::new(controller.hw_ipcc_ble_cmd_channel),
            ipcc_ble_event_channel: Mutex::new(controller.ipcc_ble_event_channel),
            ipcc_hci_acl_tx_data_channel: Mutex::new(controller.ipcc_hci_acl_tx_data_channel),
            ipcc_hci_acl_rx_data_channel: Mutex::new(controller.ipcc_hci_acl_rx_data_channel),
            slot: blocking_mutex::NoopMutex::const_new(NoopRawMutex::new(), RefCell::new(None)),
            signal: Signal::new(),
            waker: AtomicWaker::new(),
            pending_evt: blocking_mutex::NoopMutex::const_new(NoopRawMutex::new(), RefCell::new(None)),
            cc_no_status: AtomicBool::new(false),
        }
    }

    async fn grab_slot<'a>(&'a self, opcode: bt_hci::cmd::Opcode) -> SlotGuard<'d, 'a> {
        use core::task::Poll;

        poll_fn(|cx| {
            self.waker.register(cx.waker());

            let mut slot = self.slot.borrow().borrow_mut();

            if slot.is_some() || self.cc_no_status.load(Ordering::Acquire) {
                Poll::Pending
            } else {
                *slot = Some(opcode);

                Poll::Ready(SlotGuard { controller: self })
            }
        })
        .await
    }

    fn signal_cmd(&self, opcode: bt_hci::cmd::Opcode, evt: EvtBox<Ble<'d>>) {
        use bt_hci::cmd::Cmd;
        use bt_hci::cmd::controller_baseband::Reset;

        let slot = self.slot.borrow().borrow_mut();

        if let Some(waiting_opcode) = *slot
            && waiting_opcode == opcode
        {
            self.signal.signal(Some(evt));
        } else if slot.is_some() && opcode == Reset::OPCODE {
            self.signal.signal(None);
        }
    }

    async fn read_pkt(
        &self,
    ) -> Result<(EvtBox<Ble<'d>>, bt_hci::ControllerToHostPacket<'static>), embedded_io::ErrorKind> {
        use bt_hci::{ControllerToHostPacket, FromHciBytes};

        use crate::util::to_err;

        let evt: EvtBox<Ble<'d>> = self
            .ipcc_ble_event_channel
            .lock()
            .await
            .receive(|| unsafe {
                if let Some(node_ptr) =
                    critical_section::with(|cs| LinkedListNode::remove_head(cs, EVT_QUEUE.as_mut_ptr()))
                {
                    Some(EvtBox::new(node_ptr.cast()))
                } else {
                    None
                }
            })
            .await;

        let serial = evt.serial();
        let buf = unsafe { core::slice::from_raw_parts(serial as *const _ as *const u8, serial.len()) };

        Ok((
            evt,
            ControllerToHostPacket::from_hci_bytes(buf)
                .map_err(to_err)
                .map(|(pkt, _)| pkt)?,
        ))
    }

    async fn read_status(
        &self,
        _guard: &SlotGuard<'d, '_>,
    ) -> Result<bt_hci::event::CommandCompleteWithStatus<'_>, bt_hci::cmd::Error<embedded_io::ErrorKind>> {
        use bt_hci::cmd::Error as CmdError;
        use bt_hci::param::Error as ParamError;

        use crate::util::make_cc_with_cs;

        let evt = self
            .signal
            .wait()
            .await
            .ok_or(CmdError::Hci(ParamError::OPERATION_CANCELLED_BY_HOST))?;

        // Packets with CC or CS opcode are not managed by the memory manager
        let evt_serial = evt.serial();
        let evt_serial = unsafe { core::slice::from_raw_parts(evt_serial as *const _ as *const u8, evt_serial.len()) };

        make_cc_with_cs(evt_serial)
    }
}

#[cfg(feature = "bt-hci")]
impl<'d> bt_hci::controller::Controller for ControllerAdapter<'d> {
    async fn write_acl_data(&self, packet: &bt_hci::data::AclPacket<'_>) -> Result<(), Self::Error> {
        use bt_hci::WriteHci;
        use bt_hci::transport::WithIndicator;

        self.ipcc_hci_acl_tx_data_channel
            .lock()
            .await
            .send_exclusive(|| unsafe {
                WithIndicator::new(packet).write_hci(CmdPacket::writer(HCI_ACL_DATA_BUFFER.as_mut_ptr() as *mut _))
            })
            .await
    }

    async fn write_iso_data(&self, _packet: &bt_hci::data::IsoPacket<'_>) -> Result<(), Self::Error> {
        todo!()
    }

    async fn write_sync_data(&self, _packet: &bt_hci::data::SyncPacket<'_>) -> Result<(), Self::Error> {
        todo!()
    }

    async fn read<'a>(&self, buf: &'a mut [u8]) -> Result<bt_hci::ControllerToHostPacket<'a>, Self::Error> {
        use bt_hci::event::{CommandComplete, CommandStatus, EventKind};
        use bt_hci::{ControllerToHostPacket, FromHciBytes};
        use embassy_futures::select::{Either, select};

        use crate::util::to_err;

        match select(
            async {
                loop {
                    // Drop the pending evt so that the memory manager can clean it up
                    self.pending_evt.borrow().borrow_mut().take();
                    if self.cc_no_status.swap(false, Ordering::AcqRel) {
                        self.waker.wake();
                    }

                    let (evt, pkt) = self.read_pkt().await?;

                    match pkt {
                        ControllerToHostPacket::Event(ref event) => match event.kind {
                            EventKind::CommandComplete => {
                                let e = CommandComplete::from_hci_bytes_complete(event.data).map_err(to_err)?;
                                if !e.has_status() {
                                    // Store the pending event and block commands until the next read
                                    self.cc_no_status.store(true, Ordering::Release);
                                    self.pending_evt.borrow().borrow_mut().replace(evt);

                                    return Ok(pkt);
                                }

                                self.signal_cmd(e.cmd_opcode, evt);
                                continue;
                            }
                            EventKind::CommandStatus => {
                                let e = CommandStatus::from_hci_bytes_complete(event.data).map_err(to_err)?;

                                self.signal_cmd(e.cmd_opcode, evt);
                                continue;
                            }
                            _ => {
                                // Store the pending event so that it isn't dropped until the next read
                                self.pending_evt.borrow().borrow_mut().replace(evt);

                                return Ok(pkt);
                            }
                        },
                        _ => {
                            // Store the pending event so that it isn't dropped until the next read
                            self.pending_evt.borrow().borrow_mut().replace(evt);

                            return Ok(pkt);
                        }
                    }
                }
            },
            async {
                self.ipcc_hci_acl_rx_data_channel
                    .lock()
                    .await
                    .receive(|| unsafe {
                        // We must copy out the event immediately so that it is not trashed by a pending command.

                        let buf = core::slice::from_raw_parts_mut(buf as *mut _ as *mut u8, buf.len());
                        let evt: EvtBox<Ble<'d>> = EvtBox::new(HCI_ACL_DATA_BUFFER.as_mut_ptr() as *mut _);
                        let serial = evt.serial();
                        buf[..serial.len()].copy_from_slice(serial);

                        // rx will be cleared by evt box drop

                        Some(
                            ControllerToHostPacket::from_hci_bytes(buf)
                                .map(|(pkt, _)| pkt)
                                .map_err(to_err),
                        )
                    })
                    .await
            },
        )
        .await
        {
            Either::First(ret) => ret,
            Either::Second(ret) => ret,
        }
    }
}

#[cfg(feature = "bt-hci")]
impl<'d, C> bt_hci::controller::ControllerCmdSync<C> for ControllerAdapter<'d>
where
    C: bt_hci::cmd::SyncCmd,
{
    async fn exec(&self, cmd: &C) -> Result<C::Return, bt_hci::cmd::Error<Self::Error>> {
        use bt_hci::cmd::Error as CmdError;
        use bt_hci::transport::WithIndicator;
        use bt_hci::{WriteHci, cmd};

        debug!("Executing sync command with opcode {:x}", C::OPCODE.0);

        let guard = self.grab_slot(C::OPCODE).await;

        self.hw_ipcc_ble_cmd_channel
            .lock()
            .await
            .send(|| unsafe {
                WithIndicator::new(cmd)
                    .write_hci(CmdPacket::writer(BLE_CMD_BUFFER.as_mut_ptr()))
                    .map_err(CmdError::Io)
            })
            .await?;

        let e = self.read_status(&guard).await?;

        trace!("returned ccws: {:?}", e.status);

        let r = e.to_result::<C>().map_err(cmd::Error::Hci)?;
        debug!("Done executing command with opcode {:x}", C::OPCODE.0);
        Ok(r)
    }
}

#[cfg(feature = "bt-hci")]
impl<'d, C> bt_hci::controller::ControllerCmdAsync<C> for ControllerAdapter<'d>
where
    C: bt_hci::cmd::AsyncCmd,
{
    async fn exec(&self, cmd: &C) -> Result<(), bt_hci::cmd::Error<Self::Error>> {
        use bt_hci::WriteHci;
        use bt_hci::cmd::Error as CmdError;
        use bt_hci::transport::WithIndicator;

        debug!("Executing async command with opcode {:x}", C::OPCODE.0);

        let guard = self.grab_slot(C::OPCODE).await;

        self.hw_ipcc_ble_cmd_channel
            .lock()
            .await
            .send(|| unsafe {
                WithIndicator::new(cmd)
                    .write_hci(CmdPacket::writer(BLE_CMD_BUFFER.as_mut_ptr()))
                    .map_err(CmdError::Io)
            })
            .await?;

        let e = self.read_status(&guard).await?;

        trace!("returned ccws: {:?}", e.status);

        e.status.to_result()?;

        debug!("Done executing command with opcode {:x}", C::OPCODE.0);
        Ok(())
    }
}
