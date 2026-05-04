#[cfg(feature = "bt-hci")]
use core::cell::RefCell;
use core::ptr;
#[cfg(feature = "bt-hci")]
use core::sync::atomic::AtomicBool;

use embassy_stm32::ipcc::{IpccRxChannel, IpccTxChannel};
#[cfg(feature = "bt-hci")]
use embassy_sync::blocking_mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(feature = "bt-hci")]
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::{Mutex, MutexGuard};
#[cfg(feature = "bt-hci")]
use embassy_sync::signal::Signal;
use hci::Opcode;

use crate::cmd::CmdPacket;
use crate::consts::{TL_BLEEVT_CC_OPCODE, TL_BLEEVT_CS_OPCODE, TlPacketType};
use crate::evt;
use crate::evt::{EvtBox, EvtPacket, EvtStub};
use crate::sub::mm;
use crate::tables::{BLE_CMD_BUFFER, BleTable, CS_BUFFER, EVT_QUEUE, HCI_ACL_DATA_BUFFER, TL_BLE_TABLE};
use crate::unsafe_linked_list::LinkedListNode;
use crate::wb::Flag;

static ACL_EVT_OUT: Flag = Flag::new(false);
static ACL_MUTEX: Mutex<CriticalSectionRawMutex, ()> = Mutex::new(());

async fn lock_acl() -> MutexGuard<'static, CriticalSectionRawMutex, ()> {
    ACL_EVT_OUT.wait_for_low().await;
    ACL_MUTEX.lock().await
}

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
            .receive(
                || unsafe {
                    if let Some(node_ptr) =
                        critical_section::with(|cs| LinkedListNode::remove_head(cs, EVT_QUEUE.as_mut_ptr()))
                    {
                        Some(EvtBox::new(node_ptr.cast()))
                    } else {
                        None
                    }
                },
                false,
            )
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
        let _guard = lock_acl().await;

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

    /// `TL_BLE_AclNot`
    pub async fn acl_read(&mut self) -> EvtBox<Self> {
        let _guard = lock_acl().await;

        self.ipcc_hci_acl_rx_data_channel
            .receive(
                || unsafe {
                    // The closure is not async, therefore the closure must execute to completion (cannot be dropped)
                    // Therefore, the event box is guaranteed to be cleaned up if it's not leaked
                    ACL_EVT_OUT.set_high();

                    Some(EvtBox::new(HCI_ACL_DATA_BUFFER.as_mut_ptr() as *mut _))
                },
                true,
            )
            .await
    }
}

impl<'a> evt::MemoryManager for Ble<'a> {
    /// SAFETY: passing a pointer to something other than a managed event packet is UB
    unsafe fn drop_event_packet(evt: *mut EvtPacket) {
        if ptr::eq(evt, HCI_ACL_DATA_BUFFER.as_mut_ptr() as *mut _) {
            ACL_EVT_OUT.set_low();

            return;
        }

        let stub = unsafe {
            let p_evt_stub = &(*evt).evt_serial as *const _ as *const EvtStub;

            ptr::read_volatile(p_evt_stub)
        };

        if !(stub.evt_code == TL_BLEEVT_CS_OPCODE || stub.evt_code == TL_BLEEVT_CC_OPCODE) {
            mm::MemoryManager::drop_event_packet(evt);
        }
    }
}

pub extern crate stm32wb_hci as hci;

impl<'a> hci::Controller for Ble<'a> {
    async fn controller_write(&mut self, opcode: Opcode, payload: &[u8]) {
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
            .receive(
                || unsafe {
                    if let Some(node_ptr) =
                        critical_section::with(|cs| LinkedListNode::remove_head(cs, EVT_QUEUE.as_mut_ptr()))
                    {
                        Some(EvtBox::new(node_ptr.cast()))
                    } else {
                        None
                    }
                },
                false,
            )
            .await
    }

    /// `TL_BLE_AclNot`
    pub async fn acl_read(&mut self) -> EvtBox<Ble<'a>> {
        let _guard = lock_acl().await;

