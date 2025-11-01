#[doc = "Register `PL1_LO` reader"]
pub type R = crate::R<Pl1LoSpec>;
#[doc = "Register `PL1_LO` writer"]
pub type W = crate::W<Pl1LoSpec>;
#[doc = "Field `Data_byte_3` reader - Data byte 3"]
pub type DataByte3R = crate::FieldReader;
#[doc = "Field `Data_byte_3` writer - Data byte 3"]
pub type DataByte3W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `Data_byte_2` reader - Data byte 2"]
pub type DataByte2R = crate::FieldReader;
#[doc = "Field `Data_byte_2` writer - Data byte 2"]
pub type DataByte2W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `Data_byte_1` reader - Data byte 1"]
pub type DataByte1R = crate::FieldReader;
#[doc = "Field `Data_byte_1` writer - Data byte 1"]
pub type DataByte1W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `Data_byte_0` reader - Data byte 0"]
pub type DataByte0R = crate::FieldReader;
#[doc = "Field `Data_byte_0` writer - Data byte 0"]
pub type DataByte0W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Data byte 3"]
    #[inline(always)]
    pub fn data_byte_3(&self) -> DataByte3R {
        DataByte3R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Data byte 2"]
    #[inline(always)]
    pub fn data_byte_2(&self) -> DataByte2R {
        DataByte2R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Data byte 1"]
    #[inline(always)]
    pub fn data_byte_1(&self) -> DataByte1R {
        DataByte1R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Data byte 0"]
    #[inline(always)]
    pub fn data_byte_0(&self) -> DataByte0R {
        DataByte0R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Data byte 3"]
    #[inline(always)]
    pub fn data_byte_3(&mut self) -> DataByte3W<Pl1LoSpec> {
        DataByte3W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Data byte 2"]
    #[inline(always)]
    pub fn data_byte_2(&mut self) -> DataByte2W<Pl1LoSpec> {
        DataByte2W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Data byte 1"]
    #[inline(always)]
    pub fn data_byte_1(&mut self) -> DataByte1W<Pl1LoSpec> {
        DataByte1W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Data byte 0"]
    #[inline(always)]
    pub fn data_byte_0(&mut self) -> DataByte0W<Pl1LoSpec> {
        DataByte0W::new(self, 24)
    }
}
#[doc = "Pretended Networking Payload Low Filter 1\n\nYou can [`read`](crate::Reg::read) this register and get [`pl1_lo::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pl1_lo::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pl1LoSpec;
impl crate::RegisterSpec for Pl1LoSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pl1_lo::R`](R) reader structure"]
impl crate::Readable for Pl1LoSpec {}
#[doc = "`write(|w| ..)` method takes [`pl1_lo::W`](W) writer structure"]
impl crate::Writable for Pl1LoSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PL1_LO to value 0"]
impl crate::Resettable for Pl1LoSpec {}
