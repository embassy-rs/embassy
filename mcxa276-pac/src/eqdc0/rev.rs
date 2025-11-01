#[doc = "Register `REV` reader"]
pub type R = crate::R<RevSpec>;
#[doc = "Register `REV` writer"]
pub type W = crate::W<RevSpec>;
#[doc = "Field `REV` reader - REV"]
pub type RevR = crate::FieldReader<u16>;
#[doc = "Field `REV` writer - REV"]
pub type RevW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - REV"]
    #[inline(always)]
    pub fn rev(&self) -> RevR {
        RevR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:15 - REV"]
    #[inline(always)]
    pub fn rev(&mut self) -> RevW<RevSpec> {
        RevW::new(self, 0)
    }
}
#[doc = "Revolution Counter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`rev::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rev::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RevSpec;
impl crate::RegisterSpec for RevSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`rev::R`](R) reader structure"]
impl crate::Readable for RevSpec {}
#[doc = "`write(|w| ..)` method takes [`rev::W`](W) writer structure"]
impl crate::Writable for RevSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets REV to value 0"]
impl crate::Resettable for RevSpec {}
