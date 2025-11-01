#[doc = "Register `UPOS` reader"]
pub type R = crate::R<UposSpec>;
#[doc = "Register `UPOS` writer"]
pub type W = crate::W<UposSpec>;
#[doc = "Field `POS` reader - POS"]
pub type PosR = crate::FieldReader<u16>;
#[doc = "Field `POS` writer - POS"]
pub type PosW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - POS"]
    #[inline(always)]
    pub fn pos(&self) -> PosR {
        PosR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - POS"]
    #[inline(always)]
    pub fn pos(&mut self) -> PosW<UposSpec> {
        PosW::new(self, 0)
    }
}
#[doc = "Upper Position Counter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`upos::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`upos::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct UposSpec;
impl crate::RegisterSpec for UposSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`upos::R`](R) reader structure"]
impl crate::Readable for UposSpec {}
#[doc = "`write(|w| ..)` method takes [`upos::W`](W) writer structure"]
impl crate::Writable for UposSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets UPOS to value 0"]
impl crate::Resettable for UposSpec {}
