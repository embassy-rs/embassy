#[doc = "Register `TCD_SADDR` reader"]
pub type R = crate::R<TcdSaddrSpec>;
#[doc = "Register `TCD_SADDR` writer"]
pub type W = crate::W<TcdSaddrSpec>;
#[doc = "Field `SADDR` reader - Source Address"]
pub type SaddrR = crate::FieldReader<u32>;
#[doc = "Field `SADDR` writer - Source Address"]
pub type SaddrW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Source Address"]
    #[inline(always)]
    pub fn saddr(&self) -> SaddrR {
        SaddrR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Source Address"]
    #[inline(always)]
    pub fn saddr(&mut self) -> SaddrW<TcdSaddrSpec> {
        SaddrW::new(self, 0)
    }
}
#[doc = "TCD Source Address\n\nYou can [`read`](crate::Reg::read) this register and get [`tcd_saddr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcd_saddr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TcdSaddrSpec;
impl crate::RegisterSpec for TcdSaddrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`tcd_saddr::R`](R) reader structure"]
impl crate::Readable for TcdSaddrSpec {}
#[doc = "`write(|w| ..)` method takes [`tcd_saddr::W`](W) writer structure"]
impl crate::Writable for TcdSaddrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TCD_SADDR to value 0"]
impl crate::Resettable for TcdSaddrSpec {}