        self.ipcc_hci_acl_rx_data_channel
            .receive(
                || unsafe {
                    // The closure is not async, therefore the closure must execute to completion (cannot be dropped)
                    // Therefore, the event box is guaranteed to be cleaned up if it's not leaked
                    ACL_EVT_OUT.set_high();

                    Some(EvtBox::new(HCI_ACL_DATA_BUFFER.as_mut_ptr() as *mut _))
                },
                true,
            )
            .await
    }
}

/// Implement Controller for TX (Write only)
impl<'a> hci::Controller for BleTx<'a> {
    async fn controller_write(&mut self, opcode: Opcode, payload: &[u8]) {
        self.tl_write(opcode.0, payload).await;
    }

    async fn controller_read_into(&mut self, _buf: &mut [u8]) {
        panic!("BleTx cannot read!");
    }
}

/// Implement Controller for RX (Read only)
impl<'a> hci::Controller for BleRx<'a> {
    async fn controller_write(&mut self, _opcode: Opcode, _payload: &[u8]) {
        panic!("BleRx cannot write!");
    }

    async fn controller_read_into(&mut self, buf: &mut [u8]) {
        let evt_box = self.tl_read().await;
        let evt_serial = evt_box.serial();
        buf[..evt_serial.len()].copy_from_slice(evt_serial);
    }
}

#[cfg(feature = "bt-hci")]
const ERR0: bt_hci::cmd::Error<embedded_io::ErrorKind> = bt_hci::cmd::Error::Io(ERR1);
#[cfg(feature = "bt-hci")]
const ERR1: embedded_io::ErrorKind = embedded_io::ErrorKind::InvalidData;

#[cfg(feature = "bt-hci")]
const SLOTS: usize = 3;

#[cfg(feature = "bt-hci")]
pub struct AtomicController<'d> {
    hw_ipcc_ble_cmd_channel: Mutex<NoopRawMutex, IpccTxChannel<'d>>,
    ipcc_ble_event_channel: Mutex<NoopRawMutex, IpccRxChannel<'d>>,
    ipcc_hci_acl_tx_data_channel: Mutex<NoopRawMutex, IpccTxChannel<'d>>,
    ipcc_hci_acl_rx_data_channel: Mutex<NoopRawMutex, IpccRxChannel<'d>>,
    slots: blocking_mutex::NoopMutex<RefCell<[Option<bt_hci::cmd::Opcode>; SLOTS]>>,
    signals: [Signal<NoopRawMutex, Option<EvtBox<Ble<'d>>>>; SLOTS],
    acl_pending: AtomicBool,
}

#[cfg(feature = "bt-hci")]
impl<'d> AtomicController<'d> {
    pub const fn new(controller: Ble<'d>) -> Self {
        Self {
            hw_ipcc_ble_cmd_channel: Mutex::new(controller.hw_ipcc_ble_cmd_channel),
            ipcc_ble_event_channel: Mutex::new(controller.ipcc_ble_event_channel),
            ipcc_hci_acl_tx_data_channel: Mutex::new(controller.ipcc_hci_acl_tx_data_channel),
            ipcc_hci_acl_rx_data_channel: Mutex::new(controller.ipcc_hci_acl_rx_data_channel),
            slots: blocking_mutex::NoopMutex::const_new(NoopRawMutex::new(), RefCell::new([None; SLOTS])),
            signals: [Signal::new(), Signal::new(), Signal::new()],
            acl_pending: AtomicBool::new(false),
        }
    }

    fn grab_slot(&self, opcode: bt_hci::cmd::Opcode) -> (usize, &Signal<NoopRawMutex, Option<EvtBox<Ble<'d>>>>) {
        let mut slots = self.slots.borrow().borrow_mut();
        let ((index, slot), signal) = slots
            .iter_mut()
            .enumerate()
            .zip(self.signals.iter())
            .filter(|((_, slot), _)| slot.is_none())
            .next()
            .unwrap();

        *slot = Some(opcode);

        (index, signal)
    }
}

#[cfg(feature = "bt-hci")]
impl<'d> embedded_io::ErrorType for AtomicController<'d> {
    type Error = embedded_io::ErrorKind;
}

