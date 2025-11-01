#[doc = "Register `TCD_DADDR` reader"]
pub type R = crate::R<TcdDaddrSpec>;
#[doc = "Register `TCD_DADDR` writer"]
pub type W = crate::W<TcdDaddrSpec>;
#[doc = "Field `DADDR` reader - Destination Address"]
pub type DaddrR = crate::FieldReader<u32>;
#[doc = "Field `DADDR` writer - Destination Address"]
pub type DaddrW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Destination Address"]
    #[inline(always)]
    pub fn daddr(&self) -> DaddrR {
        DaddrR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Destination Address"]
    #[inline(always)]
    pub fn daddr(&mut self) -> DaddrW<TcdDaddrSpec> {
        DaddrW::new(self, 0)
    }
}
#[doc = "TCD Destination Address\n\nYou can [`read`](crate::Reg::read) this register and get [`tcd_daddr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tcd_daddr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TcdDaddrSpec;
impl crate::RegisterSpec for TcdDaddrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`tcd_daddr::R`](R) reader structure"]
impl crate::Readable for TcdDaddrSpec {}
#[doc = "`write(|w| ..)` method takes [`tcd_daddr::W`](W) writer structure"]
impl crate::Writable for TcdDaddrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TCD_DADDR to value 0"]
impl crate::Resettable for TcdDaddrSpec {}
