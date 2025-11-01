#[doc = "Register `IBIEXT2` reader"]
pub type R = crate::R<Ibiext2Spec>;
#[doc = "Register `IBIEXT2` writer"]
pub type W = crate::W<Ibiext2Spec>;
#[doc = "Field `EXT4` reader - Extra Byte 4"]
pub type Ext4R = crate::FieldReader;
#[doc = "Field `EXT4` writer - Extra Byte 4"]
pub type Ext4W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `EXT5` reader - Extra Byte 5"]
pub type Ext5R = crate::FieldReader;
#[doc = "Field `EXT5` writer - Extra Byte 5"]
pub type Ext5W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `EXT6` reader - Extra Byte 6"]
pub type Ext6R = crate::FieldReader;
#[doc = "Field `EXT6` writer - Extra Byte 6"]
pub type Ext6W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `EXT7` reader - Extra Byte 7"]
pub type Ext7R = crate::FieldReader;
#[doc = "Field `EXT7` writer - Extra Byte 7"]
pub type Ext7W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Extra Byte 4"]
    #[inline(always)]
    pub fn ext4(&self) -> Ext4R {
        Ext4R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Extra Byte 5"]
    #[inline(always)]
    pub fn ext5(&self) -> Ext5R {
        Ext5R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Extra Byte 6"]
    #[inline(always)]
    pub fn ext6(&self) -> Ext6R {
        Ext6R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Extra Byte 7"]
    #[inline(always)]
    pub fn ext7(&self) -> Ext7R {
        Ext7R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Extra Byte 4"]
    #[inline(always)]
    pub fn ext4(&mut self) -> Ext4W<Ibiext2Spec> {
        Ext4W::new(self, 0)
    }
    #[doc = "Bits 8:15 - Extra Byte 5"]
    #[inline(always)]
    pub fn ext5(&mut self) -> Ext5W<Ibiext2Spec> {
        Ext5W::new(self, 8)
    }
    #[doc = "Bits 16:23 - Extra Byte 6"]
    #[inline(always)]
    pub fn ext6(&mut self) -> Ext6W<Ibiext2Spec> {
        Ext6W::new(self, 16)
    }
    #[doc = "Bits 24:31 - Extra Byte 7"]
    #[inline(always)]
    pub fn ext7(&mut self) -> Ext7W<Ibiext2Spec> {
        Ext7W::new(self, 24)
    }
}
#[doc = "Extended IBI Data 2\n\nYou can [`read`](crate::Reg::read) this register and get [`ibiext2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ibiext2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ibiext2Spec;
impl crate::RegisterSpec for Ibiext2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ibiext2::R`](R) reader structure"]
impl crate::Readable for Ibiext2Spec {}
#[doc = "`write(|w| ..)` method takes [`ibiext2::W`](W) writer structure"]
impl crate::Writable for Ibiext2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets IBIEXT2 to value 0"]
impl crate::Resettable for Ibiext2Spec {}
