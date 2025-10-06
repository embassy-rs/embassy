#![no_std]
#![allow(unsafe_op_in_unsafe_fn)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![deny(unused_must_use)]

// must be first
mod fmt;

pub mod context;

use core::cell::RefCell;
use core::future::{Future, poll_fn};
use core::marker::PhantomData;
use core::mem::{self, MaybeUninit};
use core::ptr::{self, addr_of, addr_of_mut, copy_nonoverlapping};
use core::slice;
use core::sync::atomic::{Ordering, compiler_fence, fence};
use core::task::{Poll, Waker};

use cortex_m::peripheral::NVIC;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::pipe;
use embassy_sync::waitqueue::{AtomicWaker, WakerRegistration};
use heapless::Vec;
use {embassy_net_driver_channel as ch, nrf_pac as pac};

const RX_SIZE: usize = 8 * 1024;
const TRACE_SIZE: usize = 16 * 1024;
const TRACE_BUF: usize = 1024;
const MTU: usize = 1500;

/// Network driver.
///
/// This is the type you have to pass to `embassy-net` when creating the network stack.
pub type NetDriver<'a> = ch::Device<'a, MTU>;

static WAKER: AtomicWaker = AtomicWaker::new();

/// Call this function on IPC IRQ
pub fn on_ipc_irq() {
    trace!("irq");

    pac::IPC_NS.inten().write(|_| ());
    WAKER.wake();
}

struct Allocator<'a> {
    start: *mut u8,
    end: *mut u8,
    _phantom: PhantomData<&'a mut u8>,
}

impl<'a> Allocator<'a> {
    fn alloc_bytes(&mut self, size: usize) -> &'a mut [MaybeUninit<u8>] {
        // safety: both pointers come from the same allocation.
        let available_size = unsafe { self.end.offset_from(self.start) } as usize;
        if size > available_size {
            panic!("out of memory")
        }

        // safety: we've checked above this doesn't go out of bounds.
        let p = self.start;
        self.start = unsafe { p.add(size) };

        // safety: we've checked the pointer is in-bounds.
        unsafe { slice::from_raw_parts_mut(p as *mut _, size) }
    }

    fn alloc<T>(&mut self) -> &'a mut MaybeUninit<T> {
        let align = mem::align_of::<T>();
        let size = mem::size_of::<T>();

        let align_size = match (self.start as usize) % align {
            0 => 0,
            n => align - n,
        };

        // safety: both pointers come from the same allocation.
        let available_size = unsafe { self.end.offset_from(self.start) } as usize;
        if align_size + size > available_size {
            panic!("out of memory")
        }

        // safety: we've checked above this doesn't go out of bounds.
        let p = unsafe { self.start.add(align_size) };
        self.start = unsafe { p.add(size) };

        // safety: we've checked the pointer is aligned and in-bounds.
        unsafe { &mut *(p as *mut _) }
    }
}

/// Create a new nRF91 embassy-net driver.
pub async fn new<'a>(
    state: &'a mut State,
    shmem: &'a mut [MaybeUninit<u8>],
) -> (NetDriver<'a>, Control<'a>, Runner<'a>) {
    let (n, c, r, _) = new_internal(state, shmem, None).await;
    (n, c, r)
}

/// Create a new nRF91 embassy-net driver with trace.
pub async fn new_with_trace<'a>(
    state: &'a mut State,
    shmem: &'a mut [MaybeUninit<u8>],
    trace_buffer: &'a mut TraceBuffer,
) -> (NetDriver<'a>, Control<'a>, Runner<'a>, TraceReader<'a>) {
    let (n, c, r, t) = new_internal(state, shmem, Some(trace_buffer)).await;
    (n, c, r, t.unwrap())
}

