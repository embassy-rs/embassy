#[doc = "Register `RRCR0` reader"]
pub type R = crate::R<Rrcr0Spec>;
#[doc = "Register `RRCR0` writer"]
pub type W = crate::W<Rrcr0Spec>;
#[doc = "Round-Robin Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrEn {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<RrEn> for bool {
    #[inline(always)]
    fn from(variant: RrEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_EN` reader - Round-Robin Enable"]
pub type RrEnR = crate::BitReader<RrEn>;
impl RrEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrEn {
        match self.bits {
            false => RrEn::Disable,
            true => RrEn::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RrEn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RrEn::Enable
    }
}
#[doc = "Field `RR_EN` writer - Round-Robin Enable"]
pub type RrEnW<'a, REG> = crate::BitWriter<'a, REG, RrEn>;
impl<'a, REG> RrEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RrEn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RrEn::Enable)
    }
}
#[doc = "Round-Robin Trigger Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrTrgSel {
    #[doc = "0: External trigger"]
    Enable = 0,
    #[doc = "1: Internal trigger"]
    Disable = 1,
}
impl From<RrTrgSel> for bool {
    #[inline(always)]
    fn from(variant: RrTrgSel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RR_TRG_SEL` reader - Round-Robin Trigger Select"]
pub type RrTrgSelR = crate::BitReader<RrTrgSel>;
impl RrTrgSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrTrgSel {
        match self.bits {
            false => RrTrgSel::Enable,
            true => RrTrgSel::Disable,
        }
    }
    #[doc = "External trigger"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RrTrgSel::Enable
    }
    #[doc = "Internal trigger"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RrTrgSel::Disable
    }
}
#[doc = "Field `RR_TRG_SEL` writer - Round-Robin Trigger Select"]
pub type RrTrgSelW<'a, REG> = crate::BitWriter<'a, REG, RrTrgSel>;
impl<'a, REG> RrTrgSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "External trigger"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RrTrgSel::Enable)
    }
    #[doc = "Internal trigger"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RrTrgSel::Disable)
    }
}
#[doc = "Number of Sample Clocks\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RrNsam {
    #[doc = "0: 0 clock"]
    Wait0 = 0,
    #[doc = "1: 1 clock"]
    Wait1 = 1,
    #[doc = "2: 2 clocks"]
    Wait2 = 2,
    #[doc = "3: 3 clocks"]
    Wait3 = 3,
}
impl From<RrNsam> for u8 {
    #[inline(always)]
    fn from(variant: RrNsam) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for RrNsam {
    type Ux = u8;
}
impl crate::IsEnum for RrNsam {}
#[doc = "Field `RR_NSAM` reader - Number of Sample Clocks"]
pub type RrNsamR = crate::FieldReader<RrNsam>;
impl RrNsamR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrNsam {
        match self.bits {
            0 => RrNsam::Wait0,
            1 => RrNsam::Wait1,
            2 => RrNsam::Wait2,
            3 => RrNsam::Wait3,
            _ => unreachable!(),
        }
    }
    #[doc = "0 clock"]
    #[inline(always)]
    pub fn is_wait_0(&self) -> bool {
        *self == RrNsam::Wait0
    }
    #[doc = "1 clock"]
    #[inline(always)]
    pub fn is_wait_1(&self) -> bool {
        *self == RrNsam::Wait1
    }
    #[doc = "2 clocks"]
    #[inline(always)]
    pub fn is_wait_2(&self) -> bool {
        *self == RrNsam::Wait2
    }
    #[doc = "3 clocks"]
    #[inline(always)]
    pub fn is_wait_3(&self) -> bool {
        *self == RrNsam::Wait3
    }
}
#[doc = "Field `RR_NSAM` writer - Number of Sample Clocks"]
pub type RrNsamW<'a, REG> = crate::FieldWriter<'a, REG, 2, RrNsam, crate::Safe>;
impl<'a, REG> RrNsamW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "0 clock"]
    #[inline(always)]
    pub fn wait_0(self) -> &'a mut crate::W<REG> {
        self.variant(RrNsam::Wait0)
    }
    #[doc = "1 clock"]
    #[inline(always)]
    pub fn wait_1(self) -> &'a mut crate::W<REG> {
        self.variant(RrNsam::Wait1)
    }
    #[doc = "2 clocks"]
    #[inline(always)]
    pub fn wait_2(self) -> &'a mut crate::W<REG> {
        self.variant(RrNsam::Wait2)
    }
    #[doc = "3 clocks"]
    #[inline(always)]
    pub fn wait_3(self) -> &'a mut crate::W<REG> {
        self.variant(RrNsam::Wait3)
    }
}
#[doc = "Round Robin Clock Source Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RrClkSel {
    #[doc = "0: Select Round Robin clock Source 0"]
    Rr0 = 0,
    #[doc = "1: Select Round Robin clock Source 1"]
    Rr1 = 1,
    #[doc = "2: Select Round Robin clock Source 2"]
    Rr2 = 2,
    #[doc = "3: Select Round Robin clock Source 3"]
    Rr3 = 3,
}
impl From<RrClkSel> for u8 {
    #[inline(always)]
    fn from(variant: RrClkSel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for RrClkSel {
    type Ux = u8;
}
impl crate::IsEnum for RrClkSel {}
#[doc = "Field `RR_CLK_SEL` reader - Round Robin Clock Source Select"]
pub type RrClkSelR = crate::FieldReader<RrClkSel>;
impl RrClkSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrClkSel {
        match self.bits {
            0 => RrClkSel::Rr0,
            1 => RrClkSel::Rr1,
            2 => RrClkSel::Rr2,
            3 => RrClkSel::Rr3,
            _ => unreachable!(),
        }
    }
    #[doc = "Select Round Robin clock Source 0"]
    #[inline(always)]
    pub fn is_rr0(&self) -> bool {
        *self == RrClkSel::Rr0
    }
    #[doc = "Select Round Robin clock Source 1"]
    #[inline(always)]
    pub fn is_rr1(&self) -> bool {
        *self == RrClkSel::Rr1
    }
    #[doc = "Select Round Robin clock Source 2"]
    #[inline(always)]
    pub fn is_rr2(&self) -> bool {
        *self == RrClkSel::Rr2
    }
    #[doc = "Select Round Robin clock Source 3"]
    #[inline(always)]
    pub fn is_rr3(&self) -> bool {
        *self == RrClkSel::Rr3
    }
}
#[doc = "Field `RR_CLK_SEL` writer - Round Robin Clock Source Select"]
pub type RrClkSelW<'a, REG> = crate::FieldWriter<'a, REG, 2, RrClkSel, crate::Safe>;
impl<'a, REG> RrClkSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Select Round Robin clock Source 0"]
    #[inline(always)]
    pub fn rr0(self) -> &'a mut crate::W<REG> {
        self.variant(RrClkSel::Rr0)
    }
    #[doc = "Select Round Robin clock Source 1"]
    #[inline(always)]
    pub fn rr1(self) -> &'a mut crate::W<REG> {
        self.variant(RrClkSel::Rr1)
    }
    #[doc = "Select Round Robin clock Source 2"]
    #[inline(always)]
    pub fn rr2(self) -> &'a mut crate::W<REG> {
        self.variant(RrClkSel::Rr2)
    }
    #[doc = "Select Round Robin clock Source 3"]
    #[inline(always)]
    pub fn rr3(self) -> &'a mut crate::W<REG> {
        self.variant(RrClkSel::Rr3)
    }
}
#[doc = "Initialization Delay Modulus\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RrInitmod {
    #[doc = "0: 63 cycles (same as 111111b)"]
    Mod63 = 0,
    #[doc = "1: 1 to 63 cycles"]
    Mod1_63_1 = 1,
    #[doc = "2: 1 to 63 cycles"]
    Mod1_63_2 = 2,
    #[doc = "3: 1 to 63 cycles"]
    Mod1_63_3 = 3,
    #[doc = "4: 1 to 63 cycles"]
    Mod1_63_4 = 4,
    #[doc = "5: 1 to 63 cycles"]
    Mod1_63_5 = 5,
    #[doc = "6: 1 to 63 cycles"]
    Mod1_63_6 = 6,
    #[doc = "7: 1 to 63 cycles"]
    Mod1_63_7 = 7,
    #[doc = "8: 1 to 63 cycles"]
    Mod1_63_8 = 8,
    #[doc = "9: 1 to 63 cycles"]
    Mod1_63_9 = 9,
}
impl From<RrInitmod> for u8 {
    #[inline(always)]
    fn from(variant: RrInitmod) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for RrInitmod {
    type Ux = u8;
}
impl crate::IsEnum for RrInitmod {}
#[doc = "Field `RR_INITMOD` reader - Initialization Delay Modulus"]
pub type RrInitmodR = crate::FieldReader<RrInitmod>;
impl RrInitmodR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<RrInitmod> {
        match self.bits {
            0 => Some(RrInitmod::Mod63),
            1 => Some(RrInitmod::Mod1_63_1),
            2 => Some(RrInitmod::Mod1_63_2),
            3 => Some(RrInitmod::Mod1_63_3),
            4 => Some(RrInitmod::Mod1_63_4),
            5 => Some(RrInitmod::Mod1_63_5),
            6 => Some(RrInitmod::Mod1_63_6),
            7 => Some(RrInitmod::Mod1_63_7),
            8 => Some(RrInitmod::Mod1_63_8),
            9 => Some(RrInitmod::Mod1_63_9),
            _ => None,
        }
    }
    #[doc = "63 cycles (same as 111111b)"]
    #[inline(always)]
    pub fn is_mod_63(&self) -> bool {
        *self == RrInitmod::Mod63
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn is_mod_1_63_1(&self) -> bool {
        *self == RrInitmod::Mod1_63_1
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn is_mod_1_63_2(&self) -> bool {
        *self == RrInitmod::Mod1_63_2
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn is_mod_1_63_3(&self) -> bool {
        *self == RrInitmod::Mod1_63_3
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn is_mod_1_63_4(&self) -> bool {
        *self == RrInitmod::Mod1_63_4
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn is_mod_1_63_5(&self) -> bool {
        *self == RrInitmod::Mod1_63_5
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn is_mod_1_63_6(&self) -> bool {
        *self == RrInitmod::Mod1_63_6
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn is_mod_1_63_7(&self) -> bool {
        *self == RrInitmod::Mod1_63_7
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn is_mod_1_63_8(&self) -> bool {
        *self == RrInitmod::Mod1_63_8
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn is_mod_1_63_9(&self) -> bool {
        *self == RrInitmod::Mod1_63_9
    }
}
#[doc = "Field `RR_INITMOD` writer - Initialization Delay Modulus"]
pub type RrInitmodW<'a, REG> = crate::FieldWriter<'a, REG, 6, RrInitmod>;
impl<'a, REG> RrInitmodW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "63 cycles (same as 111111b)"]
    #[inline(always)]
    pub fn mod_63(self) -> &'a mut crate::W<REG> {
        self.variant(RrInitmod::Mod63)
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn mod_1_63_1(self) -> &'a mut crate::W<REG> {
        self.variant(RrInitmod::Mod1_63_1)
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn mod_1_63_2(self) -> &'a mut crate::W<REG> {
        self.variant(RrInitmod::Mod1_63_2)
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn mod_1_63_3(self) -> &'a mut crate::W<REG> {
        self.variant(RrInitmod::Mod1_63_3)
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn mod_1_63_4(self) -> &'a mut crate::W<REG> {
        self.variant(RrInitmod::Mod1_63_4)
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn mod_1_63_5(self) -> &'a mut crate::W<REG> {
        self.variant(RrInitmod::Mod1_63_5)
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn mod_1_63_6(self) -> &'a mut crate::W<REG> {
        self.variant(RrInitmod::Mod1_63_6)
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn mod_1_63_7(self) -> &'a mut crate::W<REG> {
        self.variant(RrInitmod::Mod1_63_7)
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn mod_1_63_8(self) -> &'a mut crate::W<REG> {
        self.variant(RrInitmod::Mod1_63_8)
    }
    #[doc = "1 to 63 cycles"]
    #[inline(always)]
    pub fn mod_1_63_9(self) -> &'a mut crate::W<REG> {
        self.variant(RrInitmod::Mod1_63_9)
    }
}
#[doc = "Number of Sample for One Channel\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RrSampleCnt {
    #[doc = "0: 1 samples"]
    Sample0 = 0,
    #[doc = "1: 2 samples"]
    Sample1 = 1,
    #[doc = "2: 3 samples"]
    Sample2 = 2,
    #[doc = "3: 4 samples"]
    Sample3 = 3,
    #[doc = "4: 5 samples"]
    Sample4 = 4,
    #[doc = "5: 6 samples"]
    Sample5 = 5,
    #[doc = "6: 7 samples"]
    Sample6 = 6,
    #[doc = "7: 8 samples"]
    Sample7 = 7,
    #[doc = "8: 9 samples"]
    Sample8 = 8,
    #[doc = "9: 10 samples"]
    Sample9 = 9,
    #[doc = "10: 11 samples"]
    Sample10 = 10,
    #[doc = "11: 12 samples"]
    Sample11 = 11,
    #[doc = "12: 13 samples"]
    Sample12 = 12,
    #[doc = "13: 14 samples"]
    Sample13 = 13,
    #[doc = "14: 15 samples"]
    Sample14 = 14,
    #[doc = "15: 16 samples"]
    Sample15 = 15,
}
impl From<RrSampleCnt> for u8 {
    #[inline(always)]
    fn from(variant: RrSampleCnt) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for RrSampleCnt {
    type Ux = u8;
}
impl crate::IsEnum for RrSampleCnt {}
#[doc = "Field `RR_SAMPLE_CNT` reader - Number of Sample for One Channel"]
pub type RrSampleCntR = crate::FieldReader<RrSampleCnt>;
impl RrSampleCntR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrSampleCnt {
        match self.bits {
            0 => RrSampleCnt::Sample0,
            1 => RrSampleCnt::Sample1,
            2 => RrSampleCnt::Sample2,
            3 => RrSampleCnt::Sample3,
            4 => RrSampleCnt::Sample4,
            5 => RrSampleCnt::Sample5,
            6 => RrSampleCnt::Sample6,
            7 => RrSampleCnt::Sample7,
            8 => RrSampleCnt::Sample8,
            9 => RrSampleCnt::Sample9,
            10 => RrSampleCnt::Sample10,
            11 => RrSampleCnt::Sample11,
            12 => RrSampleCnt::Sample12,
            13 => RrSampleCnt::Sample13,
            14 => RrSampleCnt::Sample14,
            15 => RrSampleCnt::Sample15,
            _ => unreachable!(),
        }
    }
    #[doc = "1 samples"]
    #[inline(always)]
    pub fn is_sample_0(&self) -> bool {
        *self == RrSampleCnt::Sample0
    }
    #[doc = "2 samples"]
    #[inline(always)]
    pub fn is_sample_1(&self) -> bool {
        *self == RrSampleCnt::Sample1
    }
    #[doc = "3 samples"]
    #[inline(always)]
    pub fn is_sample_2(&self) -> bool {
        *self == RrSampleCnt::Sample2
    }
    #[doc = "4 samples"]
    #[inline(always)]
    pub fn is_sample_3(&self) -> bool {
        *self == RrSampleCnt::Sample3
    }
    #[doc = "5 samples"]
    #[inline(always)]
    pub fn is_sample_4(&self) -> bool {
        *self == RrSampleCnt::Sample4
    }
    #[doc = "6 samples"]
    #[inline(always)]
    pub fn is_sample_5(&self) -> bool {
        *self == RrSampleCnt::Sample5
    }
    #[doc = "7 samples"]
    #[inline(always)]
    pub fn is_sample_6(&self) -> bool {
        *self == RrSampleCnt::Sample6
    }
    #[doc = "8 samples"]
    #[inline(always)]
    pub fn is_sample_7(&self) -> bool {
        *self == RrSampleCnt::Sample7
    }
    #[doc = "9 samples"]
    #[inline(always)]
    pub fn is_sample_8(&self) -> bool {
        *self == RrSampleCnt::Sample8
    }
    #[doc = "10 samples"]
    #[inline(always)]
    pub fn is_sample_9(&self) -> bool {
        *self == RrSampleCnt::Sample9
    }
    #[doc = "11 samples"]
    #[inline(always)]
    pub fn is_sample_10(&self) -> bool {
        *self == RrSampleCnt::Sample10
    }
    #[doc = "12 samples"]
    #[inline(always)]
    pub fn is_sample_11(&self) -> bool {
        *self == RrSampleCnt::Sample11
    }
    #[doc = "13 samples"]
    #[inline(always)]
    pub fn is_sample_12(&self) -> bool {
        *self == RrSampleCnt::Sample12
    }
    #[doc = "14 samples"]
    #[inline(always)]
    pub fn is_sample_13(&self) -> bool {
        *self == RrSampleCnt::Sample13
    }
    #[doc = "15 samples"]
    #[inline(always)]
    pub fn is_sample_14(&self) -> bool {
        *self == RrSampleCnt::Sample14
    }
    #[doc = "16 samples"]
    #[inline(always)]
    pub fn is_sample_15(&self) -> bool {
        *self == RrSampleCnt::Sample15
    }
}
#[doc = "Field `RR_SAMPLE_CNT` writer - Number of Sample for One Channel"]
pub type RrSampleCntW<'a, REG> = crate::FieldWriter<'a, REG, 4, RrSampleCnt, crate::Safe>;
impl<'a, REG> RrSampleCntW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "1 samples"]
    #[inline(always)]
    pub fn sample_0(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleCnt::Sample0)
    }
    #[doc = "2 samples"]
    #[inline(always)]
    pub fn sample_1(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleCnt::Sample1)
    }
    #[doc = "3 samples"]
    #[inline(always)]
    pub fn sample_2(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleCnt::Sample2)
    }
    #[doc = "4 samples"]
    #[inline(always)]
    pub fn sample_3(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleCnt::Sample3)
    }
    #[doc = "5 samples"]
    #[inline(always)]
    pub fn sample_4(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleCnt::Sample4)
    }
    #[doc = "6 samples"]
    #[inline(always)]
    pub fn sample_5(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleCnt::Sample5)
    }
    #[doc = "7 samples"]
    #[inline(always)]
    pub fn sample_6(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleCnt::Sample6)
    }
    #[doc = "8 samples"]
    #[inline(always)]
    pub fn sample_7(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleCnt::Sample7)
    }
    #[doc = "9 samples"]
    #[inline(always)]
    pub fn sample_8(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleCnt::Sample8)
    }
    #[doc = "10 samples"]
    #[inline(always)]
    pub fn sample_9(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleCnt::Sample9)
    }
    #[doc = "11 samples"]
    #[inline(always)]
    pub fn sample_10(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleCnt::Sample10)
    }
    #[doc = "12 samples"]
    #[inline(always)]
    pub fn sample_11(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleCnt::Sample11)
    }
    #[doc = "13 samples"]
    #[inline(always)]
    pub fn sample_12(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleCnt::Sample12)
    }
    #[doc = "14 samples"]
    #[inline(always)]
    pub fn sample_13(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleCnt::Sample13)
    }
    #[doc = "15 samples"]
    #[inline(always)]
    pub fn sample_14(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleCnt::Sample14)
    }
    #[doc = "16 samples"]
    #[inline(always)]
    pub fn sample_15(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleCnt::Sample15)
    }
}
#[doc = "Sample Time Threshold\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RrSampleThreshold {
    #[doc = "0: At least 1 sampled \"1\", the final result is \"1\""]
    Sample0 = 0,
    #[doc = "1: At least 2 sampled \"1\", the final result is \"1\""]
    Sample1 = 1,
    #[doc = "2: At least 3 sampled \"1\", the final result is \"1\""]
    Sample2 = 2,
    #[doc = "3: At least 4 sampled \"1\", the final result is \"1\""]
    Sample3 = 3,
    #[doc = "4: At least 5 sampled \"1\", the final result is \"1\""]
    Sample4 = 4,
    #[doc = "5: At least 6 sampled \"1\", the final result is \"1\""]
    Sample5 = 5,
    #[doc = "6: At least 7 sampled \"1\", the final result is \"1\""]
    Sample6 = 6,
    #[doc = "7: At least 8 sampled \"1\", the final result is \"1\""]
    Sample7 = 7,
    #[doc = "8: At least 9 sampled \"1\", the final result is \"1\""]
    Sample8 = 8,
    #[doc = "9: At least 10 sampled \"1\", the final result is \"1\""]
    Sample9 = 9,
    #[doc = "10: At least 11 sampled \"1\", the final result is \"1\""]
    Sample10 = 10,
    #[doc = "11: At least 12 sampled \"1\", the final result is \"1\""]
    Sample11 = 11,
    #[doc = "12: At least 13 sampled \"1\", the final result is \"1\""]
    Sample12 = 12,
    #[doc = "13: At least 14 sampled \"1\", the final result is \"1\""]
    Sample13 = 13,
    #[doc = "14: At least 15 sampled \"1\", the final result is \"1\""]
    Sample14 = 14,
    #[doc = "15: At least 16 sampled \"1\", the final result is \"1\""]
    Sample15 = 15,
}
impl From<RrSampleThreshold> for u8 {
    #[inline(always)]
    fn from(variant: RrSampleThreshold) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for RrSampleThreshold {
    type Ux = u8;
}
impl crate::IsEnum for RrSampleThreshold {}
#[doc = "Field `RR_SAMPLE_THRESHOLD` reader - Sample Time Threshold"]
pub type RrSampleThresholdR = crate::FieldReader<RrSampleThreshold>;
impl RrSampleThresholdR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrSampleThreshold {
        match self.bits {
            0 => RrSampleThreshold::Sample0,
            1 => RrSampleThreshold::Sample1,
            2 => RrSampleThreshold::Sample2,
            3 => RrSampleThreshold::Sample3,
            4 => RrSampleThreshold::Sample4,
            5 => RrSampleThreshold::Sample5,
            6 => RrSampleThreshold::Sample6,
            7 => RrSampleThreshold::Sample7,
            8 => RrSampleThreshold::Sample8,
            9 => RrSampleThreshold::Sample9,
            10 => RrSampleThreshold::Sample10,
            11 => RrSampleThreshold::Sample11,
            12 => RrSampleThreshold::Sample12,
            13 => RrSampleThreshold::Sample13,
            14 => RrSampleThreshold::Sample14,
            15 => RrSampleThreshold::Sample15,
            _ => unreachable!(),
        }
    }
    #[doc = "At least 1 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn is_sample_0(&self) -> bool {
        *self == RrSampleThreshold::Sample0
    }
    #[doc = "At least 2 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn is_sample_1(&self) -> bool {
        *self == RrSampleThreshold::Sample1
    }
    #[doc = "At least 3 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn is_sample_2(&self) -> bool {
        *self == RrSampleThreshold::Sample2
    }
    #[doc = "At least 4 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn is_sample_3(&self) -> bool {
        *self == RrSampleThreshold::Sample3
    }
    #[doc = "At least 5 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn is_sample_4(&self) -> bool {
        *self == RrSampleThreshold::Sample4
    }
    #[doc = "At least 6 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn is_sample_5(&self) -> bool {
        *self == RrSampleThreshold::Sample5
    }
    #[doc = "At least 7 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn is_sample_6(&self) -> bool {
        *self == RrSampleThreshold::Sample6
    }
    #[doc = "At least 8 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn is_sample_7(&self) -> bool {
        *self == RrSampleThreshold::Sample7
    }
    #[doc = "At least 9 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn is_sample_8(&self) -> bool {
        *self == RrSampleThreshold::Sample8
    }
    #[doc = "At least 10 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn is_sample_9(&self) -> bool {
        *self == RrSampleThreshold::Sample9
    }
    #[doc = "At least 11 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn is_sample_10(&self) -> bool {
        *self == RrSampleThreshold::Sample10
    }
    #[doc = "At least 12 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn is_sample_11(&self) -> bool {
        *self == RrSampleThreshold::Sample11
    }
    #[doc = "At least 13 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn is_sample_12(&self) -> bool {
        *self == RrSampleThreshold::Sample12
    }
    #[doc = "At least 14 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn is_sample_13(&self) -> bool {
        *self == RrSampleThreshold::Sample13
    }
    #[doc = "At least 15 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn is_sample_14(&self) -> bool {
        *self == RrSampleThreshold::Sample14
    }
    #[doc = "At least 16 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn is_sample_15(&self) -> bool {
        *self == RrSampleThreshold::Sample15
    }
}
#[doc = "Field `RR_SAMPLE_THRESHOLD` writer - Sample Time Threshold"]
pub type RrSampleThresholdW<'a, REG> =
    crate::FieldWriter<'a, REG, 4, RrSampleThreshold, crate::Safe>;
