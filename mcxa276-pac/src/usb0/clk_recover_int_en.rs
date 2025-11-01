#[doc = "Register `CLK_RECOVER_INT_EN` reader"]
pub type R = crate::R<ClkRecoverIntEnSpec>;
#[doc = "Register `CLK_RECOVER_INT_EN` writer"]
pub type W = crate::W<ClkRecoverIntEnSpec>;
#[doc = "Overflow error interrupt enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OvfErrorEn {
    #[doc = "0: The interrupt is masked"]
    MaskOvfErrInt = 0,
    #[doc = "1: The interrupt is enabled"]
    EnOvfErrInt = 1,
}
impl From<OvfErrorEn> for bool {
    #[inline(always)]
    fn from(variant: OvfErrorEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OVF_ERROR_EN` reader - Overflow error interrupt enable"]
pub type OvfErrorEnR = crate::BitReader<OvfErrorEn>;
impl OvfErrorEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> OvfErrorEn {
        match self.bits {
            false => OvfErrorEn::MaskOvfErrInt,
            true => OvfErrorEn::EnOvfErrInt,
        }
    }
    #[doc = "The interrupt is masked"]
    #[inline(always)]
    pub fn is_mask_ovf_err_int(&self) -> bool {
        *self == OvfErrorEn::MaskOvfErrInt
    }
    #[doc = "The interrupt is enabled"]
    #[inline(always)]
    pub fn is_en_ovf_err_int(&self) -> bool {
        *self == OvfErrorEn::EnOvfErrInt
    }
}
#[doc = "Field `OVF_ERROR_EN` writer - Overflow error interrupt enable"]
pub type OvfErrorEnW<'a, REG> = crate::BitWriter<'a, REG, OvfErrorEn>;
impl<'a, REG> OvfErrorEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The interrupt is masked"]
    #[inline(always)]
    pub fn mask_ovf_err_int(self) -> &'a mut crate::W<REG> {
        self.variant(OvfErrorEn::MaskOvfErrInt)
    }
    #[doc = "The interrupt is enabled"]
    #[inline(always)]
    pub fn en_ovf_err_int(self) -> &'a mut crate::W<REG> {
        self.variant(OvfErrorEn::EnOvfErrInt)
    }
}
impl R {
    #[doc = "Bit 4 - Overflow error interrupt enable"]
    #[inline(always)]
    pub fn ovf_error_en(&self) -> OvfErrorEnR {
        OvfErrorEnR::new(((self.bits >> 4) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 4 - Overflow error interrupt enable"]
    #[inline(always)]
    pub fn ovf_error_en(&mut self) -> OvfErrorEnW<ClkRecoverIntEnSpec> {
        OvfErrorEnW::new(self, 4)
    }
}
#[doc = "Clock Recovery Combined Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`clk_recover_int_en::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`clk_recover_int_en::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ClkRecoverIntEnSpec;
impl crate::RegisterSpec for ClkRecoverIntEnSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`clk_recover_int_en::R`](R) reader structure"]
impl crate::Readable for ClkRecoverIntEnSpec {}
#[doc = "`write(|w| ..)` method takes [`clk_recover_int_en::W`](W) writer structure"]
impl crate::Writable for ClkRecoverIntEnSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CLK_RECOVER_INT_EN to value 0x10"]
impl crate::Resettable for ClkRecoverIntEnSpec {
    const RESET_VALUE: u8 = 0x10;
}