/// Create a new nRF91 embassy-net driver.
async fn new_internal<'a>(
    state: &'a mut State,
    shmem: &'a mut [MaybeUninit<u8>],
    trace_buffer: Option<&'a mut TraceBuffer>,
) -> (NetDriver<'a>, Control<'a>, Runner<'a>, Option<TraceReader<'a>>) {
    let shmem_len = shmem.len();
    let shmem_ptr = shmem.as_mut_ptr() as *mut u8;

    const SPU_REGION_SIZE: usize = 8192; // 8kb
    trace!("  shmem_ptr = {}, shmem_len = {}", shmem_ptr, shmem_len);

    assert!(shmem_len != 0, "shmem length must not be zero");
    assert!(
        shmem_len % SPU_REGION_SIZE == 0,
        "shmem length must be a multiple of 8kb"
    );
    assert!(
        (shmem_ptr as usize) % SPU_REGION_SIZE == 0,
        "shmem pointer must be 8kb-aligned"
    );
    assert!(
        (shmem_ptr as usize + shmem_len) < 0x2002_0000,
        "shmem must be in the lower 128kb of RAM"
    );

    let spu = pac::SPU_S;
    debug!("Setting IPC RAM as nonsecure...");
    trace!(
        "  SPU_REGION_SIZE={}, shmem_ptr=0x{:08X}, shmem_len={}",
        SPU_REGION_SIZE, shmem_ptr as usize, shmem_len
    );
    let region_start = (shmem_ptr as usize - 0x2000_0000) / SPU_REGION_SIZE;
    let region_end = region_start + shmem_len / SPU_REGION_SIZE;
    trace!("  region_start={}, region_end={}", region_start, region_end);
    for i in region_start..region_end {
        spu.ramregion(i).perm().write(|w| {
            w.set_execute(true);
            w.set_write(true);
            w.set_read(true);
            w.set_secattr(false);
            w.set_lock(false);
        })
    }

    spu.periphid(42).perm().write(|w| w.set_secattr(false));

    let mut alloc = Allocator {
        start: shmem_ptr,
        end: unsafe { shmem_ptr.add(shmem_len) },
        _phantom: PhantomData,
    };
    trace!(
        "  Allocator: start=0x{:08X}, end=0x{:08X}",
        alloc.start as usize, alloc.end as usize
    );

    let cb: &mut ControlBlock = alloc.alloc().write(unsafe { mem::zeroed() });

    let rx = alloc.alloc_bytes(RX_SIZE);
    trace!("  RX buffer at {}, size={}", rx.as_ptr(), RX_SIZE);
    let trace = alloc.alloc_bytes(TRACE_SIZE);
    trace!("  Trace buffer at {}, size={}", trace.as_ptr(), TRACE_SIZE);

    cb.version = 0x00010000;
    cb.rx_base = rx.as_mut_ptr() as _;
    cb.rx_size = RX_SIZE;
    cb.control_list_ptr = &mut cb.lists[0];
    cb.data_list_ptr = &mut cb.lists[1];
    cb.modem_info_ptr = &mut cb.modem_info;
    cb.trace_ptr = &mut cb.trace;
    cb.lists[0].len = LIST_LEN;
    cb.lists[1].len = LIST_LEN;
    cb.trace.base = trace.as_mut_ptr() as _;
    cb.trace.size = TRACE_SIZE;

    let ipc = pac::IPC_NS;
    ipc.gpmem(0).write_value(cb as *mut _ as u32);
    ipc.gpmem(1).write_value(0);
    trace!("  GPMEM[0]={:#X}, GPMEM[1]={}", cb as *mut _ as u32, 0);

    // connect task/event i to channel i
    for i in 0..8 {
        ipc.send_cnf(i).write(|w| w.0 = 1 << i);
        ipc.receive_cnf(i).write(|w| w.0 = 1 << i);
    }

    compiler_fence(Ordering::SeqCst);

    let power = pac::POWER_S;
    // POWER.LTEMODEM.STARTN = 0
    // TODO: The reg is missing in the PAC??
    let startn = unsafe { (power.as_ptr() as *mut u32).add(0x610 / 4) };
    unsafe { startn.write_volatile(0) }

    unsafe { NVIC::unmask(pac::Interrupt::IPC) };

    let state_inner = &*state.inner.write(RefCell::new(StateInner {
        init: false,
        init_waker: WakerRegistration::new(),
        cb,
        requests: [const { None }; REQ_COUNT],
        next_req_serial: 0x12345678,
        net_fd: None,

        rx_control_list: ptr::null_mut(),
        rx_data_list: ptr::null_mut(),
        rx_control_len: 0,
        rx_data_len: 0,
        rx_seq_no: 0,
        rx_check: PointerChecker {
            start: rx.as_mut_ptr() as *mut u8,
            end: (rx.as_mut_ptr() as *mut u8).wrapping_add(RX_SIZE),
        },

        tx_seq_no: 0,
        tx_buf_used: [false; TX_BUF_COUNT],
        tx_waker: WakerRegistration::new(),

        trace_chans: Vec::new(),
        trace_check: PointerChecker {
            start: trace.as_mut_ptr() as *mut u8,
            end: (trace.as_mut_ptr() as *mut u8).wrapping_add(TRACE_SIZE),
        },
    }));

    let control = Control { state: state_inner };

    let (ch_runner, device) = ch::new(&mut state.ch, ch::driver::HardwareAddress::Ip);
    let state_ch = ch_runner.state_runner();
    state_ch.set_link_state(ch::driver::LinkState::Up);

    let (trace_reader, trace_writer) = if let Some(trace) = trace_buffer {
        let (r, w) = trace.trace.split();
        (Some(r), Some(w))
    } else {
        (None, None)
    };

    let runner = Runner {
        ch: ch_runner,
        state: state_inner,
        trace_writer,
    };

    (device, control, runner, trace_reader)
}

