#[doc = "Register `SM1PHASEDLY` reader"]
pub type R = crate::R<Sm1phasedlySpec>;
#[doc = "Register `SM1PHASEDLY` writer"]
pub type W = crate::W<Sm1phasedlySpec>;
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
    pub fn phasedly(&mut self) -> PhasedlyW<Sm1phasedlySpec> {
        PhasedlyW::new(self, 0)
    }
}
#[doc = "Phase Delay Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm1phasedly::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm1phasedly::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm1phasedlySpec;
impl crate::RegisterSpec for Sm1phasedlySpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm1phasedly::R`](R) reader structure"]
impl crate::Readable for Sm1phasedlySpec {}
#[doc = "`write(|w| ..)` method takes [`sm1phasedly::W`](W) writer structure"]
impl crate::Writable for Sm1phasedlySpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM1PHASEDLY to value 0"]
impl crate::Resettable for Sm1phasedlySpec {}
