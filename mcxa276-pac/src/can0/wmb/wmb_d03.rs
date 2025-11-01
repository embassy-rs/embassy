#[doc = "Register `WMB_D03` reader"]
pub type R = crate::R<WmbD03Spec>;
#[doc = "Field `Data_byte_3` reader - Data Byte 3"]
pub type DataByte3R = crate::FieldReader;
#[doc = "Field `Data_byte_2` reader - Data Byte 2"]
pub type DataByte2R = crate::FieldReader;
#[doc = "Field `Data_byte_1` reader - Data Byte 1"]
pub type DataByte1R = crate::FieldReader;
#[doc = "Field `Data_byte_0` reader - Data Byte 0"]
pub type DataByte0R = crate::FieldReader;
impl R {
    #[doc = "Bits 0:7 - Data Byte 3"]
    #[inline(always)]
    pub fn data_byte_3(&self) -> DataByte3R {
        DataByte3R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data Byte 2"]
    #[inline(always)]
    pub fn data_byte_2(&self) -> DataByte2R {
        DataByte2R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data Byte 1"]
    #[inline(always)]
    pub fn data_byte_1(&self) -> DataByte1R {
        DataByte1R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data Byte 0"]
    #[inline(always)]
    pub fn data_byte_0(&self) -> DataByte0R {
        DataByte0R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
#[doc = "Wake-Up Message Buffer for Data 0-3\n\nYou can [`read`](crate::Reg::read) this register and get [`wmb_d03::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct WmbD03Spec;
impl crate::RegisterSpec for WmbD03Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`wmb_d03::R`](R) reader structure"]
impl crate::Readable for WmbD03Spec {}
#[doc = "`reset()` method sets WMB_D03 to value 0"]
impl crate::Resettable for WmbD03Spec {}