/// State holding modem traces.
pub struct TraceBuffer {
    trace: pipe::Pipe<NoopRawMutex, TRACE_BUF>,
}

/// Represents writer half of the trace buffer.
pub type TraceWriter<'a> = pipe::Writer<'a, NoopRawMutex, TRACE_BUF>;

/// Represents the reader half of the trace buffer.
pub type TraceReader<'a> = pipe::Reader<'a, NoopRawMutex, TRACE_BUF>;

impl TraceBuffer {
    /// Create a new TraceBuffer.
    pub const fn new() -> Self {
        Self {
            trace: pipe::Pipe::new(),
        }
    }
}

/// Shared state for the driver.
pub struct State {
    ch: ch::State<MTU, 4, 4>,
    inner: MaybeUninit<RefCell<StateInner>>,
}

impl State {
    /// Create a new State.
    pub const fn new() -> Self {
        Self {
            ch: ch::State::new(),
            inner: MaybeUninit::uninit(),
        }
    }
}

const TX_BUF_COUNT: usize = 4;
const TX_BUF_SIZE: usize = 1500;

struct TraceChannelInfo {
    ptr: *mut TraceChannel,
    start: *mut u8,
    end: *mut u8,
}

const REQ_COUNT: usize = 4;

struct PendingRequest {
    req_serial: u32,
    resp_msg: *mut Message,
    waker: Waker,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct NoFreeBufs;

struct StateInner {
    init: bool,
    init_waker: WakerRegistration,

    cb: *mut ControlBlock,
    requests: [Option<PendingRequest>; REQ_COUNT],
    next_req_serial: u32,

    net_fd: Option<u32>,

    rx_control_list: *mut List,
    rx_data_list: *mut List,
    /// Number of entries in the control list
    rx_control_len: usize,
    /// Number of entries in the data list
    rx_data_len: usize,
    rx_seq_no: u16,
    rx_check: PointerChecker,

    tx_seq_no: u16,
    tx_buf_used: [bool; TX_BUF_COUNT],
    tx_waker: WakerRegistration,

