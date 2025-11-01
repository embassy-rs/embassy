#[doc = "Register `PKC_ULEN` reader"]
pub type R = crate::R<PkcUlenSpec>;
#[doc = "Register `PKC_ULEN` writer"]
pub type W = crate::W<PkcUlenSpec>;
#[doc = "Field `LEN` reader - Length of universal pointer calculation"]
pub type LenR = crate::FieldReader;
#[doc = "Field `LEN` writer - Length of universal pointer calculation"]
pub type LenW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Length of universal pointer calculation"]
    #[inline(always)]
    pub fn len(&self) -> LenR {
        LenR::new((self.bits & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Length of universal pointer calculation"]
    #[inline(always)]
    pub fn len(&mut self) -> LenW<PkcUlenSpec> {
        LenW::new(self, 0)
    }
}
#[doc = "Universal pointer length\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_ulen::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_ulen::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcUlenSpec;
impl crate::RegisterSpec for PkcUlenSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_ulen::R`](R) reader structure"]
impl crate::Readable for PkcUlenSpec {}
#[doc = "`write(|w| ..)` method takes [`pkc_ulen::W`](W) writer structure"]
impl crate::Writable for PkcUlenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_ULEN to value 0"]
impl crate::Resettable for PkcUlenSpec {}
