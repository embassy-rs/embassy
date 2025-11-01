#[doc = "Register `RX15MASK` reader"]
pub type R = crate::R<Rx15maskSpec>;
#[doc = "Register `RX15MASK` writer"]
pub type W = crate::W<Rx15maskSpec>;
#[doc = "Field `RX15M` reader - RX Buffer 15 Mask Bits"]
pub type Rx15mR = crate::FieldReader<u32>;
#[doc = "Field `RX15M` writer - RX Buffer 15 Mask Bits"]
pub type Rx15mW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - RX Buffer 15 Mask Bits"]
    #[inline(always)]
    pub fn rx15m(&self) -> Rx15mR {
        Rx15mR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - RX Buffer 15 Mask Bits"]
    #[inline(always)]
    pub fn rx15m(&mut self) -> Rx15mW<Rx15maskSpec> {
        Rx15mW::new(self, 0)
    }
}
#[doc = "Receive 15 Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`rx15mask::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rx15mask::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Rx15maskSpec;
impl crate::RegisterSpec for Rx15maskSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rx15mask::R`](R) reader structure"]
impl crate::Readable for Rx15maskSpec {}
#[doc = "`write(|w| ..)` method takes [`rx15mask::W`](W) writer structure"]
impl crate::Writable for Rx15maskSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RX15MASK to value 0"]
impl crate::Resettable for Rx15maskSpec {}
