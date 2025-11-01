#[doc = "Register `PL2_PLMASK_LO` reader"]
pub type R = crate::R<Pl2PlmaskLoSpec>;
#[doc = "Register `PL2_PLMASK_LO` writer"]
pub type W = crate::W<Pl2PlmaskLoSpec>;
#[doc = "Field `Data_byte_3` reader - Data Byte 3"]
pub type DataByte3R = crate::FieldReader;
#[doc = "Field `Data_byte_3` writer - Data Byte 3"]
pub type DataByte3W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `Data_byte_2` reader - Data Byte 2"]
pub type DataByte2R = crate::FieldReader;
#[doc = "Field `Data_byte_2` writer - Data Byte 2"]
pub type DataByte2W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `Data_byte_1` reader - Data Byte 1"]
pub type DataByte1R = crate::FieldReader;
#[doc = "Field `Data_byte_1` writer - Data Byte 1"]
pub type DataByte1W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `Data_byte_0` reader - Data Byte 0"]
pub type DataByte0R = crate::FieldReader;
#[doc = "Field `Data_byte_0` writer - Data Byte 0"]
pub type DataByte0W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
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
impl W {
    #[doc = "Bits 0:7 - Data Byte 3"]
    #[inline(always)]
    pub fn data_byte_3(&mut self) -> DataByte3W<Pl2PlmaskLoSpec> {
        DataByte3W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data Byte 2"]
    #[inline(always)]
    pub fn data_byte_2(&mut self) -> DataByte2W<Pl2PlmaskLoSpec> {
        DataByte2W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data Byte 1"]
    #[inline(always)]
    pub fn data_byte_1(&mut self) -> DataByte1W<Pl2PlmaskLoSpec> {
        DataByte1W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data Byte 0"]
    #[inline(always)]
    pub fn data_byte_0(&mut self) -> DataByte0W<Pl2PlmaskLoSpec> {
        DataByte0W::new(self, 24)
    }
}
#[doc = "Pretended Networking Payload Low Filter 2 and Payload Low Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`pl2_plmask_lo::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pl2_plmask_lo::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pl2PlmaskLoSpec;
impl crate::RegisterSpec for Pl2PlmaskLoSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pl2_plmask_lo::R`](R) reader structure"]
impl crate::Readable for Pl2PlmaskLoSpec {}
#[doc = "`write(|w| ..)` method takes [`pl2_plmask_lo::W`](W) writer structure"]
impl crate::Writable for Pl2PlmaskLoSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PL2_PLMASK_LO to value 0"]
impl crate::Resettable for Pl2PlmaskLoSpec {}
