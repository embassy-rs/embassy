#[doc = "Register `MCFGR3` reader"]
pub type R = crate::R<Mcfgr3Spec>;
#[doc = "Register `MCFGR3` writer"]
pub type W = crate::W<Mcfgr3Spec>;
#[doc = "Field `PINLOW` reader - Pin Low Timeout"]
pub type PinlowR = crate::FieldReader<u16>;
#[doc = "Field `PINLOW` writer - Pin Low Timeout"]
pub type PinlowW<'a, REG> = crate::FieldWriter<'a, REG, 12, u16>;
impl R {
    #[doc = "Bits 8:19 - Pin Low Timeout"]
    #[inline(always)]
    pub fn pinlow(&self) -> PinlowR {
        PinlowR::new(((self.bits >> 8) & 0x0fff) as u16)
    }
}
impl W {
    #[doc = "Bits 8:19 - Pin Low Timeout"]
    #[inline(always)]
    pub fn pinlow(&mut self) -> PinlowW<Mcfgr3Spec> {
        PinlowW::new(self, 8)
    }
}
#[doc = "Controller Configuration 3\n\nYou can [`read`](crate::Reg::read) this register and get [`mcfgr3::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mcfgr3::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mcfgr3Spec;
impl crate::RegisterSpec for Mcfgr3Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mcfgr3::R`](R) reader structure"]
impl crate::Readable for Mcfgr3Spec {}
#[doc = "`write(|w| ..)` method takes [`mcfgr3::W`](W) writer structure"]
impl crate::Writable for Mcfgr3Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MCFGR3 to value 0"]
impl crate::Resettable for Mcfgr3Spec {}
