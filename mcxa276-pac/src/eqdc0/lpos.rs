#[doc = "Register `LPOS` reader"]
pub type R = crate::R<LposSpec>;
#[doc = "Register `LPOS` writer"]
pub type W = crate::W<LposSpec>;
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
    pub fn pos(&mut self) -> PosW<LposSpec> {
        PosW::new(self, 0)
    }
}
#[doc = "Lower Position Counter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lpos::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpos::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LposSpec;
impl crate::RegisterSpec for LposSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`lpos::R`](R) reader structure"]
impl crate::Readable for LposSpec {}
#[doc = "`write(|w| ..)` method takes [`lpos::W`](W) writer structure"]
impl crate::Writable for LposSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LPOS to value 0"]
impl crate::Resettable for LposSpec {}
