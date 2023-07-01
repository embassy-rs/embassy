use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::Poll;

use embassy_hal_common::PeripheralRef;
use embassy_sync::waitqueue::AtomicWaker;
use embedded_hal_02::adc::{Channel, OneShot};
use pac::adc::regs::{Cs, Fcs};

use crate::dma::{Channel as DmaChannel, ContinuousTransfer, Dreq, Read, Transfer, Word};
use crate::gpio::Pin;
use crate::interrupt::typelevel::Binding;
use crate::interrupt::InterruptExt;
use crate::pac::dma::vals::TreqSel;
use crate::peripherals::ADC;
use crate::{interrupt, pac, peripherals, Peripheral};

static WAKER: AtomicWaker = AtomicWaker::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    // No errors for now
}

#[non_exhaustive]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}
pub struct Adc<'d> {
    phantom: PhantomData<&'d ADC>,
}

pub trait Input {
    fn round_robin_byte(&self) -> u8;
    fn ainsel_channel(&self) -> u8;
    fn measure_temperature(&self) -> bool;

    // disable pull-up and enable pull-down resistors
    fn prepare_resistors(&mut self) -> ();
}

// convention: only Pin1 or Temp can be starting channels (ainsel)
#[derive(Copy, Clone)]
enum RoundRobinConfig {
    // T: temperature is measured
    // P: pin1 is ainsel (else temperature)
    // R: ainsel is part of round robin
    T,
    TR,
    PT,
    PTR,
    P,
    PR,
}

impl RoundRobinConfig {
    // combine byte enabling temperature round robin
    // and byte enabling pin1 round robin according to config
    fn temp_pin1_byte(self, p1: u8) -> u8 {
        use RoundRobinConfig::*;
        let t = 1u8 << 4; // temperature channel has id 4
        match self {
            T | PR => p1,
            TR | PTR => t | p1,
            PT => t,
            P => 0,
        }
    }

    fn ainsel_channel(self, p1_channel: u8) -> u8 {
        use RoundRobinConfig::*;
        match self {
            T | TR => 4,
            _ => p1_channel,
        }
    }

    fn is_t(self) -> bool {
        use RoundRobinConfig::*;
        match self {
            P | PR => false,
            T | TR | PT | PTR => true,
        }
    }
    fn set_t(&mut self) {
        use RoundRobinConfig::*;
        match self {
            T | TR | PT | PTR => (),
            P => *self = PT,
            PR => *self = PTR,
        }
    }
}

fn prepare_resistors<'d, PIN>(pin: &mut PIN)
where
    PIN: Channel<Adc<'d>, ID = u8> + Pin,
{
    pin.pad_ctrl().modify(|w| {
        w.set_ie(true);
        let (pu, pd) = (false, true); // TODO there is another pull request related to this change, also check datasheet chapter 4.9
        w.set_pue(pu);
        w.set_pde(pd);
    });
}

pub fn round_robin_order(round_robin_byte: u8, ainsel_channel: u8) -> (u8, [Option<u8>; 5]) {
    let mut result = [None; 5];
    let length = u8::count_ones(round_robin_byte) as u8;
    let mut channel = ainsel_channel;
    for i in 0..length {
        let b = round_robin_byte >> (channel + 1);
        channel = match b {
            0 => round_robin_byte.trailing_zeros() as u8,
            _ => b.trailing_zeros() as u8 + channel + 1,
        };
        result[i as usize] = Some(channel);
    }
    (length, result)
}