    trace_chans: Vec<TraceChannelInfo, TRACE_CHANNEL_COUNT>,
    trace_check: PointerChecker,
}

impl StateInner {
    fn poll(&mut self, trace_writer: &mut Option<TraceWriter<'_>>, ch: &mut ch::Runner<MTU>) {
        trace!("poll!");
        let ipc = pac::IPC_NS;

        if ipc.events_receive(0).read() != 0 {
            ipc.events_receive(0).write_value(0);
            trace!("ipc 0");
        }

        if ipc.events_receive(2).read() != 0 {
            ipc.events_receive(2).write_value(0);
            trace!("ipc 2");

            if !self.init {
                let desc = unsafe { addr_of!((*self.cb).modem_info).read_volatile() };
                assert_eq!(desc.version, 1);

                self.rx_check.check_mut(desc.control_list_ptr);
                self.rx_check.check_mut(desc.data_list_ptr);

                self.rx_control_list = desc.control_list_ptr;
                self.rx_data_list = desc.data_list_ptr;
                let rx_control_len = unsafe { addr_of!((*self.rx_control_list).len).read_volatile() };
                let rx_data_len = unsafe { addr_of!((*self.rx_data_list).len).read_volatile() };

                trace!("modem control list length: {}", rx_control_len);
                trace!("modem data    list length: {}", rx_data_len);
                self.rx_control_len = rx_control_len;
                self.rx_data_len = rx_data_len;
                self.init = true;

                debug!("IPC initialized OK!");
                self.init_waker.wake();
            }
        }

        if ipc.events_receive(4).read() != 0 {
            ipc.events_receive(4).write_value(0);
            trace!("ipc 4");

            loop {
                let list = unsafe { &mut *self.rx_control_list };
                let control_work = self.process(list, true, ch);
                let list = unsafe { &mut *self.rx_data_list };
                let data_work = self.process(list, false, ch);
                if !control_work && !data_work {
                    break;
                }
            }
        }

        if ipc.events_receive(6).read() != 0 {
            ipc.events_receive(6).write_value(0);
            trace!("ipc 6");
        }

        if ipc.events_receive(7).read() != 0 {
            ipc.events_receive(7).write_value(0);
            trace!("ipc 7: trace");

            let msg = unsafe { addr_of!((*self.cb).trace.rx_state).read_volatile() };
            if msg != 0 {
                trace!("trace msg {}", msg);
                match msg {
                    0 => unreachable!(),
                    1 => {
                        let ctx = unsafe { addr_of!((*self.cb).trace.rx_ptr).read_volatile() } as *mut TraceContext;
                        debug!("trace init: {:?}", ctx);
                        self.trace_check.check(ctx);
                        let chans = unsafe { addr_of!((*ctx).chans).read_volatile() };
                        for chan_ptr in chans {
                            let chan = self.trace_check.check_read(chan_ptr);
                            self.trace_check.check(chan.start);
                            self.trace_check.check(chan.end);
                            assert!(chan.start < chan.end);
                            self.trace_chans
                                .push(TraceChannelInfo {
                                    ptr: chan_ptr,
                                    start: chan.start,
                                    end: chan.end,
                                })
                                .map_err(|_| ())
                                .unwrap()
                        }
                    }
                    2 => {
                        for chan_info in &self.trace_chans {
                            let read_ptr = unsafe { addr_of!((*chan_info.ptr).read_ptr).read_volatile() };
                            let write_ptr = unsafe { addr_of!((*chan_info.ptr).write_ptr).read_volatile() };
                            assert!(read_ptr >= chan_info.start && read_ptr <= chan_info.end);
                            assert!(write_ptr >= chan_info.start && write_ptr <= chan_info.end);
                            if read_ptr != write_ptr {
                                let id = unsafe { addr_of!((*chan_info.ptr).id).read_volatile() };
                                fence(Ordering::SeqCst); // synchronize volatile accesses with the slice access.
                                if read_ptr < write_ptr {
                                    Self::handle_trace(trace_writer, id, unsafe {
                                        slice::from_raw_parts(read_ptr, write_ptr.offset_from(read_ptr) as _)
                                    });
                                } else {
                                    Self::handle_trace(trace_writer, id, unsafe {
                                        slice::from_raw_parts(read_ptr, chan_info.end.offset_from(read_ptr) as _)
                                    });
                                    Self::handle_trace(trace_writer, id, unsafe {
                                        slice::from_raw_parts(
                                            chan_info.start,
                                            write_ptr.offset_from(chan_info.start) as _,
                                        )
                                    });
                                }
                                fence(Ordering::SeqCst); // synchronize volatile accesses with the slice access.
                                unsafe { addr_of_mut!((*chan_info.ptr).read_ptr).write_volatile(write_ptr) };
                            }
                        }
                    }
                    _ => warn!("unknown trace msg {}", msg),
                }
                unsafe { addr_of_mut!((*self.cb).trace.rx_state).write_volatile(0) };
            }
        }

        ipc.intenset().write(|w| {
            w.set_receive0(true);
            w.set_receive2(true);
            w.set_receive4(true);
            w.set_receive6(true);
            w.set_receive7(true);
        });
    }

    fn handle_trace(writer: &mut Option<TraceWriter<'_>>, id: u8, data: &[u8]) {
        if let Some(writer) = writer {
            trace!("trace: {} {}", id, data.len());
            let mut header = [0u8; 5];
            header[0] = 0xEF;
            header[1] = 0xBE;
            header[2..4].copy_from_slice(&(data.len() as u16).to_le_bytes());
            header[4] = id;
            writer.try_write(&header).ok();
            writer.try_write(data).ok();
        }
    }

    fn process(&mut self, list: *mut List, is_control: bool, ch: &mut ch::Runner<MTU>) -> bool {
        let mut did_work = false;
        let max = if is_control {
            self.rx_control_len
        } else {
            self.rx_data_len
        };
        for i in 0..max {
            let item_ptr = unsafe { addr_of_mut!((*list).items[i]) };
            let preamble = unsafe { addr_of!((*item_ptr).state).read_volatile() };
            if preamble & 0xFF == 0x01 && preamble >> 16 == self.rx_seq_no as u32 {
                let msg_ptr = unsafe { addr_of!((*item_ptr).message).read_volatile() };
                let msg = self.rx_check.check_read(msg_ptr);

                debug!("rx seq {} msg: {:?}", preamble >> 16, msg);

                if is_control {
                    self.handle_control(&msg);
                } else {
                    self.handle_data(&msg, ch);
                }

                unsafe { addr_of_mut!((*item_ptr).state).write_volatile(0x03) };
                self.rx_seq_no = self.rx_seq_no.wrapping_add(1);

                did_work = true;
            }
        }
        did_work
    }

