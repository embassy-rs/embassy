use core::marker::PhantomData;

use atomic_polyfill::{compiler_fence, Ordering};
use embassy_hal_internal::{into_ref, Peripheral};
use embassy_time::Timer;
use rp_pac::dma::vals::{DataSize, TreqSel};
use rp_pac::pwm::vals::Divmode;

use super::{BuilderState, ConfigureDivider, ConfigurePhaseCorrect, DivMode, PwmBuilder, SliceConfig};
use crate::builder_state;
use crate::dma::Channel;
use crate::gpio::SealedPin as _;
use crate::pwm::v2::{PwmCounter, PwmError};
use crate::pwm::{ChannelBPin, Slice};

/// The Timer registers start at a base address of 0x40054000 (defined as TIMER_BASE in SDK).
const TIMER_BASE: u32 = 0x40054000;
/// Address of lower 32 bits of 1 MHz timer
const TIMER_RAWL_ADDR: u32 = TIMER_BASE + 0x28;

const PWM_DREQ_BASE: u8 = 0x18;

/// The PWM DREQ for PWM slice 0. The DREQ is used by the PWM slice to request
/// the next value from the DMA channel. We only record the DREQ for slice 0
/// here, as the DREQ for the other slices are simply the DREQ for slice 0
/// incremented by the slice number (i.e. the DREQ for slice 5 would be
/// [`DREQ_PWM_WRAP0`] + 5).
pub const DREQ_PWM_WRAP0: u8 = 0x18;
/// DMACounter state object for the PWM builder.
pub struct DmaEdgeTimer<const SAMPLE_COUNT: usize = 9>(SliceConfig);

impl<const SAMPLE_COUNT: usize> BuilderState for DmaEdgeTimer<SAMPLE_COUNT> {
    fn get_config(&mut self) -> &mut SliceConfig {
        &mut self.0
    }
    fn get_config_owned(self) -> SliceConfig {
        self.0
    }
}

impl ConfigureDivider for DmaEdgeTimer {}
impl ConfigurePhaseCorrect for DmaEdgeTimer {}

impl PwmBuilder<DivMode> {
    pub fn edge_timer(self) -> PwmBuilder<DmaEdgeTimer> {
        PwmBuilder {
            config: SliceConfig {
                enable_dma: true,
                div_mode: Divmode::RISE,
                div: 250,
                ..self.config
            },
            _phantom: PhantomData,
        }
    }
}

impl<const SAMPLE_COUNT: usize> PwmBuilder<DmaEdgeTimer<SAMPLE_COUNT>> {
    /// Set the sample count for the edge timer. Pass the sample size as a
    /// constant value, e.g. `.with_sample_size::<9>()`.
    ///
    /// Each sample is 32 bits wide, so the total size requirement will be
    /// `SAMPLE_COUNT * 4` bytes.
    pub fn with_sample_size<const SIZE: usize>(self) -> PwmBuilder<DmaEdgeTimer<SIZE>> {
        PwmBuilder::<DmaEdgeTimer<SIZE>> {
            _phantom: PhantomData,
            config: self.config,
        }
    }

    /// Apply the configuration to the provided PWM slice, DMA channel and
    /// input pin.
    pub async fn apply<'a, PWM: Slice, DMA: Channel>(
        self,
        pwm_slice: impl Peripheral<P = PWM> + 'static,
        dma_channel: impl Peripheral<P = DMA> + 'static,
        input_pin: impl Peripheral<P = impl ChannelBPin<PWM>> + 'static,
    ) -> Result<PwmCounter<'a, PWM, DMA>, PwmError> {
        if ![Divmode::RISE, Divmode::FALL].contains(&self.config.div_mode) {
            return Err(PwmError::InvalidDivMode);
        }

        into_ref!(pwm_slice);
        into_ref!(dma_channel);
        into_ref!(input_pin);

        let dma_channel_number = dma_channel.number();
        let instance = PwmCounter::new(pwm_slice, dma_channel, input_pin.map_into());

        // Get an instance of the registers for the PWM slice, DMA channel and GPIO pin.
        let pwm_regs = instance.pwm_slice.regs();
        let dma_regs = instance.dma_channel.regs();

        // Configure the PWM slice as a timer.
        pwm_regs.csr().write(|w| {
            w.set_divmode(Divmode::RISE);
            w.set_en(false);
        });

        // Set the fractional divider to `1`.
        pwm_regs.div().write(|w| {
            w.set_int(instance.divider);
            w.set_frac(0);
        });

        // Set the top/wrap value to `0`.
        pwm_regs.top().write(|w| {
            w.set_top(0);
        });

        pwm_regs.cc().write_value(Default::default());

        // Set the GPIO pin to PWM mode.
        instance.pwm_pin.gpio().ctrl().write(|w| {
            w.set_funcsel(4);
        });
        instance.pwm_pin.pad_ctrl().write(|w| {
            w.set_od(true);
            w.set_ie(true);
        });

        // Abort the DMA channel before enabling the PWM slice to ensure that
        // there are no currently running transfers.
        trace!("Requesting DMA channel {} to abort.", dma_channel_number);
        rp_pac::DMA.chan_abort().write(|w| {
            w.set_chan_abort(1 << dma_channel_number);
        });

        // Wait for the DMA channel to abort before enabling the PWM slice,
        // otherwise it is unsafe to start if there are any running transfers.
        trace!("Waiting for DMA channel {} to abort.", dma_channel_number);
        while rp_pac::DMA.chan_abort().read().chan_abort() != 0 {
            Timer::after_micros(5).await;
        }

        // Configure the DMA slice:
        // - Set the data size to `SIZE_WORD` (32 bits).
        // - Do not increment the read address on each read.
        // - Increment the write address by 32 bits on each write.
        // - Set the transfer trigger to the PWM slice's DREQ.
        dma_regs.ctrl_trig().write(|w| {
            w.set_data_size(DataSize::SIZE_WORD);
            w.set_incr_read(false);
            w.set_incr_write(true);
            w.set_treq_sel(TreqSel(PWM_DREQ_BASE + instance.pwm_slice.number()));
            w.set_chain_to(dma_channel_number);
            w.set_en(true);
        });
        dma_regs.trans_count().write_value(SAMPLE_COUNT as u32);

        // Set the read address to the lower 32 bits of the 1 MHz timer.
        dma_regs.read_addr().write_value(TIMER_RAWL_ADDR);

        Ok(instance)
    }
}