pub struct Input0 {
    // has to be temperature
    config: RoundRobinConfig,
}
pub struct Input1<'d, 'a, Pin1>
where
    Pin1: Channel<Adc<'d>, ID = u8> + Pin,
{
    pin1: &'a mut Pin1,
    config: RoundRobinConfig,
    phantom: PhantomData<&'d bool>,
}
pub struct Input2<'d, 'a, Pin1, Pin2>
where
    Pin1: Channel<Adc<'d>, ID = u8> + Pin,
    Pin2: Channel<Adc<'d>, ID = u8> + Pin,
{
    pin1: &'a mut Pin1,
    pin2: &'a mut Pin2,
    config: RoundRobinConfig,
    phantom: PhantomData<&'d bool>,
}
pub struct Input3<'d, 'a, Pin1, Pin2, Pin3>
where
    Pin1: Channel<Adc<'d>, ID = u8> + Pin,
    Pin2: Channel<Adc<'d>, ID = u8> + Pin,
    Pin3: Channel<Adc<'d>, ID = u8> + Pin,
{
    pin1: &'a mut Pin1,
    pin2: &'a mut Pin2,
    pin3: &'a mut Pin3,
    config: RoundRobinConfig,
    phantom: PhantomData<&'d bool>,
}
pub struct Input4<'d, 'a, Pin1, Pin2, Pin3, Pin4>
where
    Pin1: Channel<Adc<'d>, ID = u8> + Pin,
    Pin2: Channel<Adc<'d>, ID = u8> + Pin,
    Pin3: Channel<Adc<'d>, ID = u8> + Pin,
    Pin4: Channel<Adc<'d>, ID = u8> + Pin,
{
    pin1: &'a mut Pin1,
    pin2: &'a mut Pin2,
    pin3: &'a mut Pin3,
    pin4: &'a mut Pin4,
    config: RoundRobinConfig,
    phantom: PhantomData<&'d bool>,
}