impl<'a, REG> RrSampleThresholdW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "At least 1 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn sample_0(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleThreshold::Sample0)
    }
    #[doc = "At least 2 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn sample_1(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleThreshold::Sample1)
    }
    #[doc = "At least 3 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn sample_2(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleThreshold::Sample2)
    }
    #[doc = "At least 4 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn sample_3(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleThreshold::Sample3)
    }
    #[doc = "At least 5 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn sample_4(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleThreshold::Sample4)
    }
    #[doc = "At least 6 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn sample_5(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleThreshold::Sample5)
    }
    #[doc = "At least 7 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn sample_6(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleThreshold::Sample6)
    }
    #[doc = "At least 8 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn sample_7(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleThreshold::Sample7)
    }
    #[doc = "At least 9 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn sample_8(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleThreshold::Sample8)
    }
    #[doc = "At least 10 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn sample_9(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleThreshold::Sample9)
    }
    #[doc = "At least 11 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn sample_10(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleThreshold::Sample10)
    }
    #[doc = "At least 12 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn sample_11(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleThreshold::Sample11)
    }
    #[doc = "At least 13 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn sample_12(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleThreshold::Sample12)
    }
    #[doc = "At least 14 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn sample_13(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleThreshold::Sample13)
    }
    #[doc = "At least 15 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn sample_14(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleThreshold::Sample14)
    }
    #[doc = "At least 16 sampled \"1\", the final result is \"1\""]
    #[inline(always)]
    pub fn sample_15(self) -> &'a mut crate::W<REG> {
        self.variant(RrSampleThreshold::Sample15)
    }
}
impl R {
    #[doc = "Bit 0 - Round-Robin Enable"]
    #[inline(always)]
    pub fn rr_en(&self) -> RrEnR {
        RrEnR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Round-Robin Trigger Select"]
    #[inline(always)]
    pub fn rr_trg_sel(&self) -> RrTrgSelR {
        RrTrgSelR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bits 8:9 - Number of Sample Clocks"]
    #[inline(always)]
    pub fn rr_nsam(&self) -> RrNsamR {
        RrNsamR::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bits 12:13 - Round Robin Clock Source Select"]
    #[inline(always)]
    pub fn rr_clk_sel(&self) -> RrClkSelR {
        RrClkSelR::new(((self.bits >> 12) & 3) as u8)
    }
    #[doc = "Bits 16:21 - Initialization Delay Modulus"]
    #[inline(always)]
    pub fn rr_initmod(&self) -> RrInitmodR {
        RrInitmodR::new(((self.bits >> 16) & 0x3f) as u8)
    }
    #[doc = "Bits 24:27 - Number of Sample for One Channel"]
    #[inline(always)]
    pub fn rr_sample_cnt(&self) -> RrSampleCntR {
        RrSampleCntR::new(((self.bits >> 24) & 0x0f) as u8)
    }
    #[doc = "Bits 28:31 - Sample Time Threshold"]
    #[inline(always)]
    pub fn rr_sample_threshold(&self) -> RrSampleThresholdR {
        RrSampleThresholdR::new(((self.bits >> 28) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Round-Robin Enable"]
    #[inline(always)]
    pub fn rr_en(&mut self) -> RrEnW<Rrcr0Spec> {
        RrEnW::new(self, 0)
    }
    #[doc = "Bit 1 - Round-Robin Trigger Select"]
    #[inline(always)]
    pub fn rr_trg_sel(&mut self) -> RrTrgSelW<Rrcr0Spec> {
        RrTrgSelW::new(self, 1)
    }
    #[doc = "Bits 8:9 - Number of Sample Clocks"]
    #[inline(always)]
    pub fn rr_nsam(&mut self) -> RrNsamW<Rrcr0Spec> {
        RrNsamW::new(self, 8)
    }
    #[doc = "Bits 12:13 - Round Robin Clock Source Select"]
    #[inline(always)]
    pub fn rr_clk_sel(&mut self) -> RrClkSelW<Rrcr0Spec> {
        RrClkSelW::new(self, 12)
    }
    #[doc = "Bits 16:21 - Initialization Delay Modulus"]
    #[inline(always)]
    pub fn rr_initmod(&mut self) -> RrInitmodW<Rrcr0Spec> {
        RrInitmodW::new(self, 16)
    }
    #[doc = "Bits 24:27 - Number of Sample for One Channel"]
    #[inline(always)]
    pub fn rr_sample_cnt(&mut self) -> RrSampleCntW<Rrcr0Spec> {
        RrSampleCntW::new(self, 24)
    }
    #[doc = "Bits 28:31 - Sample Time Threshold"]
    #[inline(always)]
    pub fn rr_sample_threshold(&mut self) -> RrSampleThresholdW<Rrcr0Spec> {
        RrSampleThresholdW::new(self, 28)
    }
}
#[doc = "Round Robin Control Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`rrcr0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rrcr0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Rrcr0Spec;
impl crate::RegisterSpec for Rrcr0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rrcr0::R`](R) reader structure"]
impl crate::Readable for Rrcr0Spec {}
#[doc = "`write(|w| ..)` method takes [`rrcr0::W`](W) writer structure"]
impl crate::Writable for Rrcr0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RRCR0 to value 0"]
impl crate::Resettable for Rrcr0Spec {}
