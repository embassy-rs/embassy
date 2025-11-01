#[doc = "Register `RETURN` reader"]
pub type R = crate::R<ReturnSpec>;
#[doc = "Register `RETURN` writer"]
pub type W = crate::W<ReturnSpec>;
#[doc = "Field `RET` reader - Return Value"]
pub type RetR = crate::FieldReader<u32>;
#[doc = "Field `RET` writer - Return Value"]
pub type RetW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Return Value"]
    #[inline(always)]
    pub fn ret(&self) -> RetR {
        RetR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Return Value"]
    #[inline(always)]
    pub fn ret(&mut self) -> RetW<ReturnSpec> {
        RetW::new(self, 0)
    }
}
#[doc = "Return Value\n\nYou can [`read`](crate::Reg::read) this register and get [`return_::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`return_::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ReturnSpec;
impl crate::RegisterSpec for ReturnSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`return_::R`](R) reader structure"]
impl crate::Readable for ReturnSpec {}
#[doc = "`write(|w| ..)` method takes [`return_::W`](W) writer structure"]
impl crate::Writable for ReturnSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RETURN to value 0"]
impl crate::Resettable for ReturnSpec {}
