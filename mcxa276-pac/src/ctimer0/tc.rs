#[doc = "Register `TC` reader"]
pub type R = crate::R<TcSpec>;
#[doc = "Register `TC` writer"]
pub type W = crate::W<TcSpec>;
#[doc = "Field `TCVAL` reader - Timer Counter Value"]
pub type TcvalR = crate::FieldReader<u32>;
#[doc = "Field `TCVAL` writer - Timer Counter Value"]
pub type TcvalW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Timer Counter Value"]
    #[inline(always)]
    pub fn tcval(&self) -> TcvalR {
        TcvalR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Timer Counter Value"]
    #[inline(always)]
    pub fn tcval(&mut self) -> TcvalW<TcSpec> {
        TcvalW::new(self, 0)
    }
}
#[doc = "Timer Counter\n\nYou can [`read`](crate::Reg::read) this register and get [`tc::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tc::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TcSpec;
impl crate::RegisterSpec for TcSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`tc::R`](R) reader structure"]
impl crate::Readable for TcSpec {}
#[doc = "`write(|w| ..)` method takes [`tc::W`](W) writer structure"]
impl crate::Writable for TcSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TC to value 0"]
impl crate::Resettable for TcSpec {}
