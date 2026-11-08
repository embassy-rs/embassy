extern crate std;

use core::cell::Cell;
use core::pin::pin;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::Waker;
use std::boxed::Box;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use std::task::Wake;
use std::vec::Vec;

use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::{Driver, PacketBuf, RawPacketBuf};

use super::*;
use crate::runner::{BusType, Runner, SealedBus};
use crate::structs::SdpcmHeader;
use crate::util::WriteBuffer;

const BASE: u32 = 0x4000;
const RECORD_COUNT: u32 = 6;
const WRITE_POINTER: u32 = RECORD_COUNT * 8;

#[derive(Debug)]
enum Op {
    BpRead(u32, Vec<u8>),
    BpRead32(u32, u32),
    BpWrite32(u32, u32),
    Read16(u8, u32, u16),
    Read32(u8, u32, u32),
    WlanRead(Vec<u8>),
    WlanWrite { pause: bool },
}

struct BusState {
    ops: RefCell<VecDeque<Op>>,
    published: Cell<u32>,
    wlan_reads: Cell<usize>,
    wlan_writes: Cell<usize>,
    irq_pending: Cell<bool>,
    irq_waker: RefCell<Option<Waker>>,
    interrupts: Cell<usize>,
}

impl BusState {
    fn next(&self) -> Op {
        self.ops.borrow_mut().pop_front().expect("unexpected bus operation")
    }

    fn interrupt(&self) {
        self.irq_pending.set(true);
        if let Some(waker) = self.irq_waker.borrow_mut().take() {
            waker.wake();
        }
    }
}

struct ScriptBus(Rc<BusState>);

impl SealedBus for ScriptBus {
    const TYPE: BusType = BusType::Spi;

    async fn init<'a>(&mut self, _: bool) -> crate::Result<()> {
        core::panic!("unexpected init")
    }

    async fn wlan_read(&mut self, data: &mut Aligned<A4, [u8]>) -> crate::Result<()> {
        match self.0.next() {
            Op::WlanRead(bytes) => {
                core::assert_eq!(data.len(), bytes.len());
                data.copy_from_slice(&bytes);
                self.0.wlan_reads.set(self.0.wlan_reads.get() + 1);
                Ok(())
            }
            op => core::panic!("unexpected wlan_read; expected {:?}", op),
        }
    }

    async fn wlan_write(&mut self, data: &mut WriteBuffer) -> crate::Result<()> {
        match self.0.next() {
            Op::WlanWrite { pause } => {
                core::assert_eq!(data.buf()[5], CHANNEL_TYPE_DATA);
                self.0.wlan_writes.set(self.0.wlan_writes.get() + 1);
                if pause {
                    core::future::pending::<()>().await;
                }
                Ok(())
            }
            op => core::panic!("unexpected wlan_write; expected {:?}", op),
        }
    }

    async fn bp_read(&mut self, addr: u32, data: &mut [u8], _: &mut Aligned<A4, [u8]>) -> crate::Result<()> {
        #[cfg(feature = "firmware-logs")]
        if addr == 0 && data.len() == crate::structs::SharedMemLog::SIZE {
            // No firmware console records in this controller script.
            data.fill(0);
            return Ok(());
        }
        match self.0.next() {
            Op::BpRead(expected, bytes) => {
                core::assert_eq!(addr, expected);
                core::assert_eq!(data.len(), bytes.len());
                data.copy_from_slice(&bytes);
                Ok(())
            }
            op => core::panic!("unexpected bp_read; expected {:?}", op),
        }
    }

    async fn bp_write(&mut self, _: u32, _: &[u8], _: &mut Aligned<A4, [u8]>) -> crate::Result<()> {
        core::panic!("unexpected bp_write")
    }

    async fn bp_read8(&mut self, _: u32) -> u8 {
        core::panic!("unexpected bp_read8")
    }

    async fn bp_write8(&mut self, _: u32, _: u8) {
        core::panic!("unexpected bp_write8")
    }

    async fn bp_read16(&mut self, _: u32) -> u16 {
        core::panic!("unexpected bp_read16")
    }

    async fn bp_write16(&mut self, _: u32, _: u16) {
        core::panic!("unexpected bp_write16")
    }

    async fn bp_read32(&mut self, addr: u32) -> u32 {
        match self.0.next() {
            Op::BpRead32(expected, value) => {
                core::assert_eq!(addr, expected);
                value
            }
            op => core::panic!("unexpected bp_read32; expected {:?}", op),
        }
    }

    async fn bp_write32(&mut self, addr: u32, value: u32) {
        match self.0.next() {
            Op::BpWrite32(expected_addr, expected_value) => {
                core::assert_eq!((addr, value), (expected_addr, expected_value));
                if addr == BASE + BTSDIO_OFFSET_BT2HOST_OUT {
                    self.0.published.set(value);
                }
            }
            op => core::panic!("unexpected bp_write32; expected {:?}", op),
        }
    }

    async fn read8(&mut self, _: u8, _: u32) -> u8 {
        core::panic!("unexpected read8")
    }

    async fn write8(&mut self, _: u8, _: u32, _: u8) {
        core::panic!("unexpected write8")
    }

    async fn read16(&mut self, func: u8, addr: u32) -> u16 {
        match self.0.next() {
            Op::Read16(expected_func, expected_addr, value) => {
                core::assert_eq!((func, addr), (expected_func, expected_addr));
                value
            }
            op => core::panic!("unexpected read16; expected {:?}", op),
        }
    }

    async fn write16(&mut self, _: u8, _: u32, _: u16) {
        core::panic!("unexpected write16")
    }

    async fn read32(&mut self, func: u8, addr: u32) -> u32 {
        match self.0.next() {
            Op::Read32(expected_func, expected_addr, value) => {
                core::assert_eq!((func, addr), (expected_func, expected_addr));
                value
            }
            op => core::panic!("unexpected read32; expected {:?}", op),
        }
    }

    async fn write32(&mut self, _: u8, _: u32, _: u32) {
        core::panic!("unexpected write32")
    }

    async fn wait_for_event(&mut self) {
        core::future::poll_fn(|cx| {
            if self.0.irq_pending.replace(false) {
                self.0.interrupts.set(self.0.interrupts.get() + 1);
                Poll::Ready(())
            } else {
                *self.0.irq_waker.borrow_mut() = Some(cx.waker().clone());
                Poll::Pending
            }
        })
        .await
    }
}

