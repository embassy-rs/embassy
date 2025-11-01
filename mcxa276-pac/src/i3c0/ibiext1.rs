#[doc = "Register `IBIEXT1` reader"]
pub type R = crate::R<Ibiext1Spec>;
#[doc = "Register `IBIEXT1` writer"]
pub type W = crate::W<Ibiext1Spec>;
#[doc = "Field `CNT` reader - Count"]
pub type CntR = crate::FieldReader;
#[doc = "Field `CNT` writer - Count"]
pub type CntW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `MAX` reader - Maximum"]
pub type MaxR = crate::FieldReader;
#[doc = "Field `EXT1` reader - Extra Byte 1"]
pub type Ext1R = crate::FieldReader;
#[doc = "Field `EXT1` writer - Extra Byte 1"]
pub type Ext1W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `EXT2` reader - Extra Byte 2"]
pub type Ext2R = crate::FieldReader;
#[doc = "Field `EXT2` writer - Extra Byte 2"]
pub type Ext2W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `EXT3` reader - Extra Byte 3"]
pub type Ext3R = crate::FieldReader;
#[doc = "Field `EXT3` writer - Extra Byte 3"]
pub type Ext3W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:2 - Count"]
    #[inline(always)]
    pub fn cnt(&self) -> CntR {
        CntR::new((self.bits & 7) as u8)
    }
    #[doc = "Bits 4:6 - Maximum"]
    #[inline(always)]
    pub fn max(&self) -> MaxR {
        MaxR::new(((self.bits >> 4) & 7) as u8)
    }
    #[doc = "Bits 8:15 - Extra Byte 1"]
    #[inline(always)]
    pub fn ext1(&self) -> Ext1R {
        Ext1R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Extra Byte 2"]
    #[inline(always)]
    pub fn ext2(&self) -> Ext2R {
        Ext2R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Extra Byte 3"]
    #[inline(always)]
    pub fn ext3(&self) -> Ext3R {
        Ext3R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:2 - Count"]
    #[inline(always)]
    pub fn cnt(&mut self) -> CntW<Ibiext1Spec> {
        CntW::new(self, 0)
    }
    #[doc = "Bits 8:15 - Extra Byte 1"]
    #[inline(always)]
    pub fn ext1(&mut self) -> Ext1W<Ibiext1Spec> {
        Ext1W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Extra Byte 2"]
    #[inline(always)]
    pub fn ext2(&mut self) -> Ext2W<Ibiext1Spec> {
        Ext2W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Extra Byte 3"]
    #[inline(always)]
    pub fn ext3(&mut self) -> Ext3W<Ibiext1Spec> {
        Ext3W::new(self, 24)
    }
}
#[doc = "Extended IBI Data 1\n\nYou can [`read`](crate::Reg::read) this register and get [`ibiext1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ibiext1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ibiext1Spec;
impl crate::RegisterSpec for Ibiext1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ibiext1::R`](R) reader structure"]
impl crate::Readable for Ibiext1Spec {}
#[doc = "`write(|w| ..)` method takes [`ibiext1::W`](W) writer structure"]
impl crate::Writable for Ibiext1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets IBIEXT1 to value 0x70"]
impl crate::Resettable for Ibiext1Spec {
    const RESET_VALUE: u32 = 0x70;
}
