#[doc = "Register `FLAGS` reader"]
pub type R = crate::R<FlagsSpec>;
#[doc = "Register `FLAGS` writer"]
pub type W = crate::W<FlagsSpec>;
#[doc = "TIMEOUT fault flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToFlag {
    #[doc = "0: A TIMEOUT fault has not occurred"]
    NoFlag = 0,
    #[doc = "1: A TIMEOUT fault has occurred"]
    Flag = 1,
}
impl From<ToFlag> for bool {
    #[inline(always)]
    fn from(variant: ToFlag) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TO_FLAG` reader - TIMEOUT fault flag"]
pub type ToFlagR = crate::BitReader<ToFlag>;
impl ToFlagR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> ToFlag {
        match self.bits {
            false => ToFlag::NoFlag,
            true => ToFlag::Flag,
        }
    }
    #[doc = "A TIMEOUT fault has not occurred"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == ToFlag::NoFlag
    }
    #[doc = "A TIMEOUT fault has occurred"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == ToFlag::Flag
    }
}
#[doc = "Field `TO_FLAG` writer - TIMEOUT fault flag"]
pub type ToFlagW<'a, REG> = crate::BitWriter1C<'a, REG, ToFlag>;
impl<'a, REG> ToFlagW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "A TIMEOUT fault has not occurred"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(ToFlag::NoFlag)
    }
    #[doc = "A TIMEOUT fault has occurred"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(ToFlag::Flag)
    }
}
#[doc = "MISCOMPARE fault flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MiscomFlag {
    #[doc = "0: A MISCOMPARE fault has not occurred"]
    NoFlag = 0,
    #[doc = "1: A MISCOMPARE fault has occurred"]
    Flag = 1,
}
impl From<MiscomFlag> for bool {
    #[inline(always)]
    fn from(variant: MiscomFlag) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MISCOM_FLAG` reader - MISCOMPARE fault flag"]
pub type MiscomFlagR = crate::BitReader<MiscomFlag>;
impl MiscomFlagR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> MiscomFlag {
        match self.bits {
            false => MiscomFlag::NoFlag,
            true => MiscomFlag::Flag,
        }
    }
    #[doc = "A MISCOMPARE fault has not occurred"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == MiscomFlag::NoFlag
    }
    #[doc = "A MISCOMPARE fault has occurred"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == MiscomFlag::Flag
    }
}
#[doc = "Field `MISCOM_FLAG` writer - MISCOMPARE fault flag"]
pub type MiscomFlagW<'a, REG> = crate::BitWriter1C<'a, REG, MiscomFlag>;
impl<'a, REG> MiscomFlagW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "A MISCOMPARE fault has not occurred"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(MiscomFlag::NoFlag)
    }
    #[doc = "A MISCOMPARE fault has occurred"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(MiscomFlag::Flag)
    }
}
#[doc = "SEQUENCE fault flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeqFlag {
    #[doc = "0: A SEQUENCE fault has not occurred"]
    NoFlag = 0,
    #[doc = "1: A SEQUENCE fault has occurred"]
    Flag = 1,
}
impl From<SeqFlag> for bool {
    #[inline(always)]
    fn from(variant: SeqFlag) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SEQ_FLAG` reader - SEQUENCE fault flag"]
pub type SeqFlagR = crate::BitReader<SeqFlag>;
impl SeqFlagR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SeqFlag {
        match self.bits {
            false => SeqFlag::NoFlag,
            true => SeqFlag::Flag,
        }
    }
    #[doc = "A SEQUENCE fault has not occurred"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == SeqFlag::NoFlag
    }
    #[doc = "A SEQUENCE fault has occurred"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == SeqFlag::Flag
    }
}
#[doc = "Field `SEQ_FLAG` writer - SEQUENCE fault flag"]
pub type SeqFlagW<'a, REG> = crate::BitWriter1C<'a, REG, SeqFlag>;
impl<'a, REG> SeqFlagW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "A SEQUENCE fault has not occurred"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(SeqFlag::NoFlag)
    }
    #[doc = "A SEQUENCE fault has occurred"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(SeqFlag::Flag)
    }
}
#[doc = "CONTROL fault flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CntFlag {
    #[doc = "0: A CONTROL fault has not occurred"]
    NoFlag = 0,
    #[doc = "1: A CONTROL fault has occurred"]
    Flag = 1,
}
impl From<CntFlag> for bool {
    #[inline(always)]
    fn from(variant: CntFlag) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CNT_FLAG` reader - CONTROL fault flag"]
pub type CntFlagR = crate::BitReader<CntFlag>;
impl CntFlagR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CntFlag {
        match self.bits {
            false => CntFlag::NoFlag,
            true => CntFlag::Flag,
        }
    }
    #[doc = "A CONTROL fault has not occurred"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == CntFlag::NoFlag
    }
    #[doc = "A CONTROL fault has occurred"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == CntFlag::Flag
    }
}
#[doc = "Field `CNT_FLAG` writer - CONTROL fault flag"]
pub type CntFlagW<'a, REG> = crate::BitWriter1C<'a, REG, CntFlag>;
impl<'a, REG> CntFlagW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "A CONTROL fault has not occurred"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(CntFlag::NoFlag)
    }
    #[doc = "A CONTROL fault has occurred"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(CntFlag::Flag)
    }
}
#[doc = "STATE fault flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateFlag {
    #[doc = "0: A STATE fault has not occurred"]
    NoFlag = 0,
    #[doc = "1: A STATE fault has occurred"]
    Flag = 1,
}
impl From<StateFlag> for bool {
    #[inline(always)]
    fn from(variant: StateFlag) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STATE_FLAG` reader - STATE fault flag"]
pub type StateFlagR = crate::BitReader<StateFlag>;
impl StateFlagR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> StateFlag {
        match self.bits {
            false => StateFlag::NoFlag,
            true => StateFlag::Flag,
        }
    }
    #[doc = "A STATE fault has not occurred"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == StateFlag::NoFlag
    }
    #[doc = "A STATE fault has occurred"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == StateFlag::Flag
    }
}
#[doc = "Field `STATE_FLAG` writer - STATE fault flag"]
pub type StateFlagW<'a, REG> = crate::BitWriter1C<'a, REG, StateFlag>;
impl<'a, REG> StateFlagW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "A STATE fault has not occurred"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(StateFlag::NoFlag)
    }
    #[doc = "A STATE fault has occurred"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(StateFlag::Flag)
    }
}
#[doc = "ADDRESS fault flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddrFlag {
    #[doc = "0: An ADDRESS fault has not occurred"]
    NoFlag = 0,
    #[doc = "1: An ADDRESS fault has occurred"]
    Flag = 1,
}
impl From<AddrFlag> for bool {
    #[inline(always)]
    fn from(variant: AddrFlag) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ADDR_FLAG` reader - ADDRESS fault flag"]
pub type AddrFlagR = crate::BitReader<AddrFlag>;
impl AddrFlagR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> AddrFlag {
        match self.bits {
            false => AddrFlag::NoFlag,
            true => AddrFlag::Flag,
        }
    }
    #[doc = "An ADDRESS fault has not occurred"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == AddrFlag::NoFlag
    }
    #[doc = "An ADDRESS fault has occurred"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == AddrFlag::Flag
    }
}
#[doc = "Field `ADDR_FLAG` writer - ADDRESS fault flag"]
pub type AddrFlagW<'a, REG> = crate::BitWriter1C<'a, REG, AddrFlag>;
impl<'a, REG> AddrFlagW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "An ADDRESS fault has not occurred"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(AddrFlag::NoFlag)
    }
    #[doc = "An ADDRESS fault has occurred"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(AddrFlag::Flag)
    }
}
#[doc = "Power-on reset flag\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PorFlag {
    #[doc = "0: A Power-on reset event has not occurred"]
    NoFlag = 0,
    #[doc = "1: A Power-on reset event has occurred"]
    Flag = 1,
}
impl From<PorFlag> for bool {
    #[inline(always)]
    fn from(variant: PorFlag) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `POR_FLAG` reader - Power-on reset flag"]
pub type PorFlagR = crate::BitReader<PorFlag>;
impl PorFlagR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> PorFlag {
        match self.bits {
            false => PorFlag::NoFlag,
            true => PorFlag::Flag,
        }
    }
    #[doc = "A Power-on reset event has not occurred"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == PorFlag::NoFlag
    }
    #[doc = "A Power-on reset event has occurred"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == PorFlag::Flag
    }
}
#[doc = "Field `POR_FLAG` writer - Power-on reset flag"]
pub type PorFlagW<'a, REG> = crate::BitWriter1C<'a, REG, PorFlag>;
impl<'a, REG> PorFlagW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "A Power-on reset event has not occurred"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(PorFlag::NoFlag)
    }
    #[doc = "A Power-on reset event has occurred"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(PorFlag::Flag)
    }
}
impl R {
    #[doc = "Bit 0 - TIMEOUT fault flag"]
    #[inline(always)]
    pub fn to_flag(&self) -> ToFlagR {
        ToFlagR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - MISCOMPARE fault flag"]
    #[inline(always)]
    pub fn miscom_flag(&self) -> MiscomFlagR {
        MiscomFlagR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - SEQUENCE fault flag"]
    #[inline(always)]
    pub fn seq_flag(&self) -> SeqFlagR {
        SeqFlagR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - CONTROL fault flag"]
    #[inline(always)]
    pub fn cnt_flag(&self) -> CntFlagR {
        CntFlagR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - STATE fault flag"]
    #[inline(always)]
    pub fn state_flag(&self) -> StateFlagR {
        StateFlagR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - ADDRESS fault flag"]
    #[inline(always)]
    pub fn addr_flag(&self) -> AddrFlagR {
        AddrFlagR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 16 - Power-on reset flag"]
    #[inline(always)]
    pub fn por_flag(&self) -> PorFlagR {
        PorFlagR::new(((self.bits >> 16) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - TIMEOUT fault flag"]
    #[inline(always)]
    pub fn to_flag(&mut self) -> ToFlagW<FlagsSpec> {
        ToFlagW::new(self, 0)
    }
    #[doc = "Bit 1 - MISCOMPARE fault flag"]
    #[inline(always)]
    pub fn miscom_flag(&mut self) -> MiscomFlagW<FlagsSpec> {
        MiscomFlagW::new(self, 1)
    }
    #[doc = "Bit 2 - SEQUENCE fault flag"]
    #[inline(always)]
    pub fn seq_flag(&mut self) -> SeqFlagW<FlagsSpec> {
        SeqFlagW::new(self, 2)
    }
    #[doc = "Bit 3 - CONTROL fault flag"]
    #[inline(always)]
    pub fn cnt_flag(&mut self) -> CntFlagW<FlagsSpec> {
        CntFlagW::new(self, 3)
    }
    #[doc = "Bit 4 - STATE fault flag"]
    #[inline(always)]
    pub fn state_flag(&mut self) -> StateFlagW<FlagsSpec> {
        StateFlagW::new(self, 4)
    }
    #[doc = "Bit 5 - ADDRESS fault flag"]
    #[inline(always)]
    pub fn addr_flag(&mut self) -> AddrFlagW<FlagsSpec> {
        AddrFlagW::new(self, 5)
    }
    #[doc = "Bit 16 - Power-on reset flag"]
    #[inline(always)]
    pub fn por_flag(&mut self) -> PorFlagW<FlagsSpec> {
        PorFlagW::new(self, 16)
    }
}
#[doc = "Flags Register\n\nYou can [`read`](crate::Reg::read) this register and get [`flags::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flags::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FlagsSpec;
impl crate::RegisterSpec for FlagsSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`flags::R`](R) reader structure"]
impl crate::Readable for FlagsSpec {}
#[doc = "`write(|w| ..)` method takes [`flags::W`](W) writer structure"]
impl crate::Writable for FlagsSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0001_003f;
}
#[doc = "`reset()` method sets FLAGS to value 0x0001_0000"]
impl crate::Resettable for FlagsSpec {
    const RESET_VALUE: u32 = 0x0001_0000;
}