    fn find_free_message(&mut self, ch: usize) -> Option<usize> {
        for i in 0..LIST_LEN {
            let preamble = unsafe { addr_of!((*self.cb).lists[ch].items[i].state).read_volatile() };
            if matches!(preamble & 0xFF, 0 | 3) {
                trace!("using tx msg idx {}", i);
                return Some(i);
            }
        }
        return None;
    }

    fn find_free_tx_buf(&mut self) -> Option<usize> {
        for i in 0..TX_BUF_COUNT {
            if !self.tx_buf_used[i] {
                trace!("using tx buf idx {}", i);
                return Some(i);
            }
        }
        return None;
    }

    fn send_message(&mut self, msg: &mut Message, data: &[u8]) -> Result<(), NoFreeBufs> {
        if data.is_empty() {
            msg.data = ptr::null_mut();
            msg.data_len = 0;
            self.send_message_raw(msg)
        } else {
            assert!(data.len() <= TX_BUF_SIZE);
            let buf_idx = self.find_free_tx_buf().ok_or(NoFreeBufs)?;
            let buf = unsafe { addr_of_mut!((*self.cb).tx_bufs[buf_idx]) } as *mut u8;
            unsafe { copy_nonoverlapping(data.as_ptr(), buf, data.len()) }
            msg.data = buf;
            msg.data_len = data.len();
            self.tx_buf_used[buf_idx] = true;

            fence(Ordering::SeqCst); // synchronize copy_nonoverlapping (non-volatile) with volatile writes below.
            if let Err(e) = self.send_message_raw(msg) {
                msg.data = ptr::null_mut();
                msg.data_len = 0;
                self.tx_buf_used[buf_idx] = false;
                self.tx_waker.wake();
                Err(e)
            } else {
                Ok(())
            }
        }
    }

    fn send_message_raw(&mut self, msg: &Message) -> Result<(), NoFreeBufs> {
        let (ch, ipc_ch) = match msg.channel {
            1 => (0, 1), // control
            2 => (1, 3), // data
            _ => unreachable!(),
        };

        // allocate a msg.
        let idx = self.find_free_message(ch).ok_or(NoFreeBufs)?;

        debug!("tx seq {} msg: {:?}", self.tx_seq_no, msg);

        let msg_slot = unsafe { addr_of_mut!((*self.cb).msgs[ch][idx]) };
        unsafe { msg_slot.write_volatile(*msg) }
        let list_item = unsafe { addr_of_mut!((*self.cb).lists[ch].items[idx]) };
        unsafe { addr_of_mut!((*list_item).message).write_volatile(msg_slot) }
        unsafe { addr_of_mut!((*list_item).state).write_volatile((self.tx_seq_no as u32) << 16 | 0x01) }
        self.tx_seq_no = self.tx_seq_no.wrapping_add(1);

        let ipc = pac::IPC_NS;
        ipc.tasks_send(ipc_ch).write_value(1);
        Ok(())
    }

    fn handle_control(&mut self, msg: &Message) {
        match msg.id >> 16 {
            1 => debug!("control msg: modem ready"),
            2 => self.handle_control_free(msg.data),
            _ => warn!("unknown control message id {:08x}", msg.id),
        }
    }

    fn handle_control_free(&mut self, ptr: *mut u8) {
        let base = unsafe { addr_of!((*self.cb).tx_bufs) } as usize;
        let ptr = ptr as usize;

        if ptr < base {
            warn!("control free bad pointer {:08x}", ptr);
            return;
        }

        let diff = ptr - base;
        let idx = diff / TX_BUF_SIZE;

        if idx >= TX_BUF_COUNT || idx * TX_BUF_SIZE != diff {
            warn!("control free bad pointer {:08x}", ptr);
            return;
        }

        trace!("control free pointer {:08x} idx {}", ptr, idx);
        if !self.tx_buf_used[idx] {
            warn!(
                "control free pointer {:08x} idx {}: buffer was already free??",
                ptr, idx
            );
        }
        self.tx_buf_used[idx] = false;
        self.tx_waker.wake();
    }

