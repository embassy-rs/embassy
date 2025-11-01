#[doc = "Register `FRMNUMH` reader"]
pub type R = crate::R<FrmnumhSpec>;
#[doc = "Field `FRM` reader - Frame Number, Bits 8-10"]
pub type FrmR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:2 - Frame Number, Bits 8-10"]
    #[inline(always)]
    pub fn frm(&self) -> FrmR {
        FrmR::new(self.bits & 7)
    }
}
#[doc = "Frame Number Register High\n\nYou can [`read`](crate::Reg::read) this register and get [`frmnumh::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FrmnumhSpec;
impl crate::RegisterSpec for FrmnumhSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`frmnumh::R`](R) reader structure"]
impl crate::Readable for FrmnumhSpec {}
#[doc = "`reset()` method sets FRMNUMH to value 0"]
impl crate::Resettable for FrmnumhSpec {}