#[derive(Default)]
struct WakeCount(AtomicUsize);

impl Wake for WakeCount {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

fn wifi_and_bt_interrupt(ops: &mut Vec<Op>, credit: u8) {
    let len = SdpcmHeader::SIZE as u16;
    let frame = SdpcmHeader {
        len,
        len_inv: !len,
        sequence: 0,
        channel_and_flags: CHANNEL_TYPE_DATA,
        next_length: 0,
        header_length: SdpcmHeader::SIZE as u8,
        wireless_flow_control: 0,
        bus_data_credit: credit,
        reserved: [0; 2],
    };
    let interrupt = CHIP.sdiod_core_base_address + SDIO_INT_STATUS;
    ops.extend([
        Op::Read16(FUNC_BUS, REG_BUS_INTERRUPT, IRQ_F2_PACKET_AVAILABLE),
        Op::Read32(
            FUNC_BUS,
            SPI_STATUS_REGISTER,
            STATUS_F2_PKT_AVAILABLE | ((len as u32) << STATUS_F2_PKT_LEN_SHIFT),
        ),
        Op::WlanRead(frame.to_bytes().to_vec()),
        Op::Read32(FUNC_BUS, SPI_STATUS_REGISTER, 0),
        Op::BpRead32(interrupt, I_HMB_FC_CHANGE),
        Op::BpWrite32(interrupt, I_HMB_FC_CHANGE),
    ]);
}

fn record_header(ops: &mut Vec<Op>, index: u32) {
    ops.extend([
        Op::BpRead32(BASE + BTSDIO_OFFSET_BT2HOST_IN, WRITE_POINTER),
        Op::BpRead(BASE + BTSDIO_OFFSET_HOST_READ_BUF + index * 8, std::vec![2, 0, 0, 4]),
    ]);
}

fn receive_record(ops: &mut Vec<Op>, index: u32) {
    record_header(ops, index);
    ops.extend([
        Op::BpRead(
            BASE + BTSDIO_OFFSET_HOST_READ_BUF + index * 8 + 4,
            std::vec![0x10 + index as u8, 0, 0, 0],
        ),
        Op::BpWrite32(BASE + BTSDIO_OFFSET_BT2HOST_OUT, (index + 1) * 8),
        Op::BpWrite32(
            HOST_CTRL_REG_ADDR,
            if index % 2 == 0 {
                BTSDIO_REG_DATA_VALID_BITMASK
            } else {
                0
            },
        ),
    ]);
}

fn host_receive(host: &BtDriver<'_>, expected: u8) {
    let mut receiver = host.rx.borrow_mut();
    let slot = receiver.try_receive().expect("missing HCI event");
    core::assert_eq!(&slot.buf[..slot.len], &[4, expected, 0]);
    slot.receive_done();
}

fn full_rx_queue_case(credit: u8) {
    let mut ops = Vec::new();
    wifi_and_bt_interrupt(&mut ops, credit);
    for index in 0..4 {
        receive_record(&mut ops, index);
    }
    // Four host slots fill; neither pointer may consume the fifth record.
    record_header(&mut ops, 4);

    // A later WiFi frame and BT interrupt must still be handled while full.
    // In particular, the new BT interrupt must be acknowledged even though
    // the previous drain remains pending, or the level IRQ could spin.
    wifi_and_bt_interrupt(&mut ops, credit);
    record_header(&mut ops, 4);

    for index in 4..RECORD_COUNT {
        // The host releases one slot at a time, without another hardware IRQ.
        ops.extend([
            Op::Read16(FUNC_BUS, REG_BUS_INTERRUPT, 0),
            Op::BpRead32(CHIP.sdiod_core_base_address + SDIO_INT_STATUS, 0),
        ]);
        receive_record(&mut ops, index);
        if index + 1 == RECORD_COUNT {
            ops.push(Op::BpRead32(BASE + BTSDIO_OFFSET_BT2HOST_IN, WRITE_POINTER));
        } else {
            // The next record must remain pending when the queue fills again.
            record_header(&mut ops, index + 1);
        }
    }

    let bus = Rc::new(BusState {
        ops: RefCell::new(ops.into()),
        published: Cell::new(0),
        wlan_reads: Cell::new(0),
        wlan_writes: Cell::new(0),
        irq_pending: Cell::new(true),
        irq_waker: RefCell::new(None),
        interrupts: Cell::new(0),
    });
    let mut state = crate::State::new();
    let mut bt_state = BtState::new();
    let (mut bt, host) = new(&mut bt_state);
    bt.addr = BASE;
    let (channel, _device) = ch::new(
        &mut state.net.ch,
        ch::driver::HardwareAddress::Ethernet([0; 6]),
        crate::MTU,
    );
    let runner = Runner::new(
        channel,
        ScriptBus(bus.clone()),
        Cyw43439,
        &state.ioctl_state,
        &state.net.events,
        &state.net.secure_network,
        Some(bt),
    );
    let wakes = Arc::new(WakeCount::default());
    let waker = Waker::from(wakes.clone());
    let mut cx = Context::from_waker(&waker);
    let mut running = pin!(runner.run());

    core::assert!(running.as_mut().poll(&mut cx).is_pending());
    core::assert_eq!(bus.published.get(), 32);
    core::assert_eq!(bus.wlan_reads.get(), 1);
    core::assert!(host.rx.borrow().is_full());
    core::assert_eq!(wakes.0.load(Ordering::Relaxed), 0);

    // Full queues neither touch the bus again nor self-wake the executor.
    let remaining = bus.ops.borrow().len();
    for _ in 0..3 {
        core::assert!(running.as_mut().poll(&mut cx).is_pending());
        core::assert_eq!(bus.ops.borrow().len(), remaining);
        core::assert_eq!(wakes.0.load(Ordering::Relaxed), 0);
    }

    bus.interrupt();
    core::assert_eq!(wakes.0.swap(0, Ordering::Relaxed), 1);
    core::assert!(running.as_mut().poll(&mut cx).is_pending());
    core::assert_eq!(bus.wlan_reads.get(), 2, "a full BT queue must not block WiFi RX");
    core::assert_eq!(bus.published.get(), 32);
    core::assert_eq!(bus.interrupts.get(), 2);
    core::assert_eq!(wakes.0.load(Ordering::Relaxed), 0);

    for index in 0..2 {
        host_receive(&host, 0x10 + index);
        core::assert_eq!(
            wakes.0.swap(0, Ordering::Relaxed),
            1,
            "queue space must wake the runner"
        );
        core::assert!(running.as_mut().poll(&mut cx).is_pending());
        core::assert_eq!(bus.published.get(), 40 + u32::from(index) * 8);
        core::assert_eq!(bus.interrupts.get(), 2, "continuation must not need a new IRQ");
        core::assert!(host.rx.borrow().is_full());
        core::assert_eq!(wakes.0.load(Ordering::Relaxed), 0);
    }

    // All six records arrive exactly once and in order, including both that
    // stayed in SRAM until host queue space was released.
    for index in 2..RECORD_COUNT as u8 {
        host_receive(&host, 0x10 + index);
    }
    core::assert!(host.rx.borrow().is_empty());
    core::assert!(bus.ops.borrow().is_empty());
    core::assert!(running.as_mut().poll(&mut cx).is_pending());
    core::assert_eq!(wakes.0.load(Ordering::Relaxed), 0, "the completed drain must disarm");
}

#[test]
fn full_rx_queue_defers_and_resumes_with_wifi_credit() {
    full_rx_queue_case(1);
}

#[test]
fn full_rx_queue_defers_and_resumes_without_wifi_credit() {
    full_rx_queue_case(0);
}

fn wifi_packet() -> PacketBuf {
    // Match xarxa-driver's heap-backed test pool: the raw descriptor is the
    // first field, and freeing the packet releases its descriptor and storage.
    #[repr(C)]
    struct TestBuf {
        raw: MaybeUninit<RawPacketBuf>,
        data: [u8; 4],
    }

    unsafe fn free(raw: NonNull<RawPacketBuf>) {
        // SAFETY: this descriptor is the first field of the allocation below,
        // whose ownership was transferred exclusively to PacketBuf.
        unsafe { drop(Box::from_raw(raw.cast::<TestBuf>().as_ptr())) };
    }

    let mut buf = Box::new(TestBuf {
        raw: MaybeUninit::uninit(),
        data: [0x55; 4],
    });
    let data = NonNull::from(&mut buf.data).cast();
    buf.raw.write(RawPacketBuf {
        free,
        pool: core::ptr::null(),
        data,
        capacity: 4,
        headroom: 0,
        len: 4,
        meta: Default::default(),
    });
    let raw = NonNull::new(Box::into_raw(buf)).unwrap().cast();
    // SAFETY: the fresh allocation and initialized bytes have one owner until
    // free reclaims them; headroom + len equals the storage capacity.
    unsafe { PacketBuf::from_raw(raw) }
}

#[test]
fn deferred_rx_progresses_while_wifi_transmit_stays_ready() {
    let mut ops = Vec::new();
    wifi_and_bt_interrupt(&mut ops, 8);
    for index in 0..4 {
        receive_record(&mut ops, index);
    }
    record_header(&mut ops, 4);
    ops.extend([
        // The runner is already parked in select4 when WiFi TX becomes ready.
        Op::WlanWrite { pause: false },
        Op::Read32(FUNC_BUS, SPI_STATUS_REGISTER, 0),
        // The next loop must service deferred RX before another ready WiFi TX.
        Op::Read16(FUNC_BUS, REG_BUS_INTERRUPT, 0),
        Op::BpRead32(CHIP.sdiod_core_base_address + SDIO_INT_STATUS, 0),
    ]);
    receive_record(&mut ops, 4);
    record_header(&mut ops, 5);
    // Park in the second WLAN write to observe progress while two further
    // packets remain queued and six WiFi credits still remain.
    ops.push(Op::WlanWrite { pause: true });

    let bus = Rc::new(BusState {
        ops: RefCell::new(ops.into()),
        published: Cell::new(0),
        wlan_reads: Cell::new(0),
        wlan_writes: Cell::new(0),
        irq_pending: Cell::new(true),
        irq_waker: RefCell::new(None),
        interrupts: Cell::new(0),
    });
    let mut state = crate::State::new();
    let mut bt_state = BtState::new();
    let (mut bt, host) = new(&mut bt_state);
    bt.addr = BASE;
    let (channel, mut device) = ch::new(
        &mut state.net.ch,
        ch::driver::HardwareAddress::Ethernet([0; 6]),
        crate::MTU,
    );
    let runner = Runner::new(
        channel,
        ScriptBus(bus.clone()),
        Cyw43439,
        &state.ioctl_state,
        &state.net.events,
        &state.net.secure_network,
        Some(bt),
    );
    let mut cx = Context::from_waker(Waker::noop());
    let mut running = pin!(runner.run());
    core::assert!(running.as_mut().poll(&mut cx).is_pending());
    core::assert_eq!(bus.published.get(), 32);
    core::assert!(host.rx.borrow().is_full());

    for _ in 0..4 {
        core::assert!(device.transmit(wifi_packet()).is_ok());
    }
    host_receive(&host, 0x10);
    core::assert!(running.as_mut().poll(&mut cx).is_pending());
    core::assert_eq!(bus.published.get(), 40, "ready WiFi TX must not starve deferred BT RX");
    core::assert_eq!(bus.wlan_writes.get(), 2);
    core::assert_eq!(bus.interrupts.get(), 1, "no new IRQ was needed");
    core::assert!(bus.ops.borrow().is_empty());
    for index in 1..5 {
        host_receive(&host, 0x10 + index);
    }
}