    fn handle_data(&mut self, msg: &Message, ch: &mut ch::Runner<MTU>) {
        if !msg.data.is_null() {
            self.rx_check.check_length(msg.data, msg.data_len);
        }

        let freed = match msg.id & 0xFFFF {
            // AT
            3 => {
                match msg.id >> 16 {
                    // AT request ack
                    2 => false,
                    // AT response
                    3 => self.handle_resp(msg),
                    // AT notification
                    4 => false,
                    x => {
                        warn!("received unknown AT kind {}", x);
                        false
                    }
                }
            }
            // IP
            4 => {
                match msg.id >> 28 {
                    // IP response
                    8 => self.handle_resp(msg),
                    // IP notification
                    9 => match (msg.id >> 16) & 0xFFF {
                        // IP receive notification
                        1 => {
                            if let Some(buf) = ch.try_rx_buf() {
                                let mut len = msg.data_len;
                                if len > buf.len() {
                                    warn!("truncating rx'd packet from {} to {} bytes", len, buf.len());
                                    len = buf.len();
                                }
                                fence(Ordering::SeqCst); // synchronize volatile accesses with the nonvolatile copy_nonoverlapping.
                                unsafe { ptr::copy_nonoverlapping(msg.data, buf.as_mut_ptr(), len) }
                                fence(Ordering::SeqCst); // synchronize volatile accesses with the nonvolatile copy_nonoverlapping.
                                ch.rx_done(len);
                            }
                            false
                        }
                        _ => false,
                    },
                    x => {
                        warn!("received unknown IP kind {}", x);
                        false
                    }
                }
            }
            x => {
                warn!("received unknown kind {}", x);
                false
            }
        };

        if !freed {
            self.send_free(msg);
        }
    }

    fn handle_resp(&mut self, msg: &Message) -> bool {
        let req_serial = u32::from_le_bytes(msg.param[0..4].try_into().unwrap());
        if req_serial == 0 {
            return false;
        }

        for optr in &mut self.requests {
            if let Some(r) = optr {
                if r.req_serial == req_serial {
                    let r = optr.take().unwrap();
                    unsafe { r.resp_msg.write(*msg) }
                    r.waker.wake();
                    *optr = None;
                    return true;
                }
            }
        }

        warn!(
            "resp with id {} serial {} doesn't match any pending req",
            msg.id, req_serial
        );
        false
    }

    fn send_free(&mut self, msg: &Message) {
        if msg.data.is_null() {
            return;
        }

        let mut free_msg: Message = unsafe { mem::zeroed() };
        free_msg.channel = 1; // control
        free_msg.id = 0x20001; // free
        free_msg.data = msg.data;
        free_msg.data_len = msg.data_len;

        unwrap!(self.send_message_raw(&free_msg));
    }
}

struct PointerChecker {
    start: *mut u8,
    end: *mut u8,
}

impl PointerChecker {
    // check the pointer is in bounds in the arena, panic otherwise.
    fn check_length(&self, ptr: *const u8, len: usize) {
        assert!(ptr as usize >= self.start as usize);
        let end_ptr = (ptr as usize).checked_add(len).unwrap();
        assert!(end_ptr <= self.end as usize);
    }

    // check the pointer is in bounds in the arena, panic otherwise.
    fn check<T>(&self, ptr: *const T) {
        assert!(ptr.is_aligned());
        self.check_length(ptr as *const u8, mem::size_of::<T>());
    }

    // check the pointer is in bounds in the arena, panic otherwise.
    fn check_read<T>(&self, ptr: *const T) -> T {
        self.check(ptr);
        unsafe { ptr.read_volatile() }
    }

    // check the pointer is in bounds in the arena, panic otherwise.
    fn check_mut<T>(&self, ptr: *mut T) {
        self.check(ptr as *const T)
    }
}

/// Control handle for the driver.
///
/// You can use this object to control the modem at runtime, such as running AT commands.
pub struct Control<'a> {
    state: &'a RefCell<StateInner>,
}

impl<'a> Control<'a> {
    /// Wait for modem IPC to be initialized.
    pub fn wait_init(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(|cx| {
            let mut state = self.state.borrow_mut();
            if state.init {
                return Poll::Ready(());
            }
            state.init_waker.register(cx.waker());
            Poll::Pending
        })
    }

