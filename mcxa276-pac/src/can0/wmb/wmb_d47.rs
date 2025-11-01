#[doc = "Register `WMB_D47` reader"]
pub type R = crate::R<WmbD47Spec>;
#[doc = "Field `Data_byte_7` reader - Data Byte 7"]
pub type DataByte7R = crate::FieldReader;
#[doc = "Field `Data_byte_6` reader - Data Byte 6"]
pub type DataByte6R = crate::FieldReader;
#[doc = "Field `Data_byte_5` reader - Data Byte 5"]
pub type DataByte5R = crate::FieldReader;
#[doc = "Field `Data_byte_4` reader - Data Byte 4"]
pub type DataByte4R = crate::FieldReader;
impl R {
    #[doc = "Bits 0:7 - Data Byte 7"]
    #[inline(always)]
    pub fn data_byte_7(&self) -> DataByte7R {
        DataByte7R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data Byte 6"]
    #[inline(always)]
    pub fn data_byte_6(&self) -> DataByte6R {
        DataByte6R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data Byte 5"]
    #[inline(always)]
    pub fn data_byte_5(&self) -> DataByte5R {
        DataByte5R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data Byte 4"]
    #[inline(always)]
    pub fn data_byte_4(&self) -> DataByte4R {
        DataByte4R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
#[doc = "Wake-Up Message Buffer Register Data 4-7\n\nYou can [`read`](crate::Reg::read) this register and get [`wmb_d47::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct WmbD47Spec;
impl crate::RegisterSpec for WmbD47Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`wmb_d47::R`](R) reader structure"]
impl crate::Readable for WmbD47Spec {}
#[doc = "`reset()` method sets WMB_D47 to value 0"]
impl crate::Resettable for WmbD47Spec {}