#[cfg(feature = "bt-hci")]
impl<'d> bt_hci::controller::Controller for AtomicController<'d> {
    async fn write_acl_data(&self, packet: &bt_hci::data::AclPacket<'_>) -> Result<(), Self::Error> {
        use bt_hci::WriteHci;
        use bt_hci::transport::WithIndicator;

        self.ipcc_hci_acl_tx_data_channel
            .lock()
            .await
            .send_exclusive(|| unsafe {
                WithIndicator::new(packet)
                    .write_hci(CmdPacket::writer(HCI_ACL_DATA_BUFFER.as_mut_ptr() as *mut _))
                    .map_err(|_| ERR0)
            })
            .await
            .map_err(|_| ERR1)
    }

    async fn write_iso_data(&self, _packet: &bt_hci::data::IsoPacket<'_>) -> Result<(), Self::Error> {
        todo!()
    }

    async fn write_sync_data(&self, _packet: &bt_hci::data::SyncPacket<'_>) -> Result<(), Self::Error> {
        todo!()
    }

    async fn read<'a>(&self, buf: &'a mut [u8]) -> Result<bt_hci::ControllerToHostPacket<'a>, Self::Error> {
        use core::sync::atomic::Ordering;

        use bt_hci::cmd::Cmd;
        use bt_hci::cmd::controller_baseband::Reset;
        use bt_hci::event::{CommandComplete, CommandCompleteWithStatus, CommandStatus, EventKind};
        use bt_hci::{ControllerToHostPacket, FromHciBytes};
        use embassy_futures::select::{Either, select};

        let signal_cmd = |opcode: bt_hci::cmd::Opcode, evt: EvtBox<Ble<'d>>| {
            let slots = self.slots.borrow().borrow_mut();

            if opcode == Reset::OPCODE {
                for (slot, signal) in slots.iter().zip(self.signals.iter()) {
                    if slot.is_some() {
                        signal.signal(None);
                    }
                }
            } else {
                for (slot, signal) in slots.iter().zip(self.signals.iter()) {
                    if let Some(waiting_opcode) = slot
                        && *waiting_opcode == opcode
                    {
                        signal.signal(Some(evt));

                        return;
                    }
                }
            }
        };

        let make_pkt =
            |buf: &'a mut [u8], evt: EvtBox<Ble<'d>>| -> Result<bt_hci::ControllerToHostPacket<'a>, Self::Error> {
                let serial = evt.serial();
                buf[..serial.len()].copy_from_slice(serial);

                let (pkt, _) = ControllerToHostPacket::from_hci_bytes(buf).map_err(|_| ERR1)?;

                Ok(pkt)
            };

        // SAFETY: buf is written and returned in both branches
        let buf2 = unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr(), buf.len()) };
        match select(
            async {
                loop {
                    let evt: EvtBox<Ble<'d>> = self
                        .ipcc_ble_event_channel
                        .lock()
                        .await
                        .receive(
                            || unsafe {
                                if let Some(node_ptr) =
                                    critical_section::with(|cs| LinkedListNode::remove_head(cs, EVT_QUEUE.as_mut_ptr()))
                                {
                                    Some(EvtBox::new(node_ptr.cast()))
                                } else {
                                    None
                                }
                            },
                            false,
                        )
                        .await;

                    let (pkt, _) = ControllerToHostPacket::from_hci_bytes(&evt.serial()).map_err(|_| ERR1)?;

                    match pkt {
                        ControllerToHostPacket::Event(ref event) => match event.kind {
                            EventKind::CommandComplete => {
                                let e = CommandComplete::from_hci_bytes_complete(event.data).map_err(|_| ERR1)?;
                                if !e.has_status() {
                                    return make_pkt(buf, evt);
                                }
                                let e: CommandCompleteWithStatus =
                                    e.try_into().map_err(|_| embedded_io::ErrorKind::InvalidData)?;

                                signal_cmd(e.cmd_opcode, evt);
                                continue;
                            }
                            EventKind::CommandStatus => {
                                let e = CommandStatus::from_hci_bytes_complete(event.data).map_err(|_| ERR1)?;

                                signal_cmd(e.cmd_opcode, evt);
                                continue;
                            }
                            _ => {
                                return make_pkt(buf, evt);
                            }
                        },
                        _ => {
                            return make_pkt(buf, evt);
                        }
                    }
                }
            },
            async {
                unsafe {
                    // This entire block is unsafe because we must copy out the event immediately so that it is not
                    // trashed by a pending command.

                    // When combined with the `close` flag, the expected behavior is that this will only receive
                    // data while the future is active.
                    let evt: EvtBox<Ble<'d>> = self
                        .ipcc_hci_acl_rx_data_channel
                        .lock()
                        .await
                        .receive(
                            || {
                                if !self.acl_pending.fetch_not(Ordering::AcqRel) {
                                    None
                                } else {
                                    Some(EvtBox::new(HCI_ACL_DATA_BUFFER.as_mut_ptr() as *mut _))
                                }
                            },
                            true,
                        )
                        .await;

                    make_pkt(buf2, evt)
                }
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
impl<'d, C> bt_hci::controller::ControllerCmdSync<C> for AtomicController<'d>
where
    C: bt_hci::cmd::SyncCmd,
{
    async fn exec(&self, cmd: &C) -> Result<C::Return, bt_hci::cmd::Error<Self::Error>> {
        use bt_hci::event::{CommandComplete, CommandCompleteWithStatus, EventKind};
        use bt_hci::transport::WithIndicator;
        use bt_hci::{ControllerToHostPacket, FromHciBytes, WriteHci, cmd};
        use embassy_hal_internal::drop::OnDrop;

        let (index, signal) = self.grab_slot(C::OPCODE);
        let _guard = OnDrop::new(|| {
            self.signals[index].reset();
            self.slots.borrow().borrow_mut()[index] = None
        });

        self.hw_ipcc_ble_cmd_channel
            .lock()
            .await
            .send(|| unsafe {
                WithIndicator::new(cmd)
                    .write_hci(CmdPacket::writer(BLE_CMD_BUFFER.as_mut_ptr()))
                    .map_err(|_| ERR0)
            })
            .await?;

        let evt = signal.wait().await.ok_or(ERR0)?;

        let (pkt, _) = ControllerToHostPacket::from_hci_bytes(&evt.serial()).map_err(|_| ERR0)?;

        let ControllerToHostPacket::Event(ref event) = pkt else {
            return Err(ERR0);
        };

        if event.kind != EventKind::CommandComplete {
            return Err(ERR0);
        }

        let e = CommandComplete::from_hci_bytes_complete(event.data).map_err(|_| ERR0)?;
        let e: CommandCompleteWithStatus = e.try_into().map_err(|_| ERR0)?;

        let r = e.to_result::<C>().map_err(cmd::Error::Hci)?;
        // info!("Done executing command with opcode {}", C::OPCODE);
        Ok(r)
    }
}

#[cfg(feature = "bt-hci")]
impl<'d, C> bt_hci::controller::ControllerCmdAsync<C> for AtomicController<'d>
where
    C: bt_hci::cmd::AsyncCmd,
{
    async fn exec(&self, cmd: &C) -> Result<(), bt_hci::cmd::Error<Self::Error>> {
        use bt_hci::event::{CommandStatus, EventKind};
        use bt_hci::transport::WithIndicator;
        use bt_hci::{ControllerToHostPacket, FromHciBytes, WriteHci};
        use embassy_hal_internal::drop::OnDrop;

        let (index, signal) = self.grab_slot(C::OPCODE);
        let _guard = OnDrop::new(|| {
            self.signals[index].reset();
            self.slots.borrow().borrow_mut()[index] = None
        });

        self.hw_ipcc_ble_cmd_channel
            .lock()
            .await
            .send(|| unsafe {
                WithIndicator::new(cmd)
                    .write_hci(CmdPacket::writer(BLE_CMD_BUFFER.as_mut_ptr()))
                    .map_err(|_| ERR0)
            })
            .await?;

        let evt = signal.wait().await.ok_or(ERR0)?;

        let (pkt, _) = ControllerToHostPacket::from_hci_bytes(&evt.serial()).map_err(|_| ERR0)?;

        let ControllerToHostPacket::Event(ref event) = pkt else {
            return Err(ERR0);
        };

        if event.kind != EventKind::CommandStatus {
            return Err(ERR0);
        }

        let e = CommandStatus::from_hci_bytes_complete(event.data).map_err(|_| ERR0)?;

        e.status.to_result()?;

        // info!("Done executing command with opcode {}", C::OPCODE);
        Ok(())
    }
}