impl Input0 {
    pub fn add<'d, 'a, PIN>(self, pin: &'a mut PIN) -> Input1<'d, 'a, PIN>
    where
        PIN: Channel<Adc<'d>, ID = u8> + Pin,
    {
        Input1 {
            pin1: pin,
            config: self.config,
            phantom: PhantomData {},
        }
    }
}

impl<'d, 'a, Pin1> Input1<'d, 'a, Pin1>
where
    Pin1: Channel<Adc<'d>, ID = u8> + Pin,
{
    pub fn add<PIN>(self, pin: &'a mut PIN) -> Input2<'d, 'a, Pin1, PIN>
    where
        PIN: Channel<Adc<'d>, ID = u8> + Pin,
    {
        let Input1 { pin1, config, .. } = self;
        Input2 {
            pin1,
            pin2: pin,
            config,
            phantom: PhantomData {},
        }
    }
    pub fn add_temperature(mut self) -> Input1<'d, 'a, Pin1> {
        self.config.set_t();
        self
    }
}

impl<'d, 'a, Pin1, Pin2> Input2<'d, 'a, Pin1, Pin2>
where
    Pin1: Channel<Adc<'d>, ID = u8> + Pin,
    Pin2: Channel<Adc<'d>, ID = u8> + Pin,
{
    pub fn add<PIN>(self, pin: &'a mut PIN) -> Input3<'d, 'a, Pin1, Pin2, PIN>
    where
        PIN: Channel<Adc<'d>, ID = u8> + Pin,
    {
        let Input2 { pin1, pin2, config, .. } = self;
        Input3 {
            pin1,
            pin2,
            pin3: pin,
            config,
            phantom: PhantomData {},
        }
    }
    pub fn add_temperature(mut self) -> Input2<'d, 'a, Pin1, Pin2> {
        self.config.set_t();
        self
    }
}

impl<'d, 'a, Pin1, Pin2, Pin3> Input3<'d, 'a, Pin1, Pin2, Pin3>
where
    Pin1: Channel<Adc<'d>, ID = u8> + Pin,
    Pin2: Channel<Adc<'d>, ID = u8> + Pin,
    Pin3: Channel<Adc<'d>, ID = u8> + Pin,
{
    pub fn add<PIN>(self, pin: &'a mut PIN) -> Input4<'d, 'a, Pin1, Pin2, Pin3, PIN>
    where
        PIN: Channel<Adc<'d>, ID = u8> + Pin,
    {
        let Input3 {
            pin1,
            pin2,
            pin3,
            config,
            ..
        } = self;
        Input4 {
            pin1,
            pin2,
            pin3,
            pin4: pin,
            config,
            phantom: PhantomData {},
        }
    }
    pub fn add_temperature(mut self) -> Input3<'d, 'a, Pin1, Pin2, Pin3> {
        self.config.set_t();
        self
    }
}

impl<'d, 'a, Pin1, Pin2, Pin3, Pin4> Input4<'d, 'a, Pin1, Pin2, Pin3, Pin4>
where
    Pin1: Channel<Adc<'d>, ID = u8> + Pin,
    Pin2: Channel<Adc<'d>, ID = u8> + Pin,
    Pin3: Channel<Adc<'d>, ID = u8> + Pin,
    Pin4: Channel<Adc<'d>, ID = u8> + Pin,
{
    pub fn add_temperature(mut self) -> Input4<'d, 'a, Pin1, Pin2, Pin3, Pin4> {
        self.config.set_t();
        self
    }
}

impl Input for Input0 {
    fn round_robin_byte(&self) -> u8 {
        // it should not matter whether we return b0001_0000 or b0000_0000
        self.config.temp_pin1_byte(0)
    }

    fn ainsel_channel(&self) -> u8 {
        4 // measure temperature
    }

    fn measure_temperature(&self) -> bool {
        true
    }

    fn prepare_resistors(&mut self) -> () {}
}

impl<'d, 'a, Pin1> Input for Input1<'d, 'a, Pin1>
where
    Pin1: Channel<Adc<'d>, ID = u8> + Pin,
{
    fn round_robin_byte(&self) -> u8 {
        let p1 = 1u8 << Pin1::channel();
        self.config.temp_pin1_byte(p1)
    }

    fn ainsel_channel(&self) -> u8 {
        self.config.ainsel_channel(Pin1::channel())
    }

    fn measure_temperature(&self) -> bool {
        self.config.is_t()
    }

    fn prepare_resistors(&mut self) -> () {
        prepare_resistors(self.pin1);
    }
}

impl<'d, 'a, Pin1, Pin2> Input for Input2<'d, 'a, Pin1, Pin2>
where
    Pin1: Channel<Adc<'d>, ID = u8> + Pin,
    Pin2: Channel<Adc<'d>, ID = u8> + Pin,
{
    fn round_robin_byte(&self) -> u8 {
        let p1 = 1u8 << Pin1::channel();
        let p2 = 1u8 << Pin2::channel();
        self.config.temp_pin1_byte(p1) | p2
    }

    fn ainsel_channel(&self) -> u8 {
        self.config.ainsel_channel(Pin1::channel())
    }

    fn measure_temperature(&self) -> bool {
        self.config.is_t()
    }

    fn prepare_resistors(&mut self) -> () {
        prepare_resistors(self.pin1);
        prepare_resistors(self.pin2);
    }
}
impl<'d, 'a, Pin1, Pin2, Pin3> Input for Input3<'d, 'a, Pin1, Pin2, Pin3>
where
    Pin1: Channel<Adc<'d>, ID = u8> + Pin,
    Pin2: Channel<Adc<'d>, ID = u8> + Pin,
    Pin3: Channel<Adc<'d>, ID = u8> + Pin,
{
    fn round_robin_byte(&self) -> u8 {
        let p1 = 1u8 << Pin1::channel();
        let p2 = 1u8 << Pin2::channel();
        let p3 = 1u8 << Pin3::channel();
        self.config.temp_pin1_byte(p1) | p2 | p3
    }

    fn ainsel_channel(&self) -> u8 {
        self.config.ainsel_channel(Pin1::channel())
    }

    fn measure_temperature(&self) -> bool {
        self.config.is_t()
    }
    fn prepare_resistors(&mut self) -> () {
        prepare_resistors(self.pin1);
        prepare_resistors(self.pin2);
        prepare_resistors(self.pin3);
    }
}
impl<'d, 'a, Pin1, Pin2, Pin3, Pin4> Input for Input4<'d, 'a, Pin1, Pin2, Pin3, Pin4>
where
    Pin1: Channel<Adc<'d>, ID = u8> + Pin,
    Pin2: Channel<Adc<'d>, ID = u8> + Pin,
    Pin3: Channel<Adc<'d>, ID = u8> + Pin,
    Pin4: Channel<Adc<'d>, ID = u8> + Pin,
{
    fn round_robin_byte(&self) -> u8 {
        let p1 = 1u8 << Pin1::channel();
        let p2 = 1u8 << Pin2::channel();
        let p3 = 1u8 << Pin3::channel();
        let p4 = 1u8 << Pin4::channel();
        self.config.temp_pin1_byte(p1) | p2 | p3 | p4
    }

    fn ainsel_channel(&self) -> u8 {
        self.config.ainsel_channel(Pin1::channel())
    }

    fn measure_temperature(&self) -> bool {
        self.config.is_t()
    }
    fn prepare_resistors(&mut self) -> () {
        prepare_resistors(self.pin1);
        prepare_resistors(self.pin2);
        prepare_resistors(self.pin3);
        prepare_resistors(self.pin4);
    }
}

// argument use_in_round_robin is only relevent when more pins are added
pub fn input_temperature(use_in_round_robin: bool) -> Input0 {
    Input0 {
        config: match use_in_round_robin {
            true => RoundRobinConfig::TR,
            false => RoundRobinConfig::T,
        },
    }
}

// argument use_in_round_robin is only relevent when more pins or temperature is added
pub fn input_from_pin<'d, 'a, PIN>(pin: &'a mut PIN, use_in_round_robin: bool) -> Input1<'d, 'a, PIN>
where
    PIN: Channel<Adc<'d>, ID = u8> + Pin,
{
    Input1 {
        pin1: pin,
        config: match use_in_round_robin {
            true => RoundRobinConfig::PR,
            false => RoundRobinConfig::P,
        },
        phantom: PhantomData {},
    }
}

#[derive(Copy, Clone)]
pub enum SamplingSpeed {
    Fastest,                       // 500kS/s
    Interval { n: u16, frac: u8 }, // n >= 96
}

impl SamplingSpeed {
    fn set_div(&self, w: &mut rp_pac::adc::regs::Div) {
        match self {
            Self::Fastest => {
                w.set_int(0);
                w.set_frac(0)
            }
            Self::Interval { n, frac } => {
                assert!(*n >= 96);
                w.set_int(*n);
                w.set_frac(*frac);
            }
        }
    }

    pub fn clock_cycles_per_256_samples(&self) -> u32 {
        match self {
            Self::Fastest => 256 * 96,
            Self::Interval { n, frac } => {
                assert!(*n >= 96);
                256 * (1 + *n as u32) + *frac as u32
            }
        }
    }

    // default adc clock speed is 48MHz
    pub fn micros_per_sample(&self, clock_hz: u32) -> f32 {
        self.clock_cycles_per_256_samples() as f32 / 256.0 / clock_hz as f32 * 1_000_000.0
    }
}

pub struct ContinuousAdc<'a, 'b, 'c, 'd, 'r, W: Word, C1: DmaChannel, C2: DmaChannel, In: Input> {
    #[allow(dead_code)]
    adc: Adc<'d>,
    transfer: ContinuousTransfer<'a, 'b, 'c, 'r, W, C1, C2>,
    input: In,
    corrupted: bool,
}

impl<'a, 'b, 'c, 'd, 'r, W: Word, C1: DmaChannel, C2: DmaChannel, In: Input>
    ContinuousAdc<'a, 'b, 'c, 'd, 'r, W, C1, C2, In>
{
    pub fn start_new(
        adc: Adc<'d>,
        ch1: PeripheralRef<'a, C1>,
        ch2: PeripheralRef<'a, C2>,
        mut input: In,
        speed: SamplingSpeed,
        control_input: &'c mut [[u32; 4]; 2],
        buffer: &'b mut [W],
    ) -> ContinuousAdc<'a, 'b, 'c, 'd, 'r, W, C1, C2, In> {
        assert!(W::size() as u8 <= 1); // u16 or u8 (will right-shift) allowed TODO static_assert possible?

        // configure adc
        let r = Adc::regs();
        assert!(r.fcs().read().empty());
        input.prepare_resistors();
        if input.measure_temperature() {
            r.cs().modify(|w| w.set_ts_en(true));
            while !r.cs().read().ready() {} // blocking wait for ready
        }
        // set input channels
        r.cs().modify(|w| {
            w.set_ainsel(input.ainsel_channel());
            w.set_rrobin(input.round_robin_byte());
        });
        r.fcs().modify(|w| {
            w.set_en(true);
            w.set_dreq_en(true);
            w.set_thresh(1);
            w.set_err(false);
            w.set_shift(W::size() as u8 == 0);
        });
        r.div().modify(|w| {
            speed.set_div(w);
        });

        // create and trigger transfer
        let transfer = ContinuousTransfer::start_new(
            ch1,
            ch2,
            control_input,
            buffer,
            TreqSel(Dreq::ADC as u8),
            // SAFETY: adc is owned and cannot be used for anything else, fifo must only accessed by dma channel
            Read::Constant(unsafe { &*(r.fifo().as_ptr() as *const W) }),
        );

        // start adc
        compiler_fence(Ordering::SeqCst);
        r.cs().modify(|w| {
            w.set_start_many(true);
        });

        ContinuousAdc {
            adc,
            transfer,
            input,
            corrupted: false,
        }
    }

    pub async fn next<'new_buf>(
        self,
        buffer: &'new_buf mut [W],
    ) -> (ContinuousAdc<'a, 'new_buf, 'c, 'd, 'r, W, C1, C2, In>, bool) {
        let ContinuousAdc {
            adc,
            transfer,
            input,
            corrupted,
        } = self;
        let (transfer, in_time) = transfer.next(buffer).await;
        (
            ContinuousAdc {
                adc,
                transfer,
                input,
                corrupted: corrupted | !in_time,
            },
            in_time,
        )
    }

    pub async fn stop(self) -> Adc<'d> {
        self.transfer.stop().await;

        // stop adc
        let r = Adc::regs();
        r.cs().modify(|w| {
            w.set_start_many(false);
        });
        if self.input.measure_temperature() {
            r.cs().modify(|w| w.set_ts_en(false));
        }
        Adc::fifo_drain().await;

        if self.corrupted {
            // TODO this is a fix to a problem where round robin order is shifted and the first few samples of any following start_many measurements seem to have random order
            // TODO I was not able to find the real cause, but it would only appear with a certain chance if the next buffer was not provided in time
            // completely reset adc
            let reset = Adc::reset();
            crate::reset::reset(reset);
            crate::reset::unreset_wait(reset);
            let r = Adc::regs();
            // Enable ADC
            r.cs().write(|w| w.set_en(true));
            // Wait for ADC ready
            while !r.cs().read().ready() {}
        }

        // you only get your adc back if you stop the ContinuousAdc like intended
        // (i.e. don't drop it while it is still running)
        self.adc
    }
}

impl<'d> Adc<'d> {
    #[inline]
    fn regs() -> pac::adc::Adc {
        pac::ADC
    }

