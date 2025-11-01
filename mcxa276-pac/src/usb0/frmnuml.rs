#[doc = "Register `FRMNUML` reader"]
pub type R = crate::R<FrmnumlSpec>;
#[doc = "Field `FRM` reader - Frame Number, Bits 0-7"]
pub type FrmR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:7 - Frame Number, Bits 0-7"]
    #[inline(always)]
    pub fn frm(&self) -> FrmR {
        FrmR::new(self.bits)
    }
}
#[doc = "Frame Number Register Low\n\nYou can [`read`](crate::Reg::read) this register and get [`frmnuml::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FrmnumlSpec;
impl crate::RegisterSpec for FrmnumlSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`frmnuml::R`](R) reader structure"]
impl crate::Readable for FrmnumlSpec {}
#[doc = "`reset()` method sets FRMNUML to value 0"]
impl crate::Resettable for FrmnumlSpec {}
