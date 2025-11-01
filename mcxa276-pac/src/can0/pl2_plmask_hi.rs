#[doc = "Register `PL2_PLMASK_HI` reader"]
pub type R = crate::R<Pl2PlmaskHiSpec>;
#[doc = "Register `PL2_PLMASK_HI` writer"]
pub type W = crate::W<Pl2PlmaskHiSpec>;
#[doc = "Field `Data_byte_7` reader - Data Byte 7"]
pub type DataByte7R = crate::FieldReader;
#[doc = "Field `Data_byte_7` writer - Data Byte 7"]
pub type DataByte7W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `Data_byte_6` reader - Data Byte 6"]
pub type DataByte6R = crate::FieldReader;
#[doc = "Field `Data_byte_6` writer - Data Byte 6"]
pub type DataByte6W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `Data_byte_5` reader - Data Byte 5"]
pub type DataByte5R = crate::FieldReader;
#[doc = "Field `Data_byte_5` writer - Data Byte 5"]
pub type DataByte5W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `Data_byte_4` reader - Data Byte 4"]
pub type DataByte4R = crate::FieldReader;
#[doc = "Field `Data_byte_4` writer - Data Byte 4"]
pub type DataByte4W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
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
impl W {
    #[doc = "Bits 0:7 - Data Byte 7"]
    #[inline(always)]
    pub fn data_byte_7(&mut self) -> DataByte7W<Pl2PlmaskHiSpec> {
        DataByte7W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data Byte 6"]
    #[inline(always)]
    pub fn data_byte_6(&mut self) -> DataByte6W<Pl2PlmaskHiSpec> {
        DataByte6W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data Byte 5"]
    #[inline(always)]
    pub fn data_byte_5(&mut self) -> DataByte5W<Pl2PlmaskHiSpec> {
        DataByte5W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data Byte 4"]
    #[inline(always)]
    pub fn data_byte_4(&mut self) -> DataByte4W<Pl2PlmaskHiSpec> {
        DataByte4W::new(self, 24)
    }
}
#[doc = "Pretended Networking Payload High Filter 2 and Payload High Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`pl2_plmask_hi::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pl2_plmask_hi::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pl2PlmaskHiSpec;
impl crate::RegisterSpec for Pl2PlmaskHiSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pl2_plmask_hi::R`](R) reader structure"]
impl crate::Readable for Pl2PlmaskHiSpec {}
#[doc = "`write(|w| ..)` method takes [`pl2_plmask_hi::W`](W) writer structure"]
impl crate::Writable for Pl2PlmaskHiSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PL2_PLMASK_HI to value 0"]
impl crate::Resettable for Pl2PlmaskHiSpec {}
