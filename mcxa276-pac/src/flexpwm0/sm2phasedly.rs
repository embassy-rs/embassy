#[doc = "Register `SM2PHASEDLY` reader"]
pub type R = crate::R<Sm2phasedlySpec>;
#[doc = "Register `SM2PHASEDLY` writer"]
pub type W = crate::W<Sm2phasedlySpec>;
#[doc = "Field `PHASEDLY` reader - Initial Count Register Bits"]
pub type PhasedlyR = crate::FieldReader<u16>;
#[doc = "Field `PHASEDLY` writer - Initial Count Register Bits"]
pub type PhasedlyW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Initial Count Register Bits"]
    #[inline(always)]
    pub fn phasedly(&self) -> PhasedlyR {
        PhasedlyR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - Initial Count Register Bits"]
    #[inline(always)]
    pub fn phasedly(&mut self) -> PhasedlyW<Sm2phasedlySpec> {
        PhasedlyW::new(self, 0)
    }
}
#[doc = "Phase Delay Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2phasedly::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2phasedly::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm2phasedlySpec;
impl crate::RegisterSpec for Sm2phasedlySpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm2phasedly::R`](R) reader structure"]
impl crate::Readable for Sm2phasedlySpec {}
#[doc = "`write(|w| ..)` method takes [`sm2phasedly::W`](W) writer structure"]
impl crate::Writable for Sm2phasedlySpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM2PHASEDLY to value 0"]
impl crate::Resettable for Sm2phasedlySpec {}