    async fn request(&self, msg: &mut Message, req_data: &[u8], resp_data: &mut [u8]) -> usize {
        // get waker
        let waker = poll_fn(|cx| Poll::Ready(cx.waker().clone())).await;

        // Send request
        let mut state = self.state.borrow_mut();
        let mut req_serial = state.next_req_serial;
        if msg.id & 0xFFFF == 3 {
            // AT response seems to keep only the lower 8 bits. Others do keep the full 32 bits..??
            req_serial &= 0xFF;
        }

        // increment next_req_serial, skip zero because we use it as an "ignore" value.
        // We have to skip when the *lowest byte* is zero because AT responses.
        state.next_req_serial = state.next_req_serial.wrapping_add(1);
        if state.next_req_serial & 0xFF == 0 {
            state.next_req_serial = state.next_req_serial.wrapping_add(1);
        }

        drop(state); // don't borrow state across awaits.

        msg.param[0..4].copy_from_slice(&req_serial.to_le_bytes());

        poll_fn(|cx| {
            let mut state = self.state.borrow_mut();
            state.tx_waker.register(cx.waker());
            match state.send_message(msg, req_data) {
                Ok(_) => Poll::Ready(()),
                Err(NoFreeBufs) => Poll::Pending,
            }
        })
        .await;

        // Setup the pending request state.
        let mut state = self.state.borrow_mut();
        let (req_slot_idx, req_slot) = state
            .requests
            .iter_mut()
            .enumerate()
            .find(|(_, x)| x.is_none())
            .unwrap();
        msg.id = 0; // zero out id, so when it becomes nonzero we know the req is done.
        let msg_ptr: *mut Message = msg;
        *req_slot = Some(PendingRequest {
            req_serial,
            resp_msg: msg_ptr,
            waker,
        });

        drop(state); // don't borrow state across awaits.

        // On cancel, unregister the request slot.
        let _drop = OnDrop::new(|| {
            // Remove request slot.
            let mut state = self.state.borrow_mut();
            let slot = &mut state.requests[req_slot_idx];
            if let Some(s) = slot {
                if s.req_serial == req_serial {
                    *slot = None;
                }
            }

            // If cancelation raced with actually receiving the response,
            // we own the data, so we have to free it.
            let msg = unsafe { &mut *msg_ptr };
            if msg.id != 0 {
                state.send_free(msg);
            }
        });
        // Wait for response.
        poll_fn(|_| {
            // we have to use the raw pointer and not the original reference `msg`
            // because that'd invalidate the raw ptr that's still stored in `req_slot`.
            if unsafe { (*msg_ptr).id } != 0 {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
        _drop.defuse();

        if msg.data.is_null() {
            // no response data.
            return 0;
        }

        // Copy response data out, if any.
        // Pointer was validated in StateInner::handle_data().
        let mut len = msg.data_len;
        if len > resp_data.len() {
            warn!("truncating response data from {} to {}", len, resp_data.len());
            len = resp_data.len();
        }
        fence(Ordering::SeqCst); // synchronize volatile accesses with the nonvolatile copy_nonoverlapping.
        unsafe { ptr::copy_nonoverlapping(msg.data, resp_data.as_mut_ptr(), len) }
        fence(Ordering::SeqCst); // synchronize volatile accesses with the nonvolatile copy_nonoverlapping.
        self.state.borrow_mut().send_free(msg);
        len
    }

    /// Run an AT command.
    ///
    /// The response is written in `resp` and its length returned.
    pub async fn at_command(&self, req: &[u8], resp: &mut [u8]) -> usize {
        let mut msg: Message = unsafe { mem::zeroed() };
        msg.channel = 2; // data
        msg.id = 0x0001_0003; // AT command
        msg.param_len = 4;

        self.request(&mut msg, req, resp).await
    }

    /// Open the raw socket used for sending/receiving IP packets.
    ///
    /// This must be done after `AT+CFUN=1` (?)
    async fn open_raw_socket(&self) -> u32 {
        let mut msg: Message = unsafe { mem::zeroed() };
        msg.channel = 2; // data
        msg.id = 0x7001_0004; // open socket
        msg.param_len = 20;

        let param = [
            0xFF, 0xFF, 0xFF, 0xFF, // req_serial
            0xFF, 0xFF, 0xFF, 0xFF, // ???
            0x05, 0x00, 0x00, 0x00, // family
            0x03, 0x00, 0x00, 0x00, // type
            0x00, 0x00, 0x00, 0x00, // protocol
        ];
        msg.param[..param.len()].copy_from_slice(&param);

        self.request(&mut msg, &[], &mut []).await;

        assert_eq!(msg.id, 0x80010004);
        assert!(msg.param_len >= 12);
        let status = u32::from_le_bytes(msg.param[8..12].try_into().unwrap());
        assert_eq!(status, 0);
        assert_eq!(msg.param_len, 16);
        let fd = u32::from_le_bytes(msg.param[12..16].try_into().unwrap());
        self.state.borrow_mut().net_fd.replace(fd);

        trace!("got FD: {}", fd);
        fd
    }

    async fn close_raw_socket(&self, fd: u32) {
        let mut msg: Message = unsafe { mem::zeroed() };
        msg.channel = 2; // data
        msg.id = 0x7009_0004; // close socket
        msg.param_len = 8;
        msg.param[4..8].copy_from_slice(&fd.to_le_bytes());

        self.request(&mut msg, &[], &mut []).await;

        assert_eq!(msg.id, 0x80090004);
        assert!(msg.param_len >= 12);
        let status = u32::from_le_bytes(msg.param[8..12].try_into().unwrap());
        assert_eq!(status, 0);
    }
}

/// Background runner for the driver.
pub struct Runner<'a> {
    ch: ch::Runner<'a, MTU>,
    state: &'a RefCell<StateInner>,
    trace_writer: Option<TraceWriter<'a>>,
}

impl<'a> Runner<'a> {
    /// Run the driver operation in the background.
    ///
    /// You must run this in a background task, concurrently with all network operations.
    pub async fn run(mut self) -> ! {
        poll_fn(|cx| {
            WAKER.register(cx.waker());

            let mut state = self.state.borrow_mut();
            state.poll(&mut self.trace_writer, &mut self.ch);

            if let Poll::Ready(buf) = self.ch.poll_tx_buf(cx) {
                if let Some(fd) = state.net_fd {
                    let mut msg: Message = unsafe { mem::zeroed() };
                    msg.channel = 2; // data
                    msg.id = 0x7006_0004; // IP send
                    msg.param_len = 12;
                    msg.param[4..8].copy_from_slice(&fd.to_le_bytes());
                    if let Err(e) = state.send_message(&mut msg, buf) {
                        warn!("tx failed: {:?}", e);
                    }
                    self.ch.tx_done();
                }
            }

            Poll::Pending
        })
        .await
    }
}

const LIST_LEN: usize = 32;

#[repr(C)]
struct ControlBlock {
    version: u32,
    rx_base: *mut u8,
    rx_size: usize,
    control_list_ptr: *mut List,
    data_list_ptr: *mut List,
    modem_info_ptr: *mut ModemInfo,
    trace_ptr: *mut Trace,
    unk: u32,

    modem_info: ModemInfo,
    trace: Trace,

    // 0 = control, 1 = data
    lists: [List; 2],
    msgs: [[Message; LIST_LEN]; 2],

    tx_bufs: [[u8; TX_BUF_SIZE]; TX_BUF_COUNT],
}

#[repr(C)]
struct ModemInfo {
    version: u32,
    control_list_ptr: *mut List,
    data_list_ptr: *mut List,
    padding: [u32; 5],
}

#[repr(C)]
struct Trace {
    size: usize,
    base: *mut u8,
    tx_state: u32,
    tx_ptr: *mut u8,
    rx_state: u32,
    rx_ptr: *mut u8,
    unk1: u32,
    unk2: u32,
}

const TRACE_CHANNEL_COUNT: usize = 3;

#[repr(C)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct TraceContext {
    unk1: u32,
    unk2: u32,
    len: u32,
    chans: [*mut TraceChannel; TRACE_CHANNEL_COUNT],
}

#[repr(C)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct TraceChannel {
    id: u8,
    unk1: u8,
    unk2: u8,
    unk3: u8,
    write_ptr: *mut u8,
    read_ptr: *mut u8,
    start: *mut u8,
    end: *mut u8,
}

#[repr(C)]
struct List {
    len: usize,
    items: [ListItem; LIST_LEN],
}

#[repr(C)]
struct ListItem {
    /// top 16 bits: seqno
    /// bottom 8 bits:
    ///     0x01: sent
    ///     0x02: held
    ///     0x03: freed
    state: u32,
    message: *mut Message,
}

#[repr(C)]
#[derive(defmt::Format, Clone, Copy)]
struct Message {
    id: u32,

    /// 1 = control, 2 = data
    channel: u8,
    unk1: u8,
    unk2: u8,
    unk3: u8,

    data: *mut u8,
    data_len: usize,
    param_len: usize,
    param: [u8; 44],
}

struct OnDrop<F: FnOnce()> {
    f: MaybeUninit<F>,
}

impl<F: FnOnce()> OnDrop<F> {
    pub fn new(f: F) -> Self {
        Self { f: MaybeUninit::new(f) }
    }

    pub fn defuse(self) {
        mem::forget(self)
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        unsafe { self.f.as_ptr().read()() }
    }
}