    #[inline]
    fn reset() -> pac::resets::regs::Peripherals {
        let mut ret = pac::resets::regs::Peripherals::default();
        ret.set_adc(true);
        ret
    }

    pub fn new(
        _inner: impl Peripheral<P = ADC> + 'd,
        _irq: impl Binding<interrupt::typelevel::ADC_IRQ_FIFO, InterruptHandler>,
        _config: Config,
    ) -> Self {
        let reset = Self::reset();
        crate::reset::reset(reset);
        crate::reset::unreset_wait(reset);
        let r = Self::regs();
        // Enable ADC
        r.cs().write(|w| w.set_en(true));
        // Wait for ADC ready
        while !r.cs().read().ready() {}

        // Setup IRQ
        interrupt::ADC_IRQ_FIFO.unpend();
        unsafe { interrupt::ADC_IRQ_FIFO.enable() };

        Self { phantom: PhantomData }
    }

    async fn wait_for_ready() {
        let r = Self::regs();
        r.inte().write(|w| w.set_fifo(true));
        compiler_fence(Ordering::SeqCst);
        poll_fn(|cx| {
            WAKER.register(cx.waker());
            if r.cs().read().ready() {
                return Poll::Ready(());
            }
            Poll::Pending
        })
        .await;
    }

    async fn fifo_drain() {
        let r = Self::regs();
        Self::wait_for_ready().await;
        // drain remaining samples from the FIFO
        while !r.fcs().read().empty() {
            let _ = r.fifo().read().val();
        }
        // reset fifo so samples don't fill the fifo when we don't want it
        r.fcs().write_value(Fcs(0));
    }

