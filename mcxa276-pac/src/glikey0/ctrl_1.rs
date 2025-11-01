#[doc = "Register `CTRL_1` reader"]
pub type R = crate::R<Ctrl1Spec>;
#[doc = "Register `CTRL_1` writer"]
pub type W = crate::W<Ctrl1Spec>;
#[doc = "Field `READ_INDEX` reader - Index status, Writing an index value to this register will request the block to return the lock status of this index."]
pub type ReadIndexR = crate::FieldReader;
#[doc = "Field `READ_INDEX` writer - Index status, Writing an index value to this register will request the block to return the lock status of this index."]
pub type ReadIndexW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `RESERVED15` reader - Reserved for Future Use"]
pub type Reserved15R = crate::FieldReader;
#[doc = "Field `WR_EN_1` reader - Write Enable One"]
pub type WrEn1R = crate::FieldReader;
#[doc = "Field `WR_EN_1` writer - Write Enable One"]
pub type WrEn1W<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `SFR_LOCK` reader - LOCK register for GLIKEY"]
pub type SfrLockR = crate::FieldReader;
#[doc = "Field `SFR_LOCK` writer - LOCK register for GLIKEY"]
pub type SfrLockW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `RESERVED31` reader - Reserved for Future Use"]
pub type Reserved31R = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:7 - Index status, Writing an index value to this register will request the block to return the lock status of this index."]
    #[inline(always)]
    pub fn read_index(&self) -> ReadIndexR {
        ReadIndexR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Reserved for Future Use"]
    #[inline(always)]
    pub fn reserved15(&self) -> Reserved15R {
        Reserved15R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:17 - Write Enable One"]
    #[inline(always)]
    pub fn wr_en_1(&self) -> WrEn1R {
        WrEn1R::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bits 18:21 - LOCK register for GLIKEY"]
    #[inline(always)]
    pub fn sfr_lock(&self) -> SfrLockR {
        SfrLockR::new(((self.bits >> 18) & 0x0f) as u8)
    }
    #[doc = "Bits 22:31 - Reserved for Future Use"]
    #[inline(always)]
    pub fn reserved31(&self) -> Reserved31R {
        Reserved31R::new(((self.bits >> 22) & 0x03ff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:7 - Index status, Writing an index value to this register will request the block to return the lock status of this index."]
    #[inline(always)]
    pub fn read_index(&mut self) -> ReadIndexW<Ctrl1Spec> {
        ReadIndexW::new(self, 0)
    }
    #[doc = "Bits 16:17 - Write Enable One"]
    #[inline(always)]
    pub fn wr_en_1(&mut self) -> WrEn1W<Ctrl1Spec> {
        WrEn1W::new(self, 16)
    }
    #[doc = "Bits 18:21 - LOCK register for GLIKEY"]
    #[inline(always)]
    pub fn sfr_lock(&mut self) -> SfrLockW<Ctrl1Spec> {
        SfrLockW::new(self, 18)
    }
}
#[doc = "Control Register 1 SFR\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl_1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl_1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ctrl1Spec;
impl crate::RegisterSpec for Ctrl1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ctrl_1::R`](R) reader structure"]
impl crate::Readable for Ctrl1Spec {}
#[doc = "`write(|w| ..)` method takes [`ctrl_1::W`](W) writer structure"]
impl crate::Writable for Ctrl1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CTRL_1 to value 0x0028_0000"]
impl crate::Resettable for Ctrl1Spec {
    const RESET_VALUE: u32 = 0x0028_0000;
}
