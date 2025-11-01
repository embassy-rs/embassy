#[doc = "Register `ERFFEL[%s]` reader"]
pub type R = crate::R<ErffelSpec>;
#[doc = "Register `ERFFEL[%s]` writer"]
pub type W = crate::W<ErffelSpec>;
#[doc = "Field `FEL` reader - Filter Element Bits"]
pub type FelR = crate::FieldReader<u32>;
#[doc = "Field `FEL` writer - Filter Element Bits"]
pub type FelW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Filter Element Bits"]
    #[inline(always)]
    pub fn fel(&self) -> FelR {
        FelR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Filter Element Bits"]
    #[inline(always)]
    pub fn fel(&mut self) -> FelW<ErffelSpec> {
        FelW::new(self, 0)
    }
}
#[doc = "Enhanced RX FIFO Filter Element\n\nYou can [`read`](crate::Reg::read) this register and get [`erffel::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`erffel::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ErffelSpec;
impl crate::RegisterSpec for ErffelSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`erffel::R`](R) reader structure"]
impl crate::Readable for ErffelSpec {}
#[doc = "`write(|w| ..)` method takes [`erffel::W`](W) writer structure"]
impl crate::Writable for ErffelSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ERFFEL[%s] to value 0"]
impl crate::Resettable for ErffelSpec {}
