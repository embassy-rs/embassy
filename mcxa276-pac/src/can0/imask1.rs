#[doc = "Register `IMASK1` reader"]
pub type R = crate::R<Imask1Spec>;
#[doc = "Register `IMASK1` writer"]
pub type W = crate::W<Imask1Spec>;
#[doc = "Field `BUF31TO0M` reader - Buffer MBi Mask"]
pub type Buf31to0mR = crate::FieldReader<u32>;
#[doc = "Field `BUF31TO0M` writer - Buffer MBi Mask"]
pub type Buf31to0mW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Buffer MBi Mask"]
    #[inline(always)]
    pub fn buf31to0m(&self) -> Buf31to0mR {
        Buf31to0mR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Buffer MBi Mask"]
    #[inline(always)]
    pub fn buf31to0m(&mut self) -> Buf31to0mW<Imask1Spec> {
        Buf31to0mW::new(self, 0)
    }
}
#[doc = "Interrupt Masks 1\n\nYou can [`read`](crate::Reg::read) this register and get [`imask1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`imask1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Imask1Spec;
impl crate::RegisterSpec for Imask1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`imask1::R`](R) reader structure"]
impl crate::Readable for Imask1Spec {}
#[doc = "`write(|w| ..)` method takes [`imask1::W`](W) writer structure"]
impl crate::Writable for Imask1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets IMASK1 to value 0"]
impl crate::Resettable for Imask1Spec {}
