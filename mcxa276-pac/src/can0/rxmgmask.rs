#[doc = "Register `RXMGMASK` reader"]
pub type R = crate::R<RxmgmaskSpec>;
#[doc = "Register `RXMGMASK` writer"]
pub type W = crate::W<RxmgmaskSpec>;
#[doc = "Field `MG` reader - Global Mask for RX Message Buffers"]
pub type MgR = crate::FieldReader<u32>;
#[doc = "Field `MG` writer - Global Mask for RX Message Buffers"]
pub type MgW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Global Mask for RX Message Buffers"]
    #[inline(always)]
    pub fn mg(&self) -> MgR {
        MgR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Global Mask for RX Message Buffers"]
    #[inline(always)]
    pub fn mg(&mut self) -> MgW<RxmgmaskSpec> {
        MgW::new(self, 0)
    }
}
#[doc = "RX Message Buffers Global Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`rxmgmask::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rxmgmask::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RxmgmaskSpec;
impl crate::RegisterSpec for RxmgmaskSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rxmgmask::R`](R) reader structure"]
impl crate::Readable for RxmgmaskSpec {}
#[doc = "`write(|w| ..)` method takes [`rxmgmask::W`](W) writer structure"]
impl crate::Writable for RxmgmaskSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RXMGMASK to value 0"]
impl crate::Resettable for RxmgmaskSpec {}
