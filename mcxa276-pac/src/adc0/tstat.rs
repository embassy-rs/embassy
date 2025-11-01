#[doc = "Register `TSTAT` reader"]
pub type R = crate::R<TstatSpec>;
#[doc = "Register `TSTAT` writer"]
pub type W = crate::W<TstatSpec>;
#[doc = "Trigger Exception Number\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TexcNum {
    #[doc = "0: No triggers have been interrupted by a high priority exception. Or CFG\\[TRES\\] = 1."]
    NoExceptions = 0,
    #[doc = "1: Trigger 0 has been interrupted by a high priority exception."]
    Bit0MeansTrigger0Interrupted = 1,
    #[doc = "2: Trigger 1 has been interrupted by a high priority exception."]
    Bit1MeansTrigger1Interrupted = 2,
    #[doc = "3: Associated trigger sequence has interrupted by a high priority exception."]
    SetBitsIndicateTriggerXInterrupted3 = 3,
    #[doc = "4: Associated trigger sequence has interrupted by a high priority exception."]
    SetBitsIndicateTriggerXInterrupted4 = 4,
    #[doc = "5: Associated trigger sequence has interrupted by a high priority exception."]
    SetBitsIndicateTriggerXInterrupted5 = 5,
    #[doc = "6: Associated trigger sequence has interrupted by a high priority exception."]
    SetBitsIndicateTriggerXInterrupted6 = 6,
    #[doc = "7: Associated trigger sequence has interrupted by a high priority exception."]
    SetBitsIndicateTriggerXInterrupted7 = 7,
    #[doc = "8: Associated trigger sequence has interrupted by a high priority exception."]
    SetBitsIndicateTriggerXInterrupted8 = 8,
    #[doc = "9: Associated trigger sequence has interrupted by a high priority exception."]
    SetBitsIndicateTriggerXInterrupted9 = 9,
    #[doc = "15: Every trigger sequence has been interrupted by a high priority exception."]
    AllBitsSetIndicateAllTriggersInterrupted = 15,
}
impl From<TexcNum> for u8 {
    #[inline(always)]
    fn from(variant: TexcNum) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for TexcNum {
    type Ux = u8;
}
impl crate::IsEnum for TexcNum {}
#[doc = "Field `TEXC_NUM` reader - Trigger Exception Number"]
pub type TexcNumR = crate::FieldReader<TexcNum>;
impl TexcNumR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<TexcNum> {
        match self.bits {
            0 => Some(TexcNum::NoExceptions),
            1 => Some(TexcNum::Bit0MeansTrigger0Interrupted),
            2 => Some(TexcNum::Bit1MeansTrigger1Interrupted),
            3 => Some(TexcNum::SetBitsIndicateTriggerXInterrupted3),
            4 => Some(TexcNum::SetBitsIndicateTriggerXInterrupted4),
            5 => Some(TexcNum::SetBitsIndicateTriggerXInterrupted5),
            6 => Some(TexcNum::SetBitsIndicateTriggerXInterrupted6),
            7 => Some(TexcNum::SetBitsIndicateTriggerXInterrupted7),
            8 => Some(TexcNum::SetBitsIndicateTriggerXInterrupted8),
            9 => Some(TexcNum::SetBitsIndicateTriggerXInterrupted9),
            15 => Some(TexcNum::AllBitsSetIndicateAllTriggersInterrupted),
            _ => None,
        }
    }
    #[doc = "No triggers have been interrupted by a high priority exception. Or CFG\\[TRES\\] = 1."]
    #[inline(always)]
    pub fn is_no_exceptions(&self) -> bool {
        *self == TexcNum::NoExceptions
    }
    #[doc = "Trigger 0 has been interrupted by a high priority exception."]
    #[inline(always)]
    pub fn is_bit0_means_trigger_0_interrupted(&self) -> bool {
        *self == TexcNum::Bit0MeansTrigger0Interrupted
    }
    #[doc = "Trigger 1 has been interrupted by a high priority exception."]
    #[inline(always)]
    pub fn is_bit1_means_trigger_1_interrupted(&self) -> bool {
        *self == TexcNum::Bit1MeansTrigger1Interrupted
    }
    #[doc = "Associated trigger sequence has interrupted by a high priority exception."]
    #[inline(always)]
    pub fn is_set_bits_indicate_trigger_x_interrupted_3(&self) -> bool {
        *self == TexcNum::SetBitsIndicateTriggerXInterrupted3
    }
    #[doc = "Associated trigger sequence has interrupted by a high priority exception."]
    #[inline(always)]
    pub fn is_set_bits_indicate_trigger_x_interrupted_4(&self) -> bool {
        *self == TexcNum::SetBitsIndicateTriggerXInterrupted4
    }
    #[doc = "Associated trigger sequence has interrupted by a high priority exception."]
    #[inline(always)]
    pub fn is_set_bits_indicate_trigger_x_interrupted_5(&self) -> bool {
        *self == TexcNum::SetBitsIndicateTriggerXInterrupted5
    }
    #[doc = "Associated trigger sequence has interrupted by a high priority exception."]
    #[inline(always)]
    pub fn is_set_bits_indicate_trigger_x_interrupted_6(&self) -> bool {
        *self == TexcNum::SetBitsIndicateTriggerXInterrupted6
    }
    #[doc = "Associated trigger sequence has interrupted by a high priority exception."]
    #[inline(always)]
    pub fn is_set_bits_indicate_trigger_x_interrupted_7(&self) -> bool {
        *self == TexcNum::SetBitsIndicateTriggerXInterrupted7
    }
    #[doc = "Associated trigger sequence has interrupted by a high priority exception."]
    #[inline(always)]
    pub fn is_set_bits_indicate_trigger_x_interrupted_8(&self) -> bool {
        *self == TexcNum::SetBitsIndicateTriggerXInterrupted8
    }
    #[doc = "Associated trigger sequence has interrupted by a high priority exception."]
    #[inline(always)]
    pub fn is_set_bits_indicate_trigger_x_interrupted_9(&self) -> bool {
        *self == TexcNum::SetBitsIndicateTriggerXInterrupted9
    }
    #[doc = "Every trigger sequence has been interrupted by a high priority exception."]
    #[inline(always)]
    pub fn is_all_bits_set_indicate_all_triggers_interrupted(&self) -> bool {
        *self == TexcNum::AllBitsSetIndicateAllTriggersInterrupted
    }
}
#[doc = "Field `TEXC_NUM` writer - Trigger Exception Number"]
pub type TexcNumW<'a, REG> = crate::FieldWriter<'a, REG, 4, TexcNum>;
impl<'a, REG> TexcNumW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "No triggers have been interrupted by a high priority exception. Or CFG\\[TRES\\] = 1."]
    #[inline(always)]
    pub fn no_exceptions(self) -> &'a mut crate::W<REG> {
        self.variant(TexcNum::NoExceptions)
    }
    #[doc = "Trigger 0 has been interrupted by a high priority exception."]
    #[inline(always)]
    pub fn bit0_means_trigger_0_interrupted(self) -> &'a mut crate::W<REG> {
        self.variant(TexcNum::Bit0MeansTrigger0Interrupted)
    }
    #[doc = "Trigger 1 has been interrupted by a high priority exception."]
    #[inline(always)]
    pub fn bit1_means_trigger_1_interrupted(self) -> &'a mut crate::W<REG> {
        self.variant(TexcNum::Bit1MeansTrigger1Interrupted)
    }
    #[doc = "Associated trigger sequence has interrupted by a high priority exception."]
    #[inline(always)]
    pub fn set_bits_indicate_trigger_x_interrupted_3(self) -> &'a mut crate::W<REG> {
        self.variant(TexcNum::SetBitsIndicateTriggerXInterrupted3)
    }
    #[doc = "Associated trigger sequence has interrupted by a high priority exception."]
    #[inline(always)]
    pub fn set_bits_indicate_trigger_x_interrupted_4(self) -> &'a mut crate::W<REG> {
        self.variant(TexcNum::SetBitsIndicateTriggerXInterrupted4)
    }
    #[doc = "Associated trigger sequence has interrupted by a high priority exception."]
    #[inline(always)]
    pub fn set_bits_indicate_trigger_x_interrupted_5(self) -> &'a mut crate::W<REG> {
        self.variant(TexcNum::SetBitsIndicateTriggerXInterrupted5)
    }
    #[doc = "Associated trigger sequence has interrupted by a high priority exception."]
    #[inline(always)]
    pub fn set_bits_indicate_trigger_x_interrupted_6(self) -> &'a mut crate::W<REG> {
        self.variant(TexcNum::SetBitsIndicateTriggerXInterrupted6)
    }
    #[doc = "Associated trigger sequence has interrupted by a high priority exception."]
    #[inline(always)]
    pub fn set_bits_indicate_trigger_x_interrupted_7(self) -> &'a mut crate::W<REG> {
        self.variant(TexcNum::SetBitsIndicateTriggerXInterrupted7)
    }
    #[doc = "Associated trigger sequence has interrupted by a high priority exception."]
    #[inline(always)]
    pub fn set_bits_indicate_trigger_x_interrupted_8(self) -> &'a mut crate::W<REG> {
        self.variant(TexcNum::SetBitsIndicateTriggerXInterrupted8)
    }
    #[doc = "Associated trigger sequence has interrupted by a high priority exception."]
    #[inline(always)]
    pub fn set_bits_indicate_trigger_x_interrupted_9(self) -> &'a mut crate::W<REG> {
        self.variant(TexcNum::SetBitsIndicateTriggerXInterrupted9)
    }
    #[doc = "Every trigger sequence has been interrupted by a high priority exception."]
    #[inline(always)]
    pub fn all_bits_set_indicate_all_triggers_interrupted(self) -> &'a mut crate::W<REG> {
        self.variant(TexcNum::AllBitsSetIndicateAllTriggersInterrupted)
    }
}
#[doc = "Trigger Completion Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum TcompFlag {
    #[doc = "0: No triggers have been completed. Trigger completion interrupts are disabled."]
    NoTrigger = 0,
    #[doc = "1: Trigger 0 has been completed and trigger 0 has enabled completion interrupts."]
    Bit0MeansTrigger0Completed = 1,
    #[doc = "2: Trigger 1 has been completed and trigger 1 has enabled completion interrupts."]
    Bit1MeansTrigger1Completed = 2,
    #[doc = "3: Associated trigger sequence has completed and has enabled completion interrupts."]
    SetBitsIndicateTriggerXCompleted3 = 3,
    #[doc = "4: Associated trigger sequence has completed and has enabled completion interrupts."]
    SetBitsIndicateTriggerXCompleted4 = 4,
    #[doc = "5: Associated trigger sequence has completed and has enabled completion interrupts."]
    SetBitsIndicateTriggerXCompleted5 = 5,
    #[doc = "6: Associated trigger sequence has completed and has enabled completion interrupts."]
    SetBitsIndicateTriggerXCompleted6 = 6,
    #[doc = "7: Associated trigger sequence has completed and has enabled completion interrupts."]
    SetBitsIndicateTriggerXCompleted7 = 7,
    #[doc = "8: Associated trigger sequence has completed and has enabled completion interrupts."]
    SetBitsIndicateTriggerXCompleted8 = 8,
    #[doc = "9: Associated trigger sequence has completed and has enabled completion interrupts."]
    SetBitsIndicateTriggerXCompleted9 = 9,
    #[doc = "15: Every trigger sequence has been completed and every trigger has enabled completion interrupts."]
    AllBitsSetIndicateAllTriggersCompleted = 15,
}
impl From<TcompFlag> for u8 {
    #[inline(always)]
    fn from(variant: TcompFlag) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for TcompFlag {
    type Ux = u8;
}
impl crate::IsEnum for TcompFlag {}
#[doc = "Field `TCOMP_FLAG` reader - Trigger Completion Flag"]
pub type TcompFlagR = crate::FieldReader<TcompFlag>;
impl TcompFlagR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<TcompFlag> {
        match self.bits {
            0 => Some(TcompFlag::NoTrigger),
            1 => Some(TcompFlag::Bit0MeansTrigger0Completed),
            2 => Some(TcompFlag::Bit1MeansTrigger1Completed),
            3 => Some(TcompFlag::SetBitsIndicateTriggerXCompleted3),
            4 => Some(TcompFlag::SetBitsIndicateTriggerXCompleted4),
            5 => Some(TcompFlag::SetBitsIndicateTriggerXCompleted5),
            6 => Some(TcompFlag::SetBitsIndicateTriggerXCompleted6),
            7 => Some(TcompFlag::SetBitsIndicateTriggerXCompleted7),
            8 => Some(TcompFlag::SetBitsIndicateTriggerXCompleted8),
            9 => Some(TcompFlag::SetBitsIndicateTriggerXCompleted9),
            15 => Some(TcompFlag::AllBitsSetIndicateAllTriggersCompleted),
            _ => None,
        }
    }
    #[doc = "No triggers have been completed. Trigger completion interrupts are disabled."]
    #[inline(always)]
    pub fn is_no_trigger(&self) -> bool {
        *self == TcompFlag::NoTrigger
    }
    #[doc = "Trigger 0 has been completed and trigger 0 has enabled completion interrupts."]
    #[inline(always)]
    pub fn is_bit0_means_trigger_0_completed(&self) -> bool {
        *self == TcompFlag::Bit0MeansTrigger0Completed
    }
    #[doc = "Trigger 1 has been completed and trigger 1 has enabled completion interrupts."]
    #[inline(always)]
    pub fn is_bit1_means_trigger_1_completed(&self) -> bool {
        *self == TcompFlag::Bit1MeansTrigger1Completed
    }
    #[doc = "Associated trigger sequence has completed and has enabled completion interrupts."]
    #[inline(always)]
    pub fn is_set_bits_indicate_trigger_x_completed_3(&self) -> bool {
        *self == TcompFlag::SetBitsIndicateTriggerXCompleted3
    }
    #[doc = "Associated trigger sequence has completed and has enabled completion interrupts."]
    #[inline(always)]
    pub fn is_set_bits_indicate_trigger_x_completed_4(&self) -> bool {
        *self == TcompFlag::SetBitsIndicateTriggerXCompleted4
    }
    #[doc = "Associated trigger sequence has completed and has enabled completion interrupts."]
    #[inline(always)]
    pub fn is_set_bits_indicate_trigger_x_completed_5(&self) -> bool {
        *self == TcompFlag::SetBitsIndicateTriggerXCompleted5
    }
    #[doc = "Associated trigger sequence has completed and has enabled completion interrupts."]
    #[inline(always)]
    pub fn is_set_bits_indicate_trigger_x_completed_6(&self) -> bool {
        *self == TcompFlag::SetBitsIndicateTriggerXCompleted6
    }
    #[doc = "Associated trigger sequence has completed and has enabled completion interrupts."]
    #[inline(always)]
    pub fn is_set_bits_indicate_trigger_x_completed_7(&self) -> bool {
        *self == TcompFlag::SetBitsIndicateTriggerXCompleted7
    }
    #[doc = "Associated trigger sequence has completed and has enabled completion interrupts."]
    #[inline(always)]
    pub fn is_set_bits_indicate_trigger_x_completed_8(&self) -> bool {
        *self == TcompFlag::SetBitsIndicateTriggerXCompleted8
    }
    #[doc = "Associated trigger sequence has completed and has enabled completion interrupts."]
    #[inline(always)]
    pub fn is_set_bits_indicate_trigger_x_completed_9(&self) -> bool {
        *self == TcompFlag::SetBitsIndicateTriggerXCompleted9
    }
    #[doc = "Every trigger sequence has been completed and every trigger has enabled completion interrupts."]
    #[inline(always)]
    pub fn is_all_bits_set_indicate_all_triggers_completed(&self) -> bool {
        *self == TcompFlag::AllBitsSetIndicateAllTriggersCompleted
    }
}
#[doc = "Field `TCOMP_FLAG` writer - Trigger Completion Flag"]
pub type TcompFlagW<'a, REG> = crate::FieldWriter<'a, REG, 4, TcompFlag>;
impl<'a, REG> TcompFlagW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "No triggers have been completed. Trigger completion interrupts are disabled."]
    #[inline(always)]
    pub fn no_trigger(self) -> &'a mut crate::W<REG> {
        self.variant(TcompFlag::NoTrigger)
    }
    #[doc = "Trigger 0 has been completed and trigger 0 has enabled completion interrupts."]
    #[inline(always)]
    pub fn bit0_means_trigger_0_completed(self) -> &'a mut crate::W<REG> {
        self.variant(TcompFlag::Bit0MeansTrigger0Completed)
    }
    #[doc = "Trigger 1 has been completed and trigger 1 has enabled completion interrupts."]
    #[inline(always)]
    pub fn bit1_means_trigger_1_completed(self) -> &'a mut crate::W<REG> {
        self.variant(TcompFlag::Bit1MeansTrigger1Completed)
    }
    #[doc = "Associated trigger sequence has completed and has enabled completion interrupts."]
    #[inline(always)]
    pub fn set_bits_indicate_trigger_x_completed_3(self) -> &'a mut crate::W<REG> {
        self.variant(TcompFlag::SetBitsIndicateTriggerXCompleted3)
    }
    #[doc = "Associated trigger sequence has completed and has enabled completion interrupts."]
    #[inline(always)]
    pub fn set_bits_indicate_trigger_x_completed_4(self) -> &'a mut crate::W<REG> {
        self.variant(TcompFlag::SetBitsIndicateTriggerXCompleted4)
    }
    #[doc = "Associated trigger sequence has completed and has enabled completion interrupts."]
    #[inline(always)]
    pub fn set_bits_indicate_trigger_x_completed_5(self) -> &'a mut crate::W<REG> {
        self.variant(TcompFlag::SetBitsIndicateTriggerXCompleted5)
    }
    #[doc = "Associated trigger sequence has completed and has enabled completion interrupts."]
    #[inline(always)]
    pub fn set_bits_indicate_trigger_x_completed_6(self) -> &'a mut crate::W<REG> {
        self.variant(TcompFlag::SetBitsIndicateTriggerXCompleted6)
    }
    #[doc = "Associated trigger sequence has completed and has enabled completion interrupts."]
    #[inline(always)]
    pub fn set_bits_indicate_trigger_x_completed_7(self) -> &'a mut crate::W<REG> {
        self.variant(TcompFlag::SetBitsIndicateTriggerXCompleted7)
    }
    #[doc = "Associated trigger sequence has completed and has enabled completion interrupts."]
    #[inline(always)]
    pub fn set_bits_indicate_trigger_x_completed_8(self) -> &'a mut crate::W<REG> {
        self.variant(TcompFlag::SetBitsIndicateTriggerXCompleted8)
    }
    #[doc = "Associated trigger sequence has completed and has enabled completion interrupts."]
    #[inline(always)]
    pub fn set_bits_indicate_trigger_x_completed_9(self) -> &'a mut crate::W<REG> {
        self.variant(TcompFlag::SetBitsIndicateTriggerXCompleted9)
    }
    #[doc = "Every trigger sequence has been completed and every trigger has enabled completion interrupts."]
    #[inline(always)]
    pub fn all_bits_set_indicate_all_triggers_completed(self) -> &'a mut crate::W<REG> {
        self.variant(TcompFlag::AllBitsSetIndicateAllTriggersCompleted)
    }
}
impl R {
    #[doc = "Bits 0:3 - Trigger Exception Number"]
    #[inline(always)]
    pub fn texc_num(&self) -> TexcNumR {
        TexcNumR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 16:19 - Trigger Completion Flag"]
    #[inline(always)]
    pub fn tcomp_flag(&self) -> TcompFlagR {
        TcompFlagR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Trigger Exception Number"]
    #[inline(always)]
    pub fn texc_num(&mut self) -> TexcNumW<TstatSpec> {
        TexcNumW::new(self, 0)
    }
    #[doc = "Bits 16:19 - Trigger Completion Flag"]
    #[inline(always)]
    pub fn tcomp_flag(&mut self) -> TcompFlagW<TstatSpec> {
        TcompFlagW::new(self, 16)
    }
}
#[doc = "Trigger Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`tstat::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tstat::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TstatSpec;
impl crate::RegisterSpec for TstatSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`tstat::R`](R) reader structure"]
impl crate::Readable for TstatSpec {}
#[doc = "`write(|w| ..)` method takes [`tstat::W`](W) writer structure"]
impl crate::Writable for TstatSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x000f_000f;
}
#[doc = "`reset()` method sets TSTAT to value 0"]
impl crate::Resettable for TstatSpec {}
