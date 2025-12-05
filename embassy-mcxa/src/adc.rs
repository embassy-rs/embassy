//! ADC driver
use core::sync::atomic::{AtomicBool, Ordering};

use embassy_hal_internal::{Peri, PeripheralType};

use crate::clocks::periph_helpers::{AdcClockSel, AdcConfig, Div4};
use crate::clocks::{Gate, PoweredClock, enable_and_reset};
use crate::pac;
use crate::pac::adc1::cfg::{HptExdi, Pwrsel, Refsel, Tcmdres, Tprictrl, Tres};
use crate::pac::adc1::cmdh1::{Avgs, Cmpen, Next, Sts};
use crate::pac::adc1::cmdl1::{Adch, Ctype, Mode};
use crate::pac::adc1::ctrl::CalAvgs;
use crate::pac::adc1::tctrl::{Tcmd, Tpri};

type Regs = pac::adc1::RegisterBlock;

static INTERRUPT_TRIGGERED: AtomicBool = AtomicBool::new(false);
// Token-based instance pattern like embassy-imxrt
pub trait Instance: Gate<MrccPeriphConfig = AdcConfig> + PeripheralType {
    fn ptr() -> *const Regs;
}

/// Token for ADC1
pub type Adc1 = crate::peripherals::ADC1;
impl Instance for crate::peripherals::ADC1 {
    #[inline(always)]
    fn ptr() -> *const Regs {
        pac::Adc1::ptr()
    }
}

