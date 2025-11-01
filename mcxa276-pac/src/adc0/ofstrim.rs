#[doc = "Register `OFSTRIM` reader"]
pub type R = crate::R<OfstrimSpec>;
#[doc = "Register `OFSTRIM` writer"]
pub type W = crate::W<OfstrimSpec>;
#[doc = "Field `OFSTRIM` reader - Trim for Offset"]
pub type OfstrimR = crate::FieldReader<u16>;
#[doc = "Field `OFSTRIM` writer - Trim for Offset"]
pub type OfstrimW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
impl R {
    #[doc = "Bits 0:9 - Trim for Offset"]
    #[inline(always)]
    pub fn ofstrim(&self) -> OfstrimR {
        OfstrimR::new((self.bits & 0x03ff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:9 - Trim for Offset"]
    #[inline(always)]
    pub fn ofstrim(&mut self) -> OfstrimW<OfstrimSpec> {
        OfstrimW::new(self, 0)
    }
}
#[doc = "Offset Trim Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ofstrim::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ofstrim::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct OfstrimSpec;
impl crate::RegisterSpec for OfstrimSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ofstrim::R`](R) reader structure"]
impl crate::Readable for OfstrimSpec {}
#[doc = "`write(|w| ..)` method takes [`ofstrim::W`](W) writer structure"]
impl crate::Writable for OfstrimSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets OFSTRIM to value 0"]
impl crate::Resettable for OfstrimSpec {}
