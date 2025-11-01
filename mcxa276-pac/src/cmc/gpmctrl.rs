#[doc = "Register `GPMCTRL` reader"]
pub type R = crate::R<GpmctrlSpec>;
#[doc = "Register `GPMCTRL` writer"]
pub type W = crate::W<GpmctrlSpec>;
#[doc = "Field `LPMODE` reader - Low-Power Mode"]
pub type LpmodeR = crate::FieldReader;
#[doc = "Field `LPMODE` writer - Low-Power Mode"]
pub type LpmodeW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - Low-Power Mode"]
    #[inline(always)]
    pub fn lpmode(&self) -> LpmodeR {
        LpmodeR::new((self.bits & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Low-Power Mode"]
    #[inline(always)]
    pub fn lpmode(&mut self) -> LpmodeW<GpmctrlSpec> {
        LpmodeW::new(self, 0)
    }
}
#[doc = "Global Power Mode Control\n\nYou can [`read`](crate::Reg::read) this register and get [`gpmctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gpmctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct GpmctrlSpec;
impl crate::RegisterSpec for GpmctrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`gpmctrl::R`](R) reader structure"]
impl crate::Readable for GpmctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`gpmctrl::W`](W) writer structure"]
impl crate::Writable for GpmctrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets GPMCTRL to value 0"]
impl crate::Resettable for GpmctrlSpec {}
