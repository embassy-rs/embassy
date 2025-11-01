#[doc = "Register `LPREQ_CFG` reader"]
pub type R = crate::R<LpreqCfgSpec>;
#[doc = "Register `LPREQ_CFG` writer"]
pub type W = crate::W<LpreqCfgSpec>;
#[doc = "Low-Power Request Output Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpreqoe {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Lpreqoe> for bool {
    #[inline(always)]
    fn from(variant: Lpreqoe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPREQOE` reader - Low-Power Request Output Enable"]
pub type LpreqoeR = crate::BitReader<Lpreqoe>;
impl LpreqoeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpreqoe {
        match self.bits {
            false => Lpreqoe::Disable,
            true => Lpreqoe::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Lpreqoe::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Lpreqoe::Enable
    }
}
#[doc = "Field `LPREQOE` writer - Low-Power Request Output Enable"]
pub type LpreqoeW<'a, REG> = crate::BitWriter<'a, REG, Lpreqoe>;
impl<'a, REG> LpreqoeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Lpreqoe::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Lpreqoe::Enable)
    }
}
#[doc = "Low-Power Request Output Pin Polarity Control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpreqpol {
    #[doc = "0: High"]
    High = 0,
    #[doc = "1: Low"]
    Low = 1,
}
impl From<Lpreqpol> for bool {
    #[inline(always)]
    fn from(variant: Lpreqpol) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPREQPOL` reader - Low-Power Request Output Pin Polarity Control"]
pub type LpreqpolR = crate::BitReader<Lpreqpol>;
impl LpreqpolR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpreqpol {
        match self.bits {
            false => Lpreqpol::High,
            true => Lpreqpol::Low,
        }
    }
    #[doc = "High"]
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        *self == Lpreqpol::High
    }
    #[doc = "Low"]
    #[inline(always)]
    pub fn is_low(&self) -> bool {
        *self == Lpreqpol::Low
    }
}
#[doc = "Field `LPREQPOL` writer - Low-Power Request Output Pin Polarity Control"]
pub type LpreqpolW<'a, REG> = crate::BitWriter<'a, REG, Lpreqpol>;
impl<'a, REG> LpreqpolW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "High"]
    #[inline(always)]
    pub fn high(self) -> &'a mut crate::W<REG> {
        self.variant(Lpreqpol::High)
    }
    #[doc = "Low"]
    #[inline(always)]
    pub fn low(self) -> &'a mut crate::W<REG> {
        self.variant(Lpreqpol::Low)
    }
}
#[doc = "Low-Power Request Output Override\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Lpreqov {
    #[doc = "0: Not forced"]
    ForceNo = 0,
    #[doc = "2: Forced low (ignore LPREQPOL settings)"]
    ForceLow = 2,
    #[doc = "3: Forced high (ignore LPREQPOL settings)"]
    ForceHigh = 3,
}
impl From<Lpreqov> for u8 {
    #[inline(always)]
    fn from(variant: Lpreqov) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Lpreqov {
    type Ux = u8;
}
impl crate::IsEnum for Lpreqov {}
#[doc = "Field `LPREQOV` reader - Low-Power Request Output Override"]
pub type LpreqovR = crate::FieldReader<Lpreqov>;
impl LpreqovR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Lpreqov> {
        match self.bits {
            0 => Some(Lpreqov::ForceNo),
            2 => Some(Lpreqov::ForceLow),
            3 => Some(Lpreqov::ForceHigh),
            _ => None,
        }
    }
    #[doc = "Not forced"]
    #[inline(always)]
    pub fn is_force_no(&self) -> bool {
        *self == Lpreqov::ForceNo
    }
    #[doc = "Forced low (ignore LPREQPOL settings)"]
    #[inline(always)]
    pub fn is_force_low(&self) -> bool {
        *self == Lpreqov::ForceLow
    }
    #[doc = "Forced high (ignore LPREQPOL settings)"]
    #[inline(always)]
    pub fn is_force_high(&self) -> bool {
        *self == Lpreqov::ForceHigh
    }
}
#[doc = "Field `LPREQOV` writer - Low-Power Request Output Override"]
pub type LpreqovW<'a, REG> = crate::FieldWriter<'a, REG, 2, Lpreqov>;
impl<'a, REG> LpreqovW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Not forced"]
    #[inline(always)]
    pub fn force_no(self) -> &'a mut crate::W<REG> {
        self.variant(Lpreqov::ForceNo)
    }
    #[doc = "Forced low (ignore LPREQPOL settings)"]
    #[inline(always)]
    pub fn force_low(self) -> &'a mut crate::W<REG> {
        self.variant(Lpreqov::ForceLow)
    }
    #[doc = "Forced high (ignore LPREQPOL settings)"]
    #[inline(always)]
    pub fn force_high(self) -> &'a mut crate::W<REG> {
        self.variant(Lpreqov::ForceHigh)
    }
}
impl R {
    #[doc = "Bit 0 - Low-Power Request Output Enable"]
    #[inline(always)]
    pub fn lpreqoe(&self) -> LpreqoeR {
        LpreqoeR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Low-Power Request Output Pin Polarity Control"]
    #[inline(always)]
    pub fn lpreqpol(&self) -> LpreqpolR {
        LpreqpolR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bits 2:3 - Low-Power Request Output Override"]
    #[inline(always)]
    pub fn lpreqov(&self) -> LpreqovR {
        LpreqovR::new(((self.bits >> 2) & 3) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Low-Power Request Output Enable"]
    #[inline(always)]
    pub fn lpreqoe(&mut self) -> LpreqoeW<LpreqCfgSpec> {
        LpreqoeW::new(self, 0)
    }
    #[doc = "Bit 1 - Low-Power Request Output Pin Polarity Control"]
    #[inline(always)]
    pub fn lpreqpol(&mut self) -> LpreqpolW<LpreqCfgSpec> {
        LpreqpolW::new(self, 1)
    }
    #[doc = "Bits 2:3 - Low-Power Request Output Override"]
    #[inline(always)]
    pub fn lpreqov(&mut self) -> LpreqovW<LpreqCfgSpec> {
        LpreqovW::new(self, 2)
    }
}
#[doc = "Low-Power Request Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`lpreq_cfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpreq_cfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LpreqCfgSpec;
impl crate::RegisterSpec for LpreqCfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`lpreq_cfg::R`](R) reader structure"]
impl crate::Readable for LpreqCfgSpec {}
#[doc = "`write(|w| ..)` method takes [`lpreq_cfg::W`](W) writer structure"]
impl crate::Writable for LpreqCfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LPREQ_CFG to value 0"]
impl crate::Resettable for LpreqCfgSpec {}
