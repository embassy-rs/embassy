#[doc = "Register `STAT` reader"]
pub type R = crate::R<StatSpec>;
#[doc = "Register `STAT` writer"]
pub type W = crate::W<StatSpec>;
#[doc = "Result FIFO 0 Ready Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdy0 {
    #[doc = "0: Result FIFO 0 data level not above watermark level."]
    BelowThreshold = 0,
    #[doc = "1: Result FIFO 0 holding data above watermark level."]
    AboveThreshold = 1,
}
impl From<Rdy0> for bool {
    #[inline(always)]
    fn from(variant: Rdy0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RDY0` reader - Result FIFO 0 Ready Flag"]
pub type Rdy0R = crate::BitReader<Rdy0>;
impl Rdy0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rdy0 {
        match self.bits {
            false => Rdy0::BelowThreshold,
            true => Rdy0::AboveThreshold,
        }
    }
    #[doc = "Result FIFO 0 data level not above watermark level."]
    #[inline(always)]
    pub fn is_below_threshold(&self) -> bool {
        *self == Rdy0::BelowThreshold
    }
    #[doc = "Result FIFO 0 holding data above watermark level."]
    #[inline(always)]
    pub fn is_above_threshold(&self) -> bool {
        *self == Rdy0::AboveThreshold
    }
}
#[doc = "Result FIFO 0 Overflow Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fof0 {
    #[doc = "0: No result FIFO 0 overflow has occurred since the last time the flag was cleared."]
    NoOverflow = 0,
    #[doc = "1: At least one result FIFO 0 overflow has occurred since the last time the flag was cleared."]
    OverflowDetected = 1,
}
impl From<Fof0> for bool {
    #[inline(always)]
    fn from(variant: Fof0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FOF0` reader - Result FIFO 0 Overflow Flag"]
pub type Fof0R = crate::BitReader<Fof0>;
impl Fof0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fof0 {
        match self.bits {
            false => Fof0::NoOverflow,
            true => Fof0::OverflowDetected,
        }
    }
    #[doc = "No result FIFO 0 overflow has occurred since the last time the flag was cleared."]
    #[inline(always)]
    pub fn is_no_overflow(&self) -> bool {
        *self == Fof0::NoOverflow
    }
    #[doc = "At least one result FIFO 0 overflow has occurred since the last time the flag was cleared."]
    #[inline(always)]
    pub fn is_overflow_detected(&self) -> bool {
        *self == Fof0::OverflowDetected
    }
}
#[doc = "Field `FOF0` writer - Result FIFO 0 Overflow Flag"]
pub type Fof0W<'a, REG> = crate::BitWriter1C<'a, REG, Fof0>;
impl<'a, REG> Fof0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No result FIFO 0 overflow has occurred since the last time the flag was cleared."]
    #[inline(always)]
    pub fn no_overflow(self) -> &'a mut crate::W<REG> {
        self.variant(Fof0::NoOverflow)
    }
    #[doc = "At least one result FIFO 0 overflow has occurred since the last time the flag was cleared."]
    #[inline(always)]
    pub fn overflow_detected(self) -> &'a mut crate::W<REG> {
        self.variant(Fof0::OverflowDetected)
    }
}
#[doc = "Interrupt Flag For High Priority Trigger Exception\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TexcInt {
    #[doc = "0: No trigger exceptions have occurred."]
    NoException = 0,
    #[doc = "1: A trigger exception has occurred and is pending acknowledgement."]
    ExceptionDetected = 1,
}
impl From<TexcInt> for bool {
    #[inline(always)]
    fn from(variant: TexcInt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TEXC_INT` reader - Interrupt Flag For High Priority Trigger Exception"]
pub type TexcIntR = crate::BitReader<TexcInt>;
impl TexcIntR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TexcInt {
        match self.bits {
            false => TexcInt::NoException,
            true => TexcInt::ExceptionDetected,
        }
    }
    #[doc = "No trigger exceptions have occurred."]
    #[inline(always)]
    pub fn is_no_exception(&self) -> bool {
        *self == TexcInt::NoException
    }
    #[doc = "A trigger exception has occurred and is pending acknowledgement."]
    #[inline(always)]
    pub fn is_exception_detected(&self) -> bool {
        *self == TexcInt::ExceptionDetected
    }
}
#[doc = "Field `TEXC_INT` writer - Interrupt Flag For High Priority Trigger Exception"]
pub type TexcIntW<'a, REG> = crate::BitWriter1C<'a, REG, TexcInt>;
impl<'a, REG> TexcIntW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No trigger exceptions have occurred."]
    #[inline(always)]
    pub fn no_exception(self) -> &'a mut crate::W<REG> {
        self.variant(TexcInt::NoException)
    }
    #[doc = "A trigger exception has occurred and is pending acknowledgement."]
    #[inline(always)]
    pub fn exception_detected(self) -> &'a mut crate::W<REG> {
        self.variant(TexcInt::ExceptionDetected)
    }
}
#[doc = "Interrupt Flag For Trigger Completion\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TcompInt {
    #[doc = "0: Either IE\\[TCOMP_IE\\] is set to 0, or no trigger sequences have run to completion."]
    FlagClear = 0,
    #[doc = "1: Trigger sequence has been completed and all data is stored in the associated FIFO."]
    CompletionDetected = 1,
}
impl From<TcompInt> for bool {
    #[inline(always)]
    fn from(variant: TcompInt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TCOMP_INT` reader - Interrupt Flag For Trigger Completion"]
pub type TcompIntR = crate::BitReader<TcompInt>;
impl TcompIntR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TcompInt {
        match self.bits {
            false => TcompInt::FlagClear,
            true => TcompInt::CompletionDetected,
        }
    }
    #[doc = "Either IE\\[TCOMP_IE\\] is set to 0, or no trigger sequences have run to completion."]
    #[inline(always)]
    pub fn is_flag_clear(&self) -> bool {
        *self == TcompInt::FlagClear
    }
    #[doc = "Trigger sequence has been completed and all data is stored in the associated FIFO."]
    #[inline(always)]
    pub fn is_completion_detected(&self) -> bool {
        *self == TcompInt::CompletionDetected
    }
}
#[doc = "Field `TCOMP_INT` writer - Interrupt Flag For Trigger Completion"]
pub type TcompIntW<'a, REG> = crate::BitWriter1C<'a, REG, TcompInt>;
impl<'a, REG> TcompIntW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Either IE\\[TCOMP_IE\\] is set to 0, or no trigger sequences have run to completion."]
    #[inline(always)]
    pub fn flag_clear(self) -> &'a mut crate::W<REG> {
        self.variant(TcompInt::FlagClear)
    }
    #[doc = "Trigger sequence has been completed and all data is stored in the associated FIFO."]
    #[inline(always)]
    pub fn completion_detected(self) -> &'a mut crate::W<REG> {
        self.variant(TcompInt::CompletionDetected)
    }
}
#[doc = "Calibration Ready\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalRdy {
    #[doc = "0: Calibration is incomplete or hasn't been ran."]
    NotSet = 0,
    #[doc = "1: The ADC is calibrated."]
    HardwareCalStepCompleted = 1,
}
impl From<CalRdy> for bool {
    #[inline(always)]
    fn from(variant: CalRdy) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAL_RDY` reader - Calibration Ready"]
pub type CalRdyR = crate::BitReader<CalRdy>;
impl CalRdyR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CalRdy {
        match self.bits {
            false => CalRdy::NotSet,
            true => CalRdy::HardwareCalStepCompleted,
        }
    }
    #[doc = "Calibration is incomplete or hasn't been ran."]
    #[inline(always)]
    pub fn is_not_set(&self) -> bool {
        *self == CalRdy::NotSet
    }
    #[doc = "The ADC is calibrated."]
    #[inline(always)]
    pub fn is_hardware_cal_step_completed(&self) -> bool {
        *self == CalRdy::HardwareCalStepCompleted
    }
}
#[doc = "ADC Active\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdcActive {
    #[doc = "0: The ADC is IDLE. There are no pending triggers to service and no active commands are being processed."]
    NotActive = 0,
    #[doc = "1: The ADC is processing a conversion, running through the power up delay, or servicing a trigger."]
    Busy = 1,
}
impl From<AdcActive> for bool {
    #[inline(always)]
    fn from(variant: AdcActive) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ADC_ACTIVE` reader - ADC Active"]
pub type AdcActiveR = crate::BitReader<AdcActive>;
impl AdcActiveR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> AdcActive {
        match self.bits {
            false => AdcActive::NotActive,
            true => AdcActive::Busy,
        }
    }
    #[doc = "The ADC is IDLE. There are no pending triggers to service and no active commands are being processed."]
    #[inline(always)]
    pub fn is_not_active(&self) -> bool {
        *self == AdcActive::NotActive
    }
    #[doc = "The ADC is processing a conversion, running through the power up delay, or servicing a trigger."]
    #[inline(always)]
    pub fn is_busy(&self) -> bool {
        *self == AdcActive::Busy
    }
}
#[doc = "Trigger Active\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Trgact {
    #[doc = "0: Command (sequence) associated with Trigger 0 currently being executed."]
    Trig0 = 0,
    #[doc = "1: Command (sequence) associated with Trigger 1 currently being executed."]
    Trig1 = 1,
    #[doc = "2: Command (sequence) associated with Trigger 2 currently being executed."]
    Trig2 = 2,
    #[doc = "3: Command (sequence) associated with Trigger 3 currently being executed."]
    Trig3 = 3,
}
impl From<Trgact> for u8 {
    #[inline(always)]
    fn from(variant: Trgact) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Trgact {
    type Ux = u8;
}
impl crate::IsEnum for Trgact {}
#[doc = "Field `TRGACT` reader - Trigger Active"]
pub type TrgactR = crate::FieldReader<Trgact>;
impl TrgactR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Trgact {
        match self.bits {
            0 => Trgact::Trig0,
            1 => Trgact::Trig1,
            2 => Trgact::Trig2,
            3 => Trgact::Trig3,
            _ => unreachable!(),
        }
    }
    #[doc = "Command (sequence) associated with Trigger 0 currently being executed."]
    #[inline(always)]
    pub fn is_trig_0(&self) -> bool {
        *self == Trgact::Trig0
    }
    #[doc = "Command (sequence) associated with Trigger 1 currently being executed."]
    #[inline(always)]
    pub fn is_trig_1(&self) -> bool {
        *self == Trgact::Trig1
    }
    #[doc = "Command (sequence) associated with Trigger 2 currently being executed."]
    #[inline(always)]
    pub fn is_trig_2(&self) -> bool {
        *self == Trgact::Trig2
    }
    #[doc = "Command (sequence) associated with Trigger 3 currently being executed."]
    #[inline(always)]
    pub fn is_trig_3(&self) -> bool {
        *self == Trgact::Trig3
    }
}
#[doc = "Command Active\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Cmdact {
    #[doc = "0: No command is currently in progress."]
    NoCommandActive = 0,
    #[doc = "1: Command 1 currently being executed."]
    Command1 = 1,
    #[doc = "2: Command 2 currently being executed."]
    Command2 = 2,
    #[doc = "3: Associated command number is currently being executed."]
    CommandX3 = 3,
    #[doc = "4: Associated command number is currently being executed."]
    CommandX4 = 4,
    #[doc = "5: Associated command number is currently being executed."]
    CommandX5 = 5,
    #[doc = "6: Associated command number is currently being executed."]
    CommandX6 = 6,
    #[doc = "7: Associated command number is currently being executed."]
    CommandX7 = 7,
}
impl From<Cmdact> for u8 {
    #[inline(always)]
    fn from(variant: Cmdact) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Cmdact {
    type Ux = u8;
}
impl crate::IsEnum for Cmdact {}
#[doc = "Field `CMDACT` reader - Command Active"]
pub type CmdactR = crate::FieldReader<Cmdact>;
impl CmdactR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cmdact {
        match self.bits {
            0 => Cmdact::NoCommandActive,
            1 => Cmdact::Command1,
            2 => Cmdact::Command2,
            3 => Cmdact::CommandX3,
            4 => Cmdact::CommandX4,
            5 => Cmdact::CommandX5,
            6 => Cmdact::CommandX6,
            7 => Cmdact::CommandX7,
            _ => unreachable!(),
        }
    }
    #[doc = "No command is currently in progress."]
    #[inline(always)]
    pub fn is_no_command_active(&self) -> bool {
        *self == Cmdact::NoCommandActive
    }
    #[doc = "Command 1 currently being executed."]
    #[inline(always)]
    pub fn is_command_1(&self) -> bool {
        *self == Cmdact::Command1
    }
    #[doc = "Command 2 currently being executed."]
    #[inline(always)]
    pub fn is_command_2(&self) -> bool {
        *self == Cmdact::Command2
    }
    #[doc = "Associated command number is currently being executed."]
    #[inline(always)]
    pub fn is_command_x_3(&self) -> bool {
        *self == Cmdact::CommandX3
    }
    #[doc = "Associated command number is currently being executed."]
    #[inline(always)]
    pub fn is_command_x_4(&self) -> bool {
        *self == Cmdact::CommandX4
    }
    #[doc = "Associated command number is currently being executed."]
    #[inline(always)]
    pub fn is_command_x_5(&self) -> bool {
        *self == Cmdact::CommandX5
    }
    #[doc = "Associated command number is currently being executed."]
    #[inline(always)]
    pub fn is_command_x_6(&self) -> bool {
        *self == Cmdact::CommandX6
    }
    #[doc = "Associated command number is currently being executed."]
    #[inline(always)]
    pub fn is_command_x_7(&self) -> bool {
        *self == Cmdact::CommandX7
    }
}
impl R {
    #[doc = "Bit 0 - Result FIFO 0 Ready Flag"]
    #[inline(always)]
    pub fn rdy0(&self) -> Rdy0R {
        Rdy0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Result FIFO 0 Overflow Flag"]
    #[inline(always)]
    pub fn fof0(&self) -> Fof0R {
        Fof0R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 8 - Interrupt Flag For High Priority Trigger Exception"]
    #[inline(always)]
    pub fn texc_int(&self) -> TexcIntR {
        TexcIntR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Interrupt Flag For Trigger Completion"]
    #[inline(always)]
    pub fn tcomp_int(&self) -> TcompIntR {
        TcompIntR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Calibration Ready"]
    #[inline(always)]
    pub fn cal_rdy(&self) -> CalRdyR {
        CalRdyR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - ADC Active"]
    #[inline(always)]
    pub fn adc_active(&self) -> AdcActiveR {
        AdcActiveR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bits 16:17 - Trigger Active"]
    #[inline(always)]
    pub fn trgact(&self) -> TrgactR {
        TrgactR::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bits 24:26 - Command Active"]
    #[inline(always)]
    pub fn cmdact(&self) -> CmdactR {
        CmdactR::new(((self.bits >> 24) & 7) as u8)
    }
}
impl W {
    #[doc = "Bit 1 - Result FIFO 0 Overflow Flag"]
    #[inline(always)]
    pub fn fof0(&mut self) -> Fof0W<StatSpec> {
        Fof0W::new(self, 1)
    }
    #[doc = "Bit 8 - Interrupt Flag For High Priority Trigger Exception"]
    #[inline(always)]
    pub fn texc_int(&mut self) -> TexcIntW<StatSpec> {
        TexcIntW::new(self, 8)
    }
    #[doc = "Bit 9 - Interrupt Flag For Trigger Completion"]
    #[inline(always)]
    pub fn tcomp_int(&mut self) -> TcompIntW<StatSpec> {
        TcompIntW::new(self, 9)
    }
}
#[doc = "Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`stat::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stat::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StatSpec;
impl crate::RegisterSpec for StatSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`stat::R`](R) reader structure"]
impl crate::Readable for StatSpec {}
#[doc = "`write(|w| ..)` method takes [`stat::W`](W) writer structure"]
impl crate::Writable for StatSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0302;
}
#[doc = "`reset()` method sets STAT to value 0"]
impl crate::Resettable for StatSpec {}
