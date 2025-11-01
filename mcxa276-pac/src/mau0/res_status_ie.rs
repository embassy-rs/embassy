#[doc = "Register `RES_STATUS_IE` reader"]
pub type R = crate::R<ResStatusIeSpec>;
#[doc = "Register `RES_STATUS_IE` writer"]
pub type W = crate::W<ResStatusIeSpec>;
#[doc = "RES0 Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res0Ie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Res0Ie> for bool {
    #[inline(always)]
    fn from(variant: Res0Ie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES0_IE` reader - RES0 Interrupt Enable"]
pub type Res0IeR = crate::BitReader<Res0Ie>;
impl Res0IeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res0Ie {
        match self.bits {
            false => Res0Ie::Disable,
            true => Res0Ie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Res0Ie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Res0Ie::Enable
    }
}
#[doc = "Field `RES0_IE` writer - RES0 Interrupt Enable"]
pub type Res0IeW<'a, REG> = crate::BitWriter<'a, REG, Res0Ie>;
impl<'a, REG> Res0IeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Ie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Res0Ie::Enable)
    }
}
#[doc = "RES1 Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res1Ie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Res1Ie> for bool {
    #[inline(always)]
    fn from(variant: Res1Ie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES1_IE` reader - RES1 Interrupt Enable"]
pub type Res1IeR = crate::BitReader<Res1Ie>;
impl Res1IeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res1Ie {
        match self.bits {
            false => Res1Ie::Disable,
            true => Res1Ie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Res1Ie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Res1Ie::Enable
    }
}
#[doc = "Field `RES1_IE` writer - RES1 Interrupt Enable"]
pub type Res1IeW<'a, REG> = crate::BitWriter<'a, REG, Res1Ie>;
impl<'a, REG> Res1IeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Ie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Res1Ie::Enable)
    }
}
#[doc = "RES2 Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res2Ie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Res2Ie> for bool {
    #[inline(always)]
    fn from(variant: Res2Ie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES2_IE` reader - RES2 Interrupt Enable"]
pub type Res2IeR = crate::BitReader<Res2Ie>;
impl Res2IeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res2Ie {
        match self.bits {
            false => Res2Ie::Disable,
            true => Res2Ie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Res2Ie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Res2Ie::Enable
    }
}
#[doc = "Field `RES2_IE` writer - RES2 Interrupt Enable"]
pub type Res2IeW<'a, REG> = crate::BitWriter<'a, REG, Res2Ie>;
impl<'a, REG> Res2IeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Ie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Res2Ie::Enable)
    }
}
#[doc = "RES3 Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Res3Ie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Res3Ie> for bool {
    #[inline(always)]
    fn from(variant: Res3Ie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RES3_IE` reader - RES3 Interrupt Enable"]
pub type Res3IeR = crate::BitReader<Res3Ie>;
impl Res3IeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Res3Ie {
        match self.bits {
            false => Res3Ie::Disable,
            true => Res3Ie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Res3Ie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Res3Ie::Enable
    }
}
#[doc = "Field `RES3_IE` writer - RES3 Interrupt Enable"]
pub type Res3IeW<'a, REG> = crate::BitWriter<'a, REG, Res3Ie>;
impl<'a, REG> Res3IeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Ie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Res3Ie::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - RES0 Interrupt Enable"]
    #[inline(always)]
    pub fn res0_ie(&self) -> Res0IeR {
        Res0IeR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - RES1 Interrupt Enable"]
    #[inline(always)]
    pub fn res1_ie(&self) -> Res1IeR {
        Res1IeR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - RES2 Interrupt Enable"]
    #[inline(always)]
    pub fn res2_ie(&self) -> Res2IeR {
        Res2IeR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - RES3 Interrupt Enable"]
    #[inline(always)]
    pub fn res3_ie(&self) -> Res3IeR {
        Res3IeR::new(((self.bits >> 3) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - RES0 Interrupt Enable"]
    #[inline(always)]
    pub fn res0_ie(&mut self) -> Res0IeW<ResStatusIeSpec> {
        Res0IeW::new(self, 0)
    }
    #[doc = "Bit 1 - RES1 Interrupt Enable"]
    #[inline(always)]
    pub fn res1_ie(&mut self) -> Res1IeW<ResStatusIeSpec> {
        Res1IeW::new(self, 1)
    }
    #[doc = "Bit 2 - RES2 Interrupt Enable"]
    #[inline(always)]
    pub fn res2_ie(&mut self) -> Res2IeW<ResStatusIeSpec> {
        Res2IeW::new(self, 2)
    }
    #[doc = "Bit 3 - RES3 Interrupt Enable"]
    #[inline(always)]
    pub fn res3_ie(&mut self) -> Res3IeW<ResStatusIeSpec> {
        Res3IeW::new(self, 3)
    }
}
#[doc = "Result Status Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`res_status_ie::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`res_status_ie::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ResStatusIeSpec;
impl crate::RegisterSpec for ResStatusIeSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`res_status_ie::R`](R) reader structure"]
impl crate::Readable for ResStatusIeSpec {}
#[doc = "`write(|w| ..)` method takes [`res_status_ie::W`](W) writer structure"]
impl crate::Writable for ResStatusIeSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RES_STATUS_IE to value 0"]
impl crate::Resettable for ResStatusIeSpec {}
