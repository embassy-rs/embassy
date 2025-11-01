#[doc = "Register `RX14MASK` reader"]
pub type R = crate::R<Rx14maskSpec>;
#[doc = "Register `RX14MASK` writer"]
pub type W = crate::W<Rx14maskSpec>;
#[doc = "Field `RX14M` reader - RX Buffer 14 Mask Bits"]
pub type Rx14mR = crate::FieldReader<u32>;
#[doc = "Field `RX14M` writer - RX Buffer 14 Mask Bits"]
pub type Rx14mW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - RX Buffer 14 Mask Bits"]
    #[inline(always)]
    pub fn rx14m(&self) -> Rx14mR {
        Rx14mR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - RX Buffer 14 Mask Bits"]
    #[inline(always)]
    pub fn rx14m(&mut self) -> Rx14mW<Rx14maskSpec> {
        Rx14mW::new(self, 0)
    }
}
#[doc = "Receive 14 Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`rx14mask::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rx14mask::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Rx14maskSpec;
impl crate::RegisterSpec for Rx14maskSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rx14mask::R`](R) reader structure"]
impl crate::Readable for Rx14maskSpec {}
#[doc = "`write(|w| ..)` method takes [`rx14mask::W`](W) writer structure"]
impl crate::Writable for Rx14maskSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RX14MASK to value 0"]
impl crate::Resettable for Rx14maskSpec {}
