#[doc = "Register `MCONFIG_EXT` reader"]
pub type R = crate::R<MconfigExtSpec>;
#[doc = "Register `MCONFIG_EXT` writer"]
pub type W = crate::W<MconfigExtSpec>;
#[doc = "I3C CAS Delay After START\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum I3cCasDel {
    #[doc = "0: No delay"]
    NoDelay = 0,
    #[doc = "1: Increases SCL clock period by 1/2"]
    OneHalfClk = 1,
    #[doc = "2: Increases SCL clock period by 1"]
    OneClk = 2,
    #[doc = "3: Increases SCL clock period by 3/2"]
    OneAndOneHalfClk = 3,
}
impl From<I3cCasDel> for u8 {
    #[inline(always)]
    fn from(variant: I3cCasDel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for I3cCasDel {
    type Ux = u8;
}
impl crate::IsEnum for I3cCasDel {}
#[doc = "Field `I3C_CAS_DEL` reader - I3C CAS Delay After START"]
pub type I3cCasDelR = crate::FieldReader<I3cCasDel>;
impl I3cCasDelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> I3cCasDel {
        match self.bits {
            0 => I3cCasDel::NoDelay,
            1 => I3cCasDel::OneHalfClk,
            2 => I3cCasDel::OneClk,
            3 => I3cCasDel::OneAndOneHalfClk,
            _ => unreachable!(),
        }
    }
    #[doc = "No delay"]
    #[inline(always)]
    pub fn is_no_delay(&self) -> bool {
        *self == I3cCasDel::NoDelay
    }
    #[doc = "Increases SCL clock period by 1/2"]
    #[inline(always)]
    pub fn is_one_half_clk(&self) -> bool {
        *self == I3cCasDel::OneHalfClk
    }
    #[doc = "Increases SCL clock period by 1"]
    #[inline(always)]
    pub fn is_one_clk(&self) -> bool {
        *self == I3cCasDel::OneClk
    }
    #[doc = "Increases SCL clock period by 3/2"]
    #[inline(always)]
    pub fn is_one_and_one_half_clk(&self) -> bool {
        *self == I3cCasDel::OneAndOneHalfClk
    }
}
#[doc = "Field `I3C_CAS_DEL` writer - I3C CAS Delay After START"]
pub type I3cCasDelW<'a, REG> = crate::FieldWriter<'a, REG, 2, I3cCasDel, crate::Safe>;
impl<'a, REG> I3cCasDelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "No delay"]
    #[inline(always)]
    pub fn no_delay(self) -> &'a mut crate::W<REG> {
        self.variant(I3cCasDel::NoDelay)
    }
    #[doc = "Increases SCL clock period by 1/2"]
    #[inline(always)]
    pub fn one_half_clk(self) -> &'a mut crate::W<REG> {
        self.variant(I3cCasDel::OneHalfClk)
    }
    #[doc = "Increases SCL clock period by 1"]
    #[inline(always)]
    pub fn one_clk(self) -> &'a mut crate::W<REG> {
        self.variant(I3cCasDel::OneClk)
    }
    #[doc = "Increases SCL clock period by 3/2"]
    #[inline(always)]
    pub fn one_and_one_half_clk(self) -> &'a mut crate::W<REG> {
        self.variant(I3cCasDel::OneAndOneHalfClk)
    }
}
#[doc = "I3C CAS Delay After Repeated START\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum I3cCasrDel {
    #[doc = "0: No delay"]
    NoDelay = 0,
    #[doc = "1: Increases SCL clock period by 1/2"]
    OneHalfClk = 1,
    #[doc = "2: Increases SCL clock period by 1"]
    OneClk = 2,
    #[doc = "3: Increases SCL clock period by 1 1/2"]
    OneAndOneHalfClk = 3,
}
impl From<I3cCasrDel> for u8 {
    #[inline(always)]
    fn from(variant: I3cCasrDel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for I3cCasrDel {
    type Ux = u8;
}
impl crate::IsEnum for I3cCasrDel {}
#[doc = "Field `I3C_CASR_DEL` reader - I3C CAS Delay After Repeated START"]
pub type I3cCasrDelR = crate::FieldReader<I3cCasrDel>;
impl I3cCasrDelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> I3cCasrDel {
        match self.bits {
            0 => I3cCasrDel::NoDelay,
            1 => I3cCasrDel::OneHalfClk,
            2 => I3cCasrDel::OneClk,
            3 => I3cCasrDel::OneAndOneHalfClk,
            _ => unreachable!(),
        }
    }
    #[doc = "No delay"]
    #[inline(always)]
    pub fn is_no_delay(&self) -> bool {
        *self == I3cCasrDel::NoDelay
    }
    #[doc = "Increases SCL clock period by 1/2"]
    #[inline(always)]
    pub fn is_one_half_clk(&self) -> bool {
        *self == I3cCasrDel::OneHalfClk
    }
    #[doc = "Increases SCL clock period by 1"]
    #[inline(always)]
    pub fn is_one_clk(&self) -> bool {
        *self == I3cCasrDel::OneClk
    }
    #[doc = "Increases SCL clock period by 1 1/2"]
    #[inline(always)]
    pub fn is_one_and_one_half_clk(&self) -> bool {
        *self == I3cCasrDel::OneAndOneHalfClk
    }
}
#[doc = "Field `I3C_CASR_DEL` writer - I3C CAS Delay After Repeated START"]
pub type I3cCasrDelW<'a, REG> = crate::FieldWriter<'a, REG, 2, I3cCasrDel, crate::Safe>;
impl<'a, REG> I3cCasrDelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "No delay"]
    #[inline(always)]
    pub fn no_delay(self) -> &'a mut crate::W<REG> {
        self.variant(I3cCasrDel::NoDelay)
    }
    #[doc = "Increases SCL clock period by 1/2"]
    #[inline(always)]
    pub fn one_half_clk(self) -> &'a mut crate::W<REG> {
        self.variant(I3cCasrDel::OneHalfClk)
    }
    #[doc = "Increases SCL clock period by 1"]
    #[inline(always)]
    pub fn one_clk(self) -> &'a mut crate::W<REG> {
        self.variant(I3cCasrDel::OneClk)
    }
    #[doc = "Increases SCL clock period by 1 1/2"]
    #[inline(always)]
    pub fn one_and_one_half_clk(self) -> &'a mut crate::W<REG> {
        self.variant(I3cCasrDel::OneAndOneHalfClk)
    }
}
impl R {
    #[doc = "Bits 16:17 - I3C CAS Delay After START"]
    #[inline(always)]
    pub fn i3c_cas_del(&self) -> I3cCasDelR {
        I3cCasDelR::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bits 18:19 - I3C CAS Delay After Repeated START"]
    #[inline(always)]
    pub fn i3c_casr_del(&self) -> I3cCasrDelR {
        I3cCasrDelR::new(((self.bits >> 18) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 16:17 - I3C CAS Delay After START"]
    #[inline(always)]
    pub fn i3c_cas_del(&mut self) -> I3cCasDelW<MconfigExtSpec> {
        I3cCasDelW::new(self, 16)
    }
    #[doc = "Bits 18:19 - I3C CAS Delay After Repeated START"]
    #[inline(always)]
    pub fn i3c_casr_del(&mut self) -> I3cCasrDelW<MconfigExtSpec> {
        I3cCasrDelW::new(self, 18)
    }
}
#[doc = "Controller Extended Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`mconfig_ext::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mconfig_ext::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MconfigExtSpec;
impl crate::RegisterSpec for MconfigExtSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mconfig_ext::R`](R) reader structure"]
impl crate::Readable for MconfigExtSpec {}
#[doc = "`write(|w| ..)` method takes [`mconfig_ext::W`](W) writer structure"]
impl crate::Writable for MconfigExtSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MCONFIG_EXT to value 0"]
impl crate::Resettable for MconfigExtSpec {}
