#[doc = "Register `IE` reader"]
pub type R = crate::R<IeSpec>;
#[doc = "Register `IE` writer"]
pub type W = crate::W<IeSpec>;
#[doc = "FIFO 0 Watermark Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fwmie0 {
    #[doc = "0: FIFO 0 watermark interrupts are not enabled."]
    Disabled = 0,
    #[doc = "1: FIFO 0 watermark interrupts are enabled."]
    Enabled = 1,
}
impl From<Fwmie0> for bool {
    #[inline(always)]
    fn from(variant: Fwmie0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FWMIE0` reader - FIFO 0 Watermark Interrupt Enable"]
pub type Fwmie0R = crate::BitReader<Fwmie0>;
impl Fwmie0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fwmie0 {
        match self.bits {
            false => Fwmie0::Disabled,
            true => Fwmie0::Enabled,
        }
    }
    #[doc = "FIFO 0 watermark interrupts are not enabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Fwmie0::Disabled
    }
    #[doc = "FIFO 0 watermark interrupts are enabled."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Fwmie0::Enabled
    }
}
#[doc = "Field `FWMIE0` writer - FIFO 0 Watermark Interrupt Enable"]
pub type Fwmie0W<'a, REG> = crate::BitWriter<'a, REG, Fwmie0>;
impl<'a, REG> Fwmie0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "FIFO 0 watermark interrupts are not enabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fwmie0::Disabled)
    }
    #[doc = "FIFO 0 watermark interrupts are enabled."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fwmie0::Enabled)
    }
}
#[doc = "Result FIFO 0 Overflow Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fofie0 {
    #[doc = "0: FIFO 0 overflow interrupts are not enabled."]
    Disabled = 0,
    #[doc = "1: FIFO 0 overflow interrupts are enabled."]
    Enabled = 1,
}
impl From<Fofie0> for bool {
    #[inline(always)]
    fn from(variant: Fofie0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FOFIE0` reader - Result FIFO 0 Overflow Interrupt Enable"]
pub type Fofie0R = crate::BitReader<Fofie0>;
impl Fofie0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fofie0 {
        match self.bits {
            false => Fofie0::Disabled,
            true => Fofie0::Enabled,
        }
    }
    #[doc = "FIFO 0 overflow interrupts are not enabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Fofie0::Disabled
    }
    #[doc = "FIFO 0 overflow interrupts are enabled."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Fofie0::Enabled
    }
}
#[doc = "Field `FOFIE0` writer - Result FIFO 0 Overflow Interrupt Enable"]
pub type Fofie0W<'a, REG> = crate::BitWriter<'a, REG, Fofie0>;
impl<'a, REG> Fofie0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "FIFO 0 overflow interrupts are not enabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fofie0::Disabled)
    }
    #[doc = "FIFO 0 overflow interrupts are enabled."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Fofie0::Enabled)
    }
}
#[doc = "Trigger Exception Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TexcIe {
    #[doc = "0: Trigger exception interrupts are disabled."]
    Disabled = 0,
    #[doc = "1: Trigger exception interrupts are enabled."]
    Enabled = 1,
}
impl From<TexcIe> for bool {
    #[inline(always)]
    fn from(variant: TexcIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TEXC_IE` reader - Trigger Exception Interrupt Enable"]
pub type TexcIeR = crate::BitReader<TexcIe>;
impl TexcIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> TexcIe {
        match self.bits {
            false => TexcIe::Disabled,
            true => TexcIe::Enabled,
        }
    }
    #[doc = "Trigger exception interrupts are disabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == TexcIe::Disabled
    }
    #[doc = "Trigger exception interrupts are enabled."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == TexcIe::Enabled
    }
}
#[doc = "Field `TEXC_IE` writer - Trigger Exception Interrupt Enable"]
pub type TexcIeW<'a, REG> = crate::BitWriter<'a, REG, TexcIe>;
impl<'a, REG> TexcIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Trigger exception interrupts are disabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(TexcIe::Disabled)
    }
    #[doc = "Trigger exception interrupts are enabled."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(TexcIe::Enabled)
    }
}
#[doc = "Trigger Completion Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TcompIe {
    #[doc = "0: Trigger completion interrupts are disabled."]
    Disabled = 0,
    #[doc = "1: Trigger completion interrupts are enabled for trigger source 0 only."]
    Trigger0CompleteEnabled = 1,
    #[doc = "2: Trigger completion interrupts are enabled for trigger source 1 only."]
    Trigger1CompleteEnabled = 2,
    #[doc = "3: Associated trigger completion interrupts are enabled."]
    TriggerXCompleteEnabled3 = 3,
    #[doc = "4: Associated trigger completion interrupts are enabled."]
    TriggerXCompleteEnabled4 = 4,
    #[doc = "5: Associated trigger completion interrupts are enabled."]
    TriggerXCompleteEnabled5 = 5,
    #[doc = "6: Associated trigger completion interrupts are enabled."]
    TriggerXCompleteEnabled6 = 6,
    #[doc = "7: Associated trigger completion interrupts are enabled."]
    TriggerXCompleteEnabled7 = 7,
    #[doc = "8: Associated trigger completion interrupts are enabled."]
    TriggerXCompleteEnabled8 = 8,
    #[doc = "9: Associated trigger completion interrupts are enabled."]
    TriggerXCompleteEnabled9 = 9,
    #[doc = "15: Trigger completion interrupts are enabled for every trigger source."]
    AllTriggerCompletesEnabled = 15,
}
impl From<TcompIe> for u8 {
    #[inline(always)]
    fn from(variant: TcompIe) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for TcompIe {
    type Ux = u8;
}
impl crate::IsEnum for TcompIe {}
#[doc = "Field `TCOMP_IE` reader - Trigger Completion Interrupt Enable"]
pub type TcompIeR = crate::FieldReader<TcompIe>;
impl TcompIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<TcompIe> {
        match self.bits {
            0 => Some(TcompIe::Disabled),
            1 => Some(TcompIe::Trigger0CompleteEnabled),
            2 => Some(TcompIe::Trigger1CompleteEnabled),
            3 => Some(TcompIe::TriggerXCompleteEnabled3),
            4 => Some(TcompIe::TriggerXCompleteEnabled4),
            5 => Some(TcompIe::TriggerXCompleteEnabled5),
            6 => Some(TcompIe::TriggerXCompleteEnabled6),
            7 => Some(TcompIe::TriggerXCompleteEnabled7),
            8 => Some(TcompIe::TriggerXCompleteEnabled8),
            9 => Some(TcompIe::TriggerXCompleteEnabled9),
            15 => Some(TcompIe::AllTriggerCompletesEnabled),
            _ => None,
        }
    }
    #[doc = "Trigger completion interrupts are disabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == TcompIe::Disabled
    }
    #[doc = "Trigger completion interrupts are enabled for trigger source 0 only."]
    #[inline(always)]
    pub fn is_trigger_0_complete_enabled(&self) -> bool {
        *self == TcompIe::Trigger0CompleteEnabled
    }
    #[doc = "Trigger completion interrupts are enabled for trigger source 1 only."]
    #[inline(always)]
    pub fn is_trigger_1_complete_enabled(&self) -> bool {
        *self == TcompIe::Trigger1CompleteEnabled
    }
    #[doc = "Associated trigger completion interrupts are enabled."]
    #[inline(always)]
    pub fn is_trigger_x_complete_enabled_3(&self) -> bool {
        *self == TcompIe::TriggerXCompleteEnabled3
    }
    #[doc = "Associated trigger completion interrupts are enabled."]
    #[inline(always)]
    pub fn is_trigger_x_complete_enabled_4(&self) -> bool {
        *self == TcompIe::TriggerXCompleteEnabled4
    }
    #[doc = "Associated trigger completion interrupts are enabled."]
    #[inline(always)]
    pub fn is_trigger_x_complete_enabled_5(&self) -> bool {
        *self == TcompIe::TriggerXCompleteEnabled5
    }
    #[doc = "Associated trigger completion interrupts are enabled."]
    #[inline(always)]
    pub fn is_trigger_x_complete_enabled_6(&self) -> bool {
        *self == TcompIe::TriggerXCompleteEnabled6
    }
    #[doc = "Associated trigger completion interrupts are enabled."]
    #[inline(always)]
    pub fn is_trigger_x_complete_enabled_7(&self) -> bool {
        *self == TcompIe::TriggerXCompleteEnabled7
    }
    #[doc = "Associated trigger completion interrupts are enabled."]
    #[inline(always)]
    pub fn is_trigger_x_complete_enabled_8(&self) -> bool {
        *self == TcompIe::TriggerXCompleteEnabled8
    }
    #[doc = "Associated trigger completion interrupts are enabled."]
    #[inline(always)]
    pub fn is_trigger_x_complete_enabled_9(&self) -> bool {
        *self == TcompIe::TriggerXCompleteEnabled9
    }
    #[doc = "Trigger completion interrupts are enabled for every trigger source."]
    #[inline(always)]
    pub fn is_all_trigger_completes_enabled(&self) -> bool {
        *self == TcompIe::AllTriggerCompletesEnabled
    }
}
#[doc = "Field `TCOMP_IE` writer - Trigger Completion Interrupt Enable"]
pub type TcompIeW<'a, REG> = crate::FieldWriter<'a, REG, 4, TcompIe>;
impl<'a, REG> TcompIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Trigger completion interrupts are disabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(TcompIe::Disabled)
    }
    #[doc = "Trigger completion interrupts are enabled for trigger source 0 only."]
    #[inline(always)]
    pub fn trigger_0_complete_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(TcompIe::Trigger0CompleteEnabled)
    }
    #[doc = "Trigger completion interrupts are enabled for trigger source 1 only."]
    #[inline(always)]
    pub fn trigger_1_complete_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(TcompIe::Trigger1CompleteEnabled)
    }
    #[doc = "Associated trigger completion interrupts are enabled."]
    #[inline(always)]
    pub fn trigger_x_complete_enabled_3(self) -> &'a mut crate::W<REG> {
        self.variant(TcompIe::TriggerXCompleteEnabled3)
    }
    #[doc = "Associated trigger completion interrupts are enabled."]
    #[inline(always)]
    pub fn trigger_x_complete_enabled_4(self) -> &'a mut crate::W<REG> {
        self.variant(TcompIe::TriggerXCompleteEnabled4)
    }
    #[doc = "Associated trigger completion interrupts are enabled."]
    #[inline(always)]
    pub fn trigger_x_complete_enabled_5(self) -> &'a mut crate::W<REG> {
        self.variant(TcompIe::TriggerXCompleteEnabled5)
    }
    #[doc = "Associated trigger completion interrupts are enabled."]
    #[inline(always)]
    pub fn trigger_x_complete_enabled_6(self) -> &'a mut crate::W<REG> {
        self.variant(TcompIe::TriggerXCompleteEnabled6)
    }
    #[doc = "Associated trigger completion interrupts are enabled."]
    #[inline(always)]
    pub fn trigger_x_complete_enabled_7(self) -> &'a mut crate::W<REG> {
        self.variant(TcompIe::TriggerXCompleteEnabled7)
    }
    #[doc = "Associated trigger completion interrupts are enabled."]
    #[inline(always)]
    pub fn trigger_x_complete_enabled_8(self) -> &'a mut crate::W<REG> {
        self.variant(TcompIe::TriggerXCompleteEnabled8)
    }
    #[doc = "Associated trigger completion interrupts are enabled."]
    #[inline(always)]
    pub fn trigger_x_complete_enabled_9(self) -> &'a mut crate::W<REG> {
        self.variant(TcompIe::TriggerXCompleteEnabled9)
    }
    #[doc = "Trigger completion interrupts are enabled for every trigger source."]
    #[inline(always)]
    pub fn all_trigger_completes_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(TcompIe::AllTriggerCompletesEnabled)
    }
}
impl R {
    #[doc = "Bit 0 - FIFO 0 Watermark Interrupt Enable"]
    #[inline(always)]
    pub fn fwmie0(&self) -> Fwmie0R {
        Fwmie0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Result FIFO 0 Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn fofie0(&self) -> Fofie0R {
        Fofie0R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 8 - Trigger Exception Interrupt Enable"]
    #[inline(always)]
    pub fn texc_ie(&self) -> TexcIeR {
        TexcIeR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bits 16:19 - Trigger Completion Interrupt Enable"]
    #[inline(always)]
    pub fn tcomp_ie(&self) -> TcompIeR {
        TcompIeR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - FIFO 0 Watermark Interrupt Enable"]
    #[inline(always)]
    pub fn fwmie0(&mut self) -> Fwmie0W<IeSpec> {
        Fwmie0W::new(self, 0)
    }
    #[doc = "Bit 1 - Result FIFO 0 Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn fofie0(&mut self) -> Fofie0W<IeSpec> {
        Fofie0W::new(self, 1)
    }
    #[doc = "Bit 8 - Trigger Exception Interrupt Enable"]
    #[inline(always)]
    pub fn texc_ie(&mut self) -> TexcIeW<IeSpec> {
        TexcIeW::new(self, 8)
    }
    #[doc = "Bits 16:19 - Trigger Completion Interrupt Enable"]
    #[inline(always)]
    pub fn tcomp_ie(&mut self) -> TcompIeW<IeSpec> {
        TcompIeW::new(self, 16)
    }
}
#[doc = "Interrupt Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ie::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ie::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IeSpec;
impl crate::RegisterSpec for IeSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ie::R`](R) reader structure"]
impl crate::Readable for IeSpec {}
#[doc = "`write(|w| ..)` method takes [`ie::W`](W) writer structure"]
impl crate::Writable for IeSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets IE to value 0"]
impl crate::Resettable for IeSpec {}
