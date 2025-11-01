#[doc = "Register `CR` reader"]
pub type R = crate::R<CrSpec>;
#[doc = "Register `CR` writer"]
pub type W = crate::W<CrSpec>;
#[doc = "Software Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Swr {
    #[doc = "0: No effect."]
    Swr0 = 0,
    #[doc = "1: Resets all RTC registers except for the SWR bit . The SWR bit is cleared by POR and by software explicitly clearing it."]
    Swr1 = 1,
}
impl From<Swr> for bool {
    #[inline(always)]
    fn from(variant: Swr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SWR` reader - Software Reset"]
pub type SwrR = crate::BitReader<Swr>;
impl SwrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Swr {
        match self.bits {
            false => Swr::Swr0,
            true => Swr::Swr1,
        }
    }
    #[doc = "No effect."]
    #[inline(always)]
    pub fn is_swr_0(&self) -> bool {
        *self == Swr::Swr0
    }
    #[doc = "Resets all RTC registers except for the SWR bit . The SWR bit is cleared by POR and by software explicitly clearing it."]
    #[inline(always)]
    pub fn is_swr_1(&self) -> bool {
        *self == Swr::Swr1
    }
}
#[doc = "Field `SWR` writer - Software Reset"]
pub type SwrW<'a, REG> = crate::BitWriter<'a, REG, Swr>;
impl<'a, REG> SwrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect."]
    #[inline(always)]
    pub fn swr_0(self) -> &'a mut crate::W<REG> {
        self.variant(Swr::Swr0)
    }
    #[doc = "Resets all RTC registers except for the SWR bit . The SWR bit is cleared by POR and by software explicitly clearing it."]
    #[inline(always)]
    pub fn swr_1(self) -> &'a mut crate::W<REG> {
        self.variant(Swr::Swr1)
    }
}
#[doc = "Update Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Um {
    #[doc = "0: Registers cannot be written when locked."]
    Um0 = 0,
    #[doc = "1: Registers can be written when locked under limited conditions."]
    Um1 = 1,
}
impl From<Um> for bool {
    #[inline(always)]
    fn from(variant: Um) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `UM` reader - Update Mode"]
pub type UmR = crate::BitReader<Um>;
impl UmR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Um {
        match self.bits {
            false => Um::Um0,
            true => Um::Um1,
        }
    }
    #[doc = "Registers cannot be written when locked."]
    #[inline(always)]
    pub fn is_um_0(&self) -> bool {
        *self == Um::Um0
    }
    #[doc = "Registers can be written when locked under limited conditions."]
    #[inline(always)]
    pub fn is_um_1(&self) -> bool {
        *self == Um::Um1
    }
}
#[doc = "Field `UM` writer - Update Mode"]
pub type UmW<'a, REG> = crate::BitWriter<'a, REG, Um>;
impl<'a, REG> UmW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Registers cannot be written when locked."]
    #[inline(always)]
    pub fn um_0(self) -> &'a mut crate::W<REG> {
        self.variant(Um::Um0)
    }
    #[doc = "Registers can be written when locked under limited conditions."]
    #[inline(always)]
    pub fn um_1(self) -> &'a mut crate::W<REG> {
        self.variant(Um::Um1)
    }
}
#[doc = "LPO Select\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpos {
    #[doc = "0: RTC prescaler increments using 32.768 kHz clock."]
    Lpos0 = 0,
    #[doc = "1: RTC prescaler increments using 1 kHz LPO, bits \\[4:0\\] of the prescaler are ignored."]
    Lpos1 = 1,
}
impl From<Lpos> for bool {
    #[inline(always)]
    fn from(variant: Lpos) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPOS` reader - LPO Select"]
pub type LposR = crate::BitReader<Lpos>;
impl LposR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpos {
        match self.bits {
            false => Lpos::Lpos0,
            true => Lpos::Lpos1,
        }
    }
    #[doc = "RTC prescaler increments using 32.768 kHz clock."]
    #[inline(always)]
    pub fn is_lpos_0(&self) -> bool {
        *self == Lpos::Lpos0
    }
    #[doc = "RTC prescaler increments using 1 kHz LPO, bits \\[4:0\\] of the prescaler are ignored."]
    #[inline(always)]
    pub fn is_lpos_1(&self) -> bool {
        *self == Lpos::Lpos1
    }
}
#[doc = "Field `LPOS` writer - LPO Select"]
pub type LposW<'a, REG> = crate::BitWriter<'a, REG, Lpos>;
impl<'a, REG> LposW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "RTC prescaler increments using 32.768 kHz clock."]
    #[inline(always)]
    pub fn lpos_0(self) -> &'a mut crate::W<REG> {
        self.variant(Lpos::Lpos0)
    }
    #[doc = "RTC prescaler increments using 1 kHz LPO, bits \\[4:0\\] of the prescaler are ignored."]
    #[inline(always)]
    pub fn lpos_1(self) -> &'a mut crate::W<REG> {
        self.variant(Lpos::Lpos1)
    }
}
impl R {
    #[doc = "Bit 0 - Software Reset"]
    #[inline(always)]
    pub fn swr(&self) -> SwrR {
        SwrR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 3 - Update Mode"]
    #[inline(always)]
    pub fn um(&self) -> UmR {
        UmR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 7 - LPO Select"]
    #[inline(always)]
    pub fn lpos(&self) -> LposR {
        LposR::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Software Reset"]
    #[inline(always)]
    pub fn swr(&mut self) -> SwrW<CrSpec> {
        SwrW::new(self, 0)
    }
    #[doc = "Bit 3 - Update Mode"]
    #[inline(always)]
    pub fn um(&mut self) -> UmW<CrSpec> {
        UmW::new(self, 3)
    }
    #[doc = "Bit 7 - LPO Select"]
    #[inline(always)]
    pub fn lpos(&mut self) -> LposW<CrSpec> {
        LposW::new(self, 7)
    }
}
#[doc = "RTC Control\n\nYou can [`read`](crate::Reg::read) this register and get [`cr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CrSpec;
impl crate::RegisterSpec for CrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cr::R`](R) reader structure"]
impl crate::Readable for CrSpec {}
#[doc = "`write(|w| ..)` method takes [`cr::W`](W) writer structure"]
impl crate::Writable for CrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CR to value 0x80"]
impl crate::Resettable for CrSpec {
    const RESET_VALUE: u32 = 0x80;
}
