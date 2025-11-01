#[doc = "Register `USBFRMADJUST` reader"]
pub type R = crate::R<UsbfrmadjustSpec>;
#[doc = "Register `USBFRMADJUST` writer"]
pub type W = crate::W<UsbfrmadjustSpec>;
#[doc = "Field `ADJ` reader - Frame Adjustment"]
pub type AdjR = crate::FieldReader;
#[doc = "Field `ADJ` writer - Frame Adjustment"]
pub type AdjW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Frame Adjustment"]
    #[inline(always)]
    pub fn adj(&self) -> AdjR {
        AdjR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:7 - Frame Adjustment"]
    #[inline(always)]
    pub fn adj(&mut self) -> AdjW<UsbfrmadjustSpec> {
        AdjW::new(self, 0)
    }
}
#[doc = "Frame Adjust\n\nYou can [`read`](crate::Reg::read) this register and get [`usbfrmadjust::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`usbfrmadjust::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct UsbfrmadjustSpec;
impl crate::RegisterSpec for UsbfrmadjustSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`usbfrmadjust::R`](R) reader structure"]
impl crate::Readable for UsbfrmadjustSpec {}
#[doc = "`write(|w| ..)` method takes [`usbfrmadjust::W`](W) writer structure"]
impl crate::Writable for UsbfrmadjustSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets USBFRMADJUST to value 0"]
impl crate::Resettable for UsbfrmadjustSpec {}
