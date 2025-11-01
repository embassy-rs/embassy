#[doc = "Register `RAM_INTERLEAVE` reader"]
pub type R = crate::R<RamInterleaveSpec>;
#[doc = "Register `RAM_INTERLEAVE` writer"]
pub type W = crate::W<RamInterleaveSpec>;
#[doc = "Controls RAM access for RAMA1 and RAMA2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interleave {
    #[doc = "0: RAM access is consecutive."]
    Normal = 0,
    #[doc = "1: RAM access is interleaved. This setting is need for PKC L0 memory access."]
    Interleave = 1,
}
impl From<Interleave> for bool {
    #[inline(always)]
    fn from(variant: Interleave) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INTERLEAVE` reader - Controls RAM access for RAMA1 and RAMA2"]
pub type InterleaveR = crate::BitReader<Interleave>;
impl InterleaveR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Interleave {
        match self.bits {
            false => Interleave::Normal,
            true => Interleave::Interleave,
        }
    }
    #[doc = "RAM access is consecutive."]
    #[inline(always)]
    pub fn is_normal(&self) -> bool {
        *self == Interleave::Normal
    }
    #[doc = "RAM access is interleaved. This setting is need for PKC L0 memory access."]
    #[inline(always)]
    pub fn is_interleave(&self) -> bool {
        *self == Interleave::Interleave
    }
}
#[doc = "Field `INTERLEAVE` writer - Controls RAM access for RAMA1 and RAMA2"]
pub type InterleaveW<'a, REG> = crate::BitWriter<'a, REG, Interleave>;
impl<'a, REG> InterleaveW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "RAM access is consecutive."]
    #[inline(always)]
    pub fn normal(self) -> &'a mut crate::W<REG> {
        self.variant(Interleave::Normal)
    }
    #[doc = "RAM access is interleaved. This setting is need for PKC L0 memory access."]
    #[inline(always)]
    pub fn interleave(self) -> &'a mut crate::W<REG> {
        self.variant(Interleave::Interleave)
    }
}
impl R {
    #[doc = "Bit 0 - Controls RAM access for RAMA1 and RAMA2"]
    #[inline(always)]
    pub fn interleave(&self) -> InterleaveR {
        InterleaveR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Controls RAM access for RAMA1 and RAMA2"]
    #[inline(always)]
    pub fn interleave(&mut self) -> InterleaveW<RamInterleaveSpec> {
        InterleaveW::new(self, 0)
    }
}
#[doc = "Controls RAM Interleave Integration\n\nYou can [`read`](crate::Reg::read) this register and get [`ram_interleave::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ram_interleave::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RamInterleaveSpec;
impl crate::RegisterSpec for RamInterleaveSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ram_interleave::R`](R) reader structure"]
impl crate::Readable for RamInterleaveSpec {}
#[doc = "`write(|w| ..)` method takes [`ram_interleave::W`](W) writer structure"]
impl crate::Writable for RamInterleaveSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RAM_INTERLEAVE to value 0"]
impl crate::Resettable for RamInterleaveSpec {}
