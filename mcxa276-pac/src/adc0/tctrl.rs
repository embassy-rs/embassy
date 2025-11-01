#[doc = "Register `TCTRL[%s]` reader"]
pub type R = crate::R<TctrlSpec>;
#[doc = "Register `TCTRL[%s]` writer"]
pub type W = crate::W<TctrlSpec>;
#[doc = "Trigger Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hten {
    #[doc = "0: Hardware trigger source disabled"]
    Disabled = 0,
    #[doc = "1: Hardware trigger source enabled"]
    Enabled = 1,
}
impl From<Hten> for bool {
    #[inline(always)]
    fn from(variant: Hten) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HTEN` reader - Trigger Enable"]
pub type HtenR = crate::BitReader<Hten>;
impl HtenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hten {
        match self.bits {
            false => Hten::Disabled,
            true => Hten::Enabled,
        }
    }
    #[doc = "Hardware trigger source disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Hten::Disabled
    }
    #[doc = "Hardware trigger source enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Hten::Enabled
    }
}
#[doc = "Field `HTEN` writer - Trigger Enable"]
pub type HtenW<'a, REG> = crate::BitWriter<'a, REG, Hten>;
impl<'a, REG> HtenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Hardware trigger source disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Hten::Disabled)
    }
    #[doc = "Hardware trigger source enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Hten::Enabled)
    }
}
#[doc = "Trigger Priority Setting\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Tpri {
    #[doc = "0: Set to highest priority, Level 1"]
    HighestPriority = 0,
    #[doc = "1: Set to corresponding priority level"]
    CorrespondingLowerPriority1 = 1,
    #[doc = "2: Set to corresponding priority level"]
    CorrespondingLowerPriority2 = 2,
    #[doc = "3: Set to lowest priority, Level 4"]
    LowestPriority = 3,
}
impl From<Tpri> for u8 {
    #[inline(always)]
    fn from(variant: Tpri) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Tpri {
    type Ux = u8;
}
impl crate::IsEnum for Tpri {}
#[doc = "Field `TPRI` reader - Trigger Priority Setting"]
pub type TpriR = crate::FieldReader<Tpri>;
impl TpriR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpri {
        match self.bits {
            0 => Tpri::HighestPriority,
            1 => Tpri::CorrespondingLowerPriority1,
            2 => Tpri::CorrespondingLowerPriority2,
            3 => Tpri::LowestPriority,
            _ => unreachable!(),
        }
    }
    #[doc = "Set to highest priority, Level 1"]
    #[inline(always)]
    pub fn is_highest_priority(&self) -> bool {
        *self == Tpri::HighestPriority
    }
    #[doc = "Set to corresponding priority level"]
    #[inline(always)]
    pub fn is_corresponding_lower_priority_1(&self) -> bool {
        *self == Tpri::CorrespondingLowerPriority1
    }
    #[doc = "Set to corresponding priority level"]
    #[inline(always)]
    pub fn is_corresponding_lower_priority_2(&self) -> bool {
        *self == Tpri::CorrespondingLowerPriority2
    }
    #[doc = "Set to lowest priority, Level 4"]
    #[inline(always)]
    pub fn is_lowest_priority(&self) -> bool {
        *self == Tpri::LowestPriority
    }
}
#[doc = "Field `TPRI` writer - Trigger Priority Setting"]
pub type TpriW<'a, REG> = crate::FieldWriter<'a, REG, 2, Tpri, crate::Safe>;
impl<'a, REG> TpriW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Set to highest priority, Level 1"]
    #[inline(always)]
    pub fn highest_priority(self) -> &'a mut crate::W<REG> {
        self.variant(Tpri::HighestPriority)
    }
    #[doc = "Set to corresponding priority level"]
    #[inline(always)]
    pub fn corresponding_lower_priority_1(self) -> &'a mut crate::W<REG> {
        self.variant(Tpri::CorrespondingLowerPriority1)
    }
    #[doc = "Set to corresponding priority level"]
    #[inline(always)]
    pub fn corresponding_lower_priority_2(self) -> &'a mut crate::W<REG> {
        self.variant(Tpri::CorrespondingLowerPriority2)
    }
    #[doc = "Set to lowest priority, Level 4"]
    #[inline(always)]
    pub fn lowest_priority(self) -> &'a mut crate::W<REG> {
        self.variant(Tpri::LowestPriority)
    }
}
#[doc = "Field `RSYNC` reader - Trigger Resync"]
pub type RsyncR = crate::BitReader;
#[doc = "Field `RSYNC` writer - Trigger Resync"]
pub type RsyncW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `TDLY` reader - Trigger Delay Select"]
pub type TdlyR = crate::FieldReader;
#[doc = "Field `TDLY` writer - Trigger Delay Select"]
pub type TdlyW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `TSYNC` reader - Trigger Synchronous Select"]
pub type TsyncR = crate::BitReader;
#[doc = "Field `TSYNC` writer - Trigger Synchronous Select"]
pub type TsyncW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Trigger Command Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Tcmd {
    #[doc = "0: Not a valid selection from the command buffer. Trigger event is ignored."]
    NotValid = 0,
    #[doc = "1: CMD1 is executed"]
    ExecuteCmd1 = 1,
    #[doc = "2: Corresponding CMD is executed"]
    ExecuteCorrespondingCmd2 = 2,
    #[doc = "3: Corresponding CMD is executed"]
    ExecuteCorrespondingCmd3 = 3,
    #[doc = "4: Corresponding CMD is executed"]
    ExecuteCorrespondingCmd4 = 4,
    #[doc = "5: Corresponding CMD is executed"]
    ExecuteCorrespondingCmd5 = 5,
    #[doc = "6: Corresponding CMD is executed"]
    ExecuteCorrespondingCmd6 = 6,
    #[doc = "7: CMD7 is executed"]
    ExecuteCmd7 = 7,
}
impl From<Tcmd> for u8 {
    #[inline(always)]
    fn from(variant: Tcmd) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Tcmd {
    type Ux = u8;
}
impl crate::IsEnum for Tcmd {}
#[doc = "Field `TCMD` reader - Trigger Command Select"]
pub type TcmdR = crate::FieldReader<Tcmd>;
impl TcmdR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tcmd {
        match self.bits {
            0 => Tcmd::NotValid,
            1 => Tcmd::ExecuteCmd1,
            2 => Tcmd::ExecuteCorrespondingCmd2,
            3 => Tcmd::ExecuteCorrespondingCmd3,
            4 => Tcmd::ExecuteCorrespondingCmd4,
            5 => Tcmd::ExecuteCorrespondingCmd5,
            6 => Tcmd::ExecuteCorrespondingCmd6,
            7 => Tcmd::ExecuteCmd7,
            _ => unreachable!(),
        }
    }
    #[doc = "Not a valid selection from the command buffer. Trigger event is ignored."]
    #[inline(always)]
    pub fn is_not_valid(&self) -> bool {
        *self == Tcmd::NotValid
    }
    #[doc = "CMD1 is executed"]
    #[inline(always)]
    pub fn is_execute_cmd1(&self) -> bool {
        *self == Tcmd::ExecuteCmd1
    }
    #[doc = "Corresponding CMD is executed"]
    #[inline(always)]
    pub fn is_execute_corresponding_cmd_2(&self) -> bool {
        *self == Tcmd::ExecuteCorrespondingCmd2
    }
    #[doc = "Corresponding CMD is executed"]
    #[inline(always)]
    pub fn is_execute_corresponding_cmd_3(&self) -> bool {
        *self == Tcmd::ExecuteCorrespondingCmd3
    }
    #[doc = "Corresponding CMD is executed"]
    #[inline(always)]
    pub fn is_execute_corresponding_cmd_4(&self) -> bool {
        *self == Tcmd::ExecuteCorrespondingCmd4
    }
    #[doc = "Corresponding CMD is executed"]
    #[inline(always)]
    pub fn is_execute_corresponding_cmd_5(&self) -> bool {
        *self == Tcmd::ExecuteCorrespondingCmd5
    }
    #[doc = "Corresponding CMD is executed"]
    #[inline(always)]
    pub fn is_execute_corresponding_cmd_6(&self) -> bool {
        *self == Tcmd::ExecuteCorrespondingCmd6
    }
    #[doc = "CMD7 is executed"]
    #[inline(always)]
    pub fn is_execute_cmd7(&self) -> bool {
        *self == Tcmd::ExecuteCmd7
    }
}
#[doc = "Field `TCMD` writer - Trigger Command Select"]
pub type TcmdW<'a, REG> = crate::FieldWriter<'a, REG, 3, Tcmd, crate::Safe>;
impl<'a, REG> TcmdW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Not a valid selection from the command buffer. Trigger event is ignored."]
    #[inline(always)]
    pub fn not_valid(self) -> &'a mut crate::W<REG> {
        self.variant(Tcmd::NotValid)
    }
    #[doc = "CMD1 is executed"]
    #[inline(always)]
    pub fn execute_cmd1(self) -> &'a mut crate::W<REG> {
        self.variant(Tcmd::ExecuteCmd1)
    }
    #[doc = "Corresponding CMD is executed"]
    #[inline(always)]
    pub fn execute_corresponding_cmd_2(self) -> &'a mut crate::W<REG> {
        self.variant(Tcmd::ExecuteCorrespondingCmd2)
    }
    #[doc = "Corresponding CMD is executed"]
    #[inline(always)]
    pub fn execute_corresponding_cmd_3(self) -> &'a mut crate::W<REG> {
        self.variant(Tcmd::ExecuteCorrespondingCmd3)
    }
    #[doc = "Corresponding CMD is executed"]
    #[inline(always)]
    pub fn execute_corresponding_cmd_4(self) -> &'a mut crate::W<REG> {
        self.variant(Tcmd::ExecuteCorrespondingCmd4)
    }
    #[doc = "Corresponding CMD is executed"]
    #[inline(always)]
    pub fn execute_corresponding_cmd_5(self) -> &'a mut crate::W<REG> {
        self.variant(Tcmd::ExecuteCorrespondingCmd5)
    }
    #[doc = "Corresponding CMD is executed"]
    #[inline(always)]
    pub fn execute_corresponding_cmd_6(self) -> &'a mut crate::W<REG> {
        self.variant(Tcmd::ExecuteCorrespondingCmd6)
    }
    #[doc = "CMD7 is executed"]
    #[inline(always)]
    pub fn execute_cmd7(self) -> &'a mut crate::W<REG> {
        self.variant(Tcmd::ExecuteCmd7)
    }
}
impl R {
    #[doc = "Bit 0 - Trigger Enable"]
    #[inline(always)]
    pub fn hten(&self) -> HtenR {
        HtenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bits 8:9 - Trigger Priority Setting"]
    #[inline(always)]
    pub fn tpri(&self) -> TpriR {
        TpriR::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bit 15 - Trigger Resync"]
    #[inline(always)]
    pub fn rsync(&self) -> RsyncR {
        RsyncR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bits 16:19 - Trigger Delay Select"]
    #[inline(always)]
    pub fn tdly(&self) -> TdlyR {
        TdlyR::new(((self.bits >> 16) & 0x0f) as u8)
    }
    #[doc = "Bit 23 - Trigger Synchronous Select"]
    #[inline(always)]
    pub fn tsync(&self) -> TsyncR {
        TsyncR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bits 24:26 - Trigger Command Select"]
    #[inline(always)]
    pub fn tcmd(&self) -> TcmdR {
        TcmdR::new(((self.bits >> 24) & 7) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Trigger Enable"]
    #[inline(always)]
    pub fn hten(&mut self) -> HtenW<TctrlSpec> {
        HtenW::new(self, 0)
    }
    #[doc = "Bits 8:9 - Trigger Priority Setting"]
    #[inline(always)]
    pub fn tpri(&mut self) -> TpriW<TctrlSpec> {
        TpriW::new(self, 8)
    }
    #[doc = "Bit 15 - Trigger Resync"]
    #[inline(always)]
    pub fn rsync(&mut self) -> RsyncW<TctrlSpec> {
        RsyncW::new(self, 15)
    }
    #[doc = "Bits 16:19 - Trigger Delay Select"]
    #[inline(always)]
    pub fn tdly(&mut self) -> TdlyW<TctrlSpec> {
        TdlyW::new(self, 16)
    }
    #[doc = "Bit 23 - Trigger Synchronous Select"]
    #[inline(always)]
    pub fn tsync(&mut self) -> TsyncW<TctrlSpec> {
        TsyncW::new(self, 23)
    }
    #[doc = "Bits 24:26 - Trigger Command Select"]
    #[inline(always)]
    pub fn tcmd(&mut self) -> TcmdW<TctrlSpec> {
        TcmdW::new(self, 24)
    }
}
#[doc = "Trigger Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`tctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TctrlSpec;
impl crate::RegisterSpec for TctrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`tctrl::R`](R) reader structure"]
impl crate::Readable for TctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`tctrl::W`](W) writer structure"]
impl crate::Writable for TctrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TCTRL[%s] to value 0"]
impl crate::Resettable for TctrlSpec {}
