//! True Random Number Generator (TRNG)

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_futures::block_on;
use embassy_sync::waitqueue::AtomicWaker;

use crate::clocks::{SysconPeripheral, enable_and_reset};
use crate::interrupt::typelevel::Interrupt;
use crate::pac::trng::vals::{
    IntCtrlEntVal, IntCtrlFrqCtFail, IntCtrlHwErr, IntMaskEntVal, IntMaskFrqCtFail, IntMaskHwErr, IntStatusEntVal,
    IntStatusFrqCtFail, IntStatusHwErr,
};
use crate::{Peri, PeripheralType, interrupt, peripherals};

static RNG_WAKER: AtomicWaker = AtomicWaker::new();

/// RNG ;error
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Seed error.
    SeedError,

    /// HW Error.
    HwError,

    /// Frequency Count Fail
    FreqCountFail,
}

/// RNG interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::info().regs;
        let int_status = regs.int_status().read();

        if int_status.ent_val() == IntStatusEntVal::ENT_VAL_1
            || int_status.hw_err() == IntStatusHwErr::HW_ERR_1
            || int_status.frq_ct_fail() == IntStatusFrqCtFail::FRQ_CT_FAIL_1
        {
            regs.int_ctrl().modify(|w| {
                w.set_ent_val(IntCtrlEntVal::ENT_VAL_0);
                w.set_hw_err(IntCtrlHwErr::HW_ERR_0);
                w.set_frq_ct_fail(IntCtrlFrqCtFail::FRQ_CT_FAIL_0);
            });
            RNG_WAKER.wake();
        }
    }
}

/// RNG driver.
pub struct Rng<'d> {
    info: Info,
    _lifetime: PhantomData<&'d ()>,
}

impl<'d> Rng<'d> {
    /// Create a new RNG driver.
    pub fn new<T: Instance>(
        _inner: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        enable_and_reset::<T>();

        let mut random = Self {
            info: T::info(),
            _lifetime: PhantomData,
        };
        random.init();

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        random
    }

    /// Reset the RNG.
    pub fn reset(&mut self) {
        self.info.regs.mctl().write(|w| {
            w.set_rst_def(true);
            w.set_prgm(true);
        });
    }

    /// Fill the given slice with random values.
    pub async fn async_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        // We have a total of 16 words (512 bits) of entropy at our
        // disposal. The idea here is to read all bits and copy the
        // necessary bytes to the slice.
        for chunk in dest.chunks_mut(64) {
            self.async_fill_chunk(chunk).await?;
        }