    pub async fn dma_read<'a, C: DmaChannel, W: Word, I: Input>(
        &mut self,
        input: &mut I,
        dma_ch: PeripheralRef<'a, C>,
        data: &'a mut [W],
        speed: SamplingSpeed,
    ) -> () {
        let r = Self::regs();
        assert!(r.fcs().read().empty());

        input.prepare_resistors();

        if input.measure_temperature() {
            // enable temperature
            r.cs().modify(|w| w.set_ts_en(true));
            if !r.cs().read().ready() {
                Self::wait_for_ready().await;
            }
        }

        // set input channels
        r.cs().modify(|w| {
            w.set_ainsel(input.ainsel_channel());
            w.set_rrobin(input.round_robin_byte());
        });

        r.fcs().modify(|w| {
            w.set_en(true);
            w.set_dreq_en(true);
            w.set_thresh(1);
            w.set_err(false);
            w.set_shift(W::size() as u8 == 0);
        });
        r.div().modify(|w| {
            speed.set_div(w);
        });

        let p = dma_ch.regs();
        p.write_addr().write_value(data.as_ptr() as u32);
        p.read_addr().write_value(r.fifo().as_ptr() as u32);
        p.trans_count().write_value(data.len() as u32);
        compiler_fence(Ordering::SeqCst);
        p.ctrl_trig().write(|w| {
            w.set_treq_sel(TreqSel(Dreq::ADC as u8));
            w.set_data_size(W::size());
            w.set_incr_read(false);
            w.set_incr_write(true);
            w.set_chain_to(dma_ch.number());
            w.set_en(true);
        }); // trigger dma start
        compiler_fence(Ordering::SeqCst);

        // start adc
        r.cs().modify(|w| {
            w.set_start_many(true);
        });

        // wait for finish
        Transfer::new(dma_ch).await;

        // stop adc
        r.cs().modify(|w| {
            w.set_start_many(false);
        });

        // disable temperature sensing
        if input.measure_temperature() {
            r.cs().modify(|w| w.set_ts_en(false));
        }

        Self::fifo_drain().await;
    }

