#[doc = "Register `CTRL_0` reader"]
pub type R = crate::R<Ctrl0Spec>;
#[doc = "Register `CTRL_0` writer"]
pub type W = crate::W<Ctrl0Spec>;
#[doc = "Field `WRITE_INDEX` reader - Write Index"]
pub type WriteIndexR = crate::FieldReader;
#[doc = "Field `WRITE_INDEX` writer - Write Index"]
pub type WriteIndexW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `RESERVED15` reader - Reserved for Future Use"]
pub type Reserved15R = crate::FieldReader;
#[doc = "Field `WR_EN_0` reader - Write Enable 0"]
pub type WrEn0R = crate::FieldReader;
#[doc = "Field `WR_EN_0` writer - Write Enable 0"]
pub type WrEn0W<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Soft reset for the core reset (SFR configuration will be preseved).This register reads as 0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SftRst {
    #[doc = "0: No effect"]
    Disable = 0,
    #[doc = "1: Triggers the soft reset"]
    Enable = 1,
}
impl From<SftRst> for bool {
    #[inline(always)]
    fn from(variant: SftRst) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SFT_RST` reader - Soft reset for the core reset (SFR configuration will be preseved).This register reads as 0"]
pub type SftRstR = crate::BitReader<SftRst>;
impl SftRstR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SftRst {
        match self.bits {
            false => SftRst::Disable,
            true => SftRst::Enable,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == SftRst::Disable
    }
    #[doc = "Triggers the soft reset"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == SftRst::Enable
    }
}
#[doc = "Field `SFT_RST` writer - Soft reset for the core reset (SFR configuration will be preseved).This register reads as 0"]
pub type SftRstW<'a, REG> = crate::BitWriter<'a, REG, SftRst>;
impl<'a, REG> SftRstW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(SftRst::Disable)
    }
    #[doc = "Triggers the soft reset"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(SftRst::Enable)
    }
}
#[doc = "Field `RESERVED31` reader - Reserved for Future Use"]
pub type Reserved31R = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:7 - Write Index"]
    #[inline(always)]
    pub fn write_index(&self) -> WriteIndexR {
        WriteIndexR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Reserved for Future Use"]
    #[inline(always)]
    pub fn reserved15(&self) -> Reserved15R {
        Reserved15R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:17 - Write Enable 0"]
    #[inline(always)]
    pub fn wr_en_0(&self) -> WrEn0R {
        WrEn0R::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bit 18 - Soft reset for the core reset (SFR configuration will be preseved).This register reads as 0"]
    #[inline(always)]
    pub fn sft_rst(&self) -> SftRstR {
        SftRstR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bits 19:31 - Reserved for Future Use"]
    #[inline(always)]
    pub fn reserved31(&self) -> Reserved31R {
        Reserved31R::new(((self.bits >> 19) & 0x1fff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:7 - Write Index"]
    #[inline(always)]
    pub fn write_index(&mut self) -> WriteIndexW<Ctrl0Spec> {
        WriteIndexW::new(self, 0)
    }
    #[doc = "Bits 16:17 - Write Enable 0"]
    #[inline(always)]
    pub fn wr_en_0(&mut self) -> WrEn0W<Ctrl0Spec> {
        WrEn0W::new(self, 16)
    }
    #[doc = "Bit 18 - Soft reset for the core reset (SFR configuration will be preseved).This register reads as 0"]
    #[inline(always)]
    pub fn sft_rst(&mut self) -> SftRstW<Ctrl0Spec> {
        SftRstW::new(self, 18)
    }
}
#[doc = "Control Register 0 SFR\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl_0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl_0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ctrl0Spec;
impl crate::RegisterSpec for Ctrl0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ctrl_0::R`](R) reader structure"]
impl crate::Readable for Ctrl0Spec {}
#[doc = "`write(|w| ..)` method takes [`ctrl_0::W`](W) writer structure"]
impl crate::Writable for Ctrl0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CTRL_0 to value 0x0002_0000"]
impl crate::Resettable for Ctrl0Spec {
    const RESET_VALUE: u32 = 0x0002_0000;
}