        Ok(())
    }

    async fn async_fill_chunk(&mut self, chunk: &mut [u8]) -> Result<(), Error> {
        // wait for interrupt
        let res = poll_fn(|cx| {
            // Check if already ready.
            // TODO: Is this necessary? Could we just check once after
            // the waker has been registered?
            if self.info.regs.int_status().read().ent_val() == IntStatusEntVal::ENT_VAL_1 {
                return Poll::Ready(Ok(()));
            }

            RNG_WAKER.register(cx.waker());

            self.unmask_interrupts();

            let mctl = self.info.regs.mctl().read();

            // Check again if interrupt fired
            if mctl.ent_val() {
                Poll::Ready(Ok(()))
            } else if mctl.err() {
                Poll::Ready(Err(Error::HwError))
            } else if mctl.fct_fail() {
                Poll::Ready(Err(Error::FreqCountFail))
            } else {
                Poll::Pending
            }
        })
        .await;

        let bits = self.info.regs.mctl().read();

        if bits.ent_val() {
            let mut entropy = [0; 16];

            for (i, item) in entropy.iter_mut().enumerate() {
                *item = self.info.regs.ent(i).read().ent();
            }

            // Read MCTL after reading ENT15
            let _ = self.info.regs.mctl().read();

            if entropy.contains(&0) {
                return Err(Error::SeedError);
            }

            // SAFETY: entropy is the same for input and output types in
            // native endianness.
            let entropy: [u8; 64] = unsafe { core::mem::transmute(entropy) };

            // write bytes to chunk
            chunk.copy_from_slice(&entropy[..chunk.len()]);
        }

        res
    }

    fn mask_interrupts(&mut self) {
        self.info.regs.int_mask().write(|w| {
            w.set_ent_val(IntMaskEntVal::ENT_VAL_0);
            w.set_hw_err(IntMaskHwErr::HW_ERR_0);
            w.set_frq_ct_fail(IntMaskFrqCtFail::FRQ_CT_FAIL_0);
        });
    }

    fn unmask_interrupts(&mut self) {
        self.info.regs.int_mask().modify(|w| {
            w.set_ent_val(IntMaskEntVal::ENT_VAL_1);
            w.set_hw_err(IntMaskHwErr::HW_ERR_1);
            w.set_frq_ct_fail(IntMaskFrqCtFail::FRQ_CT_FAIL_1);
        });
    }

    fn enable_interrupts(&mut self) {
        self.info.regs.int_ctrl().write(|w| {
            w.set_ent_val(IntCtrlEntVal::ENT_VAL_1);
            w.set_hw_err(IntCtrlHwErr::HW_ERR_1);
            w.set_frq_ct_fail(IntCtrlFrqCtFail::FRQ_CT_FAIL_1);
        });
    }

    fn init(&mut self) {
        self.mask_interrupts();

        // Switch TRNG to programming mode
        self.info.regs.mctl().modify(|w| w.set_prgm(true));

        self.enable_interrupts();

        // Switch TRNG to Run Mode
        self.info.regs.mctl().modify(|w| {
            w.set_trng_acc(true);
            w.set_prgm(false);
        });
    }

    /// Generate a random u32
    pub fn blocking_next_u32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        block_on(self.async_fill_bytes(&mut bytes)).unwrap();
        u32::from_ne_bytes(bytes)
    }

    /// Generate a random u64
    pub fn blocking_next_u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        block_on(self.async_fill_bytes(&mut bytes)).unwrap();
        u64::from_ne_bytes(bytes)
    }

    /// Fill a slice with random bytes.
    pub fn blocking_fill_bytes(&mut self, dest: &mut [u8]) {
        block_on(self.async_fill_bytes(dest)).unwrap();
    }
}

impl<'d> rand_core_06::RngCore for Rng<'d> {
    fn next_u32(&mut self) -> u32 {
        self.blocking_next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.blocking_next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.blocking_fill_bytes(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core_06::Error> {
        self.blocking_fill_bytes(dest);
        Ok(())
    }
}

impl<'d> rand_core_06::CryptoRng for Rng<'d> {}

impl<'d> rand_core_09::RngCore for Rng<'d> {
    fn next_u32(&mut self) -> u32 {
        self.blocking_next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.blocking_next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.blocking_fill_bytes(dest);
    }
}

impl<'d> rand_core_09::CryptoRng for Rng<'d> {}

impl<'d> rand_core_10::TryRng for Rng<'d> {
    type Error = core::convert::Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(self.blocking_next_u32())
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        Ok(self.blocking_next_u64())
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        self.blocking_fill_bytes(dest);
        Ok(())
    }
}

impl<'d> rand_core_10::TryCryptoRng for Rng<'d> {}

struct Info {
    regs: crate::pac::trng::Trng,
}

trait SealedInstance {
    fn info() -> Info;
}

/// RNG instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + SysconPeripheral + 'static + Send {
    /// Interrupt for this RNG instance.
    type Interrupt: interrupt::typelevel::Interrupt;
}

impl Instance for peripherals::RNG {
    type Interrupt = crate::interrupt::typelevel::RNG;
}

impl SealedInstance for peripherals::RNG {
    fn info() -> Info {
        Info { regs: crate::pac::TRNG }
    }
}