    pub async fn read<PIN: Channel<Adc<'d>, ID = u8> + Pin>(&mut self, pin: &mut PIN) -> u16 {
        let r = Self::regs();
        prepare_resistors(pin);
        r.cs().modify(|w| {
            w.set_ainsel(PIN::channel());
            w.set_start_once(true)
        });
        Self::wait_for_ready().await;
        r.result().read().result().into()
    }

    pub async fn read_temperature(&mut self) -> u16 {
        let r = Self::regs();
        r.cs().modify(|w| w.set_ts_en(true));
        if !r.cs().read().ready() {
            Self::wait_for_ready().await;
        }
        r.cs().modify(|w| {
            w.set_ainsel(4);
            w.set_start_once(true)
        });
        Self::wait_for_ready().await;
        r.result().read().result().into()
    }

    pub fn blocking_read<PIN: Channel<Adc<'d>, ID = u8> + Pin>(&mut self, pin: &mut PIN) -> u16 {
        let r = Self::regs();
        prepare_resistors(pin);
        r.cs().modify(|w| {
            w.set_ainsel(PIN::channel());
            w.set_start_once(true)
        });
        while !r.cs().read().ready() {}
        r.result().read().result().into()
    }

    pub fn blocking_read_temperature(&mut self) -> u16 {
        let r = Self::regs();
        r.cs().modify(|w| w.set_ts_en(true));
        while !r.cs().read().ready() {}
        r.cs().modify(|w| {
            w.set_ainsel(4);
            w.set_start_once(true)
        });
        while !r.cs().read().ready() {}
        r.result().read().result().into()
    }
}

impl<'d> Drop for Adc<'d> {
    fn drop(&mut self) {
        let r = Self::regs();
        r.cs().write_value(Cs(0));
        while !r.cs().read().ready() {}
        while !r.fcs().read().empty() {
            let _ = r.fifo().read().val();
        }
        r.fcs().write_value(Fcs(0));
    }
}

macro_rules! impl_pin {
    ($pin:ident, $channel:expr) => {
        impl Channel<Adc<'static>> for peripherals::$pin {
            type ID = u8;
            fn channel() -> u8 {
                $channel
            }
        }
    };
}

pub struct InterruptHandler {
    _empty: (),
}

impl interrupt::typelevel::Handler<interrupt::typelevel::ADC_IRQ_FIFO> for InterruptHandler {
    unsafe fn on_interrupt() {
        let r = Adc::regs();
        r.inte().write(|w| w.set_fifo(false));
        WAKER.wake();
    }
}

impl_pin!(PIN_26, 0);
impl_pin!(PIN_27, 1);
impl_pin!(PIN_28, 2);
impl_pin!(PIN_29, 3);

impl<WORD, PIN> OneShot<Adc<'static>, WORD, PIN> for Adc<'static>
where
    WORD: From<u16>,
    PIN: Channel<Adc<'static>, ID = u8> + Pin,
{
    type Error = ();
    fn read(&mut self, pin: &mut PIN) -> nb::Result<WORD, Self::Error> {
        Ok(self.blocking_read(pin).into())
    }
}