// Also implement Instance for the Peri wrapper type
// impl Instance for embassy_hal_internal::Peri<'_, crate::peripherals::ADC1> {
//     #[inline(always)]
//     fn ptr() -> *const Regs {
//         pac::Adc1::ptr()
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TriggerPriorityPolicy {
    ConvPreemptImmediatelyNotAutoResumed = 0,
    ConvPreemptSoftlyNotAutoResumed = 1,
    ConvPreemptImmediatelyAutoRestarted = 4,
    ConvPreemptSoftlyAutoRestarted = 5,
    ConvPreemptImmediatelyAutoResumed = 12,
    ConvPreemptSoftlyAutoResumed = 13,
    ConvPreemptSubsequentlyNotAutoResumed = 2,
    ConvPreemptSubsequentlyAutoRestarted = 6,
    ConvPreemptSubsequentlyAutoResumed = 14,
    TriggerPriorityExceptionDisabled = 16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LpadcConfig {
    pub enable_in_doze_mode: bool,
    pub conversion_average_mode: CalAvgs,
    pub enable_analog_preliminary: bool,
    pub power_up_delay: u8,
    pub reference_voltage_source: Refsel,
    pub power_level_mode: Pwrsel,
    pub trigger_priority_policy: TriggerPriorityPolicy,
    pub enable_conv_pause: bool,
    pub conv_pause_delay: u16,
    pub fifo_watermark: u8,
    pub power: PoweredClock,
    pub source: AdcClockSel,
    pub div: Div4,
}

impl Default for LpadcConfig {
    fn default() -> Self {
        LpadcConfig {
            enable_in_doze_mode: true,
            conversion_average_mode: CalAvgs::NoAverage,
            enable_analog_preliminary: false,
            power_up_delay: 0x80,
            reference_voltage_source: Refsel::Option1,
            power_level_mode: Pwrsel::Lowest,
            trigger_priority_policy: TriggerPriorityPolicy::ConvPreemptImmediatelyNotAutoResumed,
            enable_conv_pause: false,
            conv_pause_delay: 0,
            fifo_watermark: 0,
            power: PoweredClock::NormalEnabledDeepSleepDisabled,
            source: AdcClockSel::FroLfDiv,
            div: Div4::no_div(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConvCommandConfig {
    pub sample_channel_mode: Ctype,
    pub channel_number: Adch,
    pub chained_next_command_number: Next,
    pub enable_auto_channel_increment: bool,
    pub loop_count: u8,
    pub hardware_average_mode: Avgs,
    pub sample_time_mode: Sts,
    pub hardware_compare_mode: Cmpen,
    pub hardware_compare_value_high: u32,
    pub hardware_compare_value_low: u32,
    pub conversion_resolution_mode: Mode,
    pub enable_wait_trigger: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConvTriggerConfig {
    pub target_command_id: Tcmd,
    pub delay_power: u8,
    pub priority: Tpri,
    pub enable_hardware_trigger: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConvResult {
    pub command_id_source: u32,
    pub loop_count_index: u32,
    pub trigger_id_source: u32,
    pub conv_value: u16,
}

pub struct Adc<'a, I: Instance> {
    _inst: core::marker::PhantomData<&'a mut I>,
}

impl<'a, I: Instance> Adc<'a, I> {
    /// initialize ADC
    pub fn new(_inst: Peri<'a, I>, config: LpadcConfig) -> Self {
        let adc = unsafe { &*I::ptr() };

        let _clock_freq = unsafe {
            enable_and_reset::<I>(&AdcConfig {
                power: config.power,
                source: config.source,
                div: config.div,
            })
            .expect("Adc Init should not fail")
        };

        /* Reset the module. */
        adc.ctrl().modify(|_, w| w.rst().held_in_reset());
        adc.ctrl().modify(|_, w| w.rst().released_from_reset());

        adc.ctrl().modify(|_, w| w.rstfifo0().trigger_reset());

        /* Disable the module before setting configuration. */
        adc.ctrl().modify(|_, w| w.adcen().disabled());

        /* Configure the module generally. */
        if config.enable_in_doze_mode {
            adc.ctrl().modify(|_, w| w.dozen().enabled());
        } else {
            adc.ctrl().modify(|_, w| w.dozen().disabled());
        }

        /* Set calibration average mode. */
        adc.ctrl()
            .modify(|_, w| w.cal_avgs().variant(config.conversion_average_mode));

        adc.cfg().write(|w| unsafe {
            let w = if config.enable_analog_preliminary {
                w.pwren().pre_enabled()
            } else {
                w
            };

            w.pudly()
                .bits(config.power_up_delay)
                .refsel()
                .variant(config.reference_voltage_source)
                .pwrsel()
                .variant(config.power_level_mode)
                .tprictrl()
                .variant(match config.trigger_priority_policy {
                    TriggerPriorityPolicy::ConvPreemptSoftlyNotAutoResumed
                    | TriggerPriorityPolicy::ConvPreemptSoftlyAutoRestarted
                    | TriggerPriorityPolicy::ConvPreemptSoftlyAutoResumed => Tprictrl::FinishCurrentOnPriority,
                    TriggerPriorityPolicy::ConvPreemptSubsequentlyNotAutoResumed
                    | TriggerPriorityPolicy::ConvPreemptSubsequentlyAutoRestarted
                    | TriggerPriorityPolicy::ConvPreemptSubsequentlyAutoResumed => Tprictrl::FinishSequenceOnPriority,
                    _ => Tprictrl::AbortCurrentOnPriority,
                })
                .tres()
                .variant(match config.trigger_priority_policy {
                    TriggerPriorityPolicy::ConvPreemptImmediatelyAutoRestarted
                    | TriggerPriorityPolicy::ConvPreemptSoftlyAutoRestarted
                    | TriggerPriorityPolicy::ConvPreemptImmediatelyAutoResumed
                    | TriggerPriorityPolicy::ConvPreemptSoftlyAutoResumed
                    | TriggerPriorityPolicy::ConvPreemptSubsequentlyAutoRestarted
                    | TriggerPriorityPolicy::ConvPreemptSubsequentlyAutoResumed => Tres::Enabled,
                    _ => Tres::Disabled,
                })
                .tcmdres()
                .variant(match config.trigger_priority_policy {
                    TriggerPriorityPolicy::ConvPreemptImmediatelyAutoResumed
                    | TriggerPriorityPolicy::ConvPreemptSoftlyAutoResumed
                    | TriggerPriorityPolicy::ConvPreemptSubsequentlyAutoResumed
                    | TriggerPriorityPolicy::TriggerPriorityExceptionDisabled => Tcmdres::Enabled,
                    _ => Tcmdres::Disabled,
                })
                .hpt_exdi()
                .variant(match config.trigger_priority_policy {
                    TriggerPriorityPolicy::TriggerPriorityExceptionDisabled => HptExdi::Disabled,
                    _ => HptExdi::Enabled,
                })
        });

        if config.enable_conv_pause {
            adc.pause()
                .modify(|_, w| unsafe { w.pauseen().enabled().pausedly().bits(config.conv_pause_delay) });
        } else {
            adc.pause().write(|w| unsafe { w.bits(0) });
        }

        adc.fctrl0()
            .write(|w| unsafe { w.fwmark().bits(config.fifo_watermark) });

        // Enable ADC
        adc.ctrl().modify(|_, w| w.adcen().enabled());

        Self {
            _inst: core::marker::PhantomData,
        }
    }

    pub fn deinit(&self) {
        let adc = unsafe { &*I::ptr() };
        adc.ctrl().modify(|_, w| w.adcen().disabled());
    }

    pub fn do_offset_calibration(&self) {
        let adc = unsafe { &*I::ptr() };
        // Enable calibration mode
        adc.ctrl()
            .modify(|_, w| w.calofs().offset_calibration_request_pending());

        // Wait for calibration to complete (polling status register)
        while adc.stat().read().cal_rdy().is_not_set() {}
    }

    pub fn get_gain_conv_result(&self, mut gain_adjustment: f32) -> u32 {
        let mut gcra_array = [0u32; 17];
        let mut gcalr: u32 = 0;

        for i in (1..=17).rev() {
            let shift = 16 - (i - 1);
            let step = 1.0 / (1u32 << shift) as f32;
            let tmp = (gain_adjustment / step) as u32;
            gcra_array[i - 1] = tmp;
            gain_adjustment -= tmp as f32 * step;
        }

        for i in (1..=17).rev() {
            gcalr += gcra_array[i - 1] << (i - 1);
        }
        gcalr
    }

    pub fn do_auto_calibration(&self) {
        let adc = unsafe { &*I::ptr() };
        adc.ctrl().modify(|_, w| w.cal_req().calibration_request_pending());

        while adc.gcc0().read().rdy().is_gain_cal_not_valid() {}

        let mut gcca = adc.gcc0().read().gain_cal().bits() as u32;
        if gcca & ((0xFFFF + 1) >> 1) != 0 {
            gcca |= !0xFFFF;
        }

        let gcra = 131072.0 / (131072.0 - gcca as f32);

        // Write to GCR0
        adc.gcr0().write(|w| unsafe { w.bits(self.get_gain_conv_result(gcra)) });

        adc.gcr0().modify(|_, w| w.rdy().set_bit());

        // Wait for calibration to complete (polling status register)
        while adc.stat().read().cal_rdy().is_not_set() {}
    }

    pub fn do_software_trigger(&self, trigger_id_mask: u32) {
        let adc = unsafe { &*I::ptr() };
        adc.swtrig().write(|w| unsafe { w.bits(trigger_id_mask) });
    }

    pub fn get_default_conv_command_config(&self) -> ConvCommandConfig {
        ConvCommandConfig {
            sample_channel_mode: Ctype::SingleEndedASideChannel,
            channel_number: Adch::SelectCh0,
            chained_next_command_number: Next::NoNextCmdTerminateOnFinish,
            enable_auto_channel_increment: false,
            loop_count: 0,
            hardware_average_mode: Avgs::NoAverage,
            sample_time_mode: Sts::Sample3p5,
            hardware_compare_mode: Cmpen::DisabledAlwaysStoreResult,
            hardware_compare_value_high: 0,
            hardware_compare_value_low: 0,
            conversion_resolution_mode: Mode::Data12Bits,
            enable_wait_trigger: false,
        }
    }

    //TBD Need to add cmdlx and cmdhx with x {2..7}
    pub fn set_conv_command_config(&self, index: u32, config: &ConvCommandConfig) {
        let adc = unsafe { &*I::ptr() };

        match index {
            1 => {
                adc.cmdl1().write(|w| {
                    w.adch()
                        .variant(config.channel_number)
                        .mode()
                        .variant(config.conversion_resolution_mode)
                });
                adc.cmdh1().write(|w| unsafe {
                    w.next()
                        .variant(config.chained_next_command_number)
                        .loop_()
                        .bits(config.loop_count)
                        .avgs()
                        .variant(config.hardware_average_mode)
                        .sts()
                        .variant(config.sample_time_mode)
                        .cmpen()
                        .variant(config.hardware_compare_mode);
                    if config.enable_wait_trigger {
                        w.wait_trig().enabled();
                    }
                    if config.enable_auto_channel_increment {
                        w.lwi().enabled();
                    }
                    w
                });
            }
            _ => panic!("Invalid command index: must be between 1 and 7"),
        }
    }

    pub fn get_default_conv_trigger_config(&self) -> ConvTriggerConfig {
        ConvTriggerConfig {
            target_command_id: Tcmd::NotValid,
            delay_power: 0,
            priority: Tpri::HighestPriority,
            enable_hardware_trigger: false,
        }
    }

    pub fn set_conv_trigger_config(&self, trigger_id: usize, config: &ConvTriggerConfig) {
        let adc = unsafe { &*I::ptr() };
        let tctrl = &adc.tctrl(trigger_id);

        tctrl.write(|w| unsafe {
            let w = w.tcmd().variant(config.target_command_id);
            let w = w.tdly().bits(config.delay_power);
            w.tpri().variant(config.priority);
            if config.enable_hardware_trigger {
                w.hten().enabled()
            } else {
                w
            }
        });
    }

    pub fn do_reset_fifo(&self) {
        let adc = unsafe { &*I::ptr() };
        adc.ctrl().modify(|_, w| w.rstfifo0().trigger_reset());
    }

    pub fn enable_interrupt(&self, mask: u32) {
        let adc = unsafe { &*I::ptr() };
        adc.ie().modify(|r, w| unsafe { w.bits(r.bits() | mask) });
        INTERRUPT_TRIGGERED.store(false, Ordering::SeqCst);
    }

    pub fn is_interrupt_triggered(&self) -> bool {
        INTERRUPT_TRIGGERED.load(Ordering::Relaxed)
    }
}

pub fn get_conv_result() -> Option<ConvResult> {
    let adc = unsafe { &*pac::Adc1::ptr() };
    let fifo = adc.resfifo0().read().bits();
    const VALID_MASK: u32 = 1 << 31;
    if fifo & VALID_MASK == 0 {
        return None;
    }

    Some(ConvResult {
        command_id_source: (fifo >> 24) & 0x0F,
        loop_count_index: (fifo >> 20) & 0x0F,
        trigger_id_source: (fifo >> 16) & 0x0F,
        conv_value: (fifo & 0xFFFF) as u16,
    })
}

pub fn on_interrupt() {
    if get_conv_result().is_some() {
        INTERRUPT_TRIGGERED.store(true, Ordering::SeqCst);
    }
}

pub struct AdcHandler;
impl crate::interrupt::typelevel::Handler<crate::interrupt::typelevel::ADC1> for AdcHandler {
    unsafe fn on_interrupt() {
        on_interrupt();
    }
}
