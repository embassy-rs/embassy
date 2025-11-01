#[doc = "Register `RPC` reader"]
pub type R = crate::R<RpcSpec>;
#[doc = "Register `RPC` writer"]
pub type W = crate::W<RpcSpec>;
#[doc = "Field `FILTCFG` reader - Reset Filter Configuration"]
pub type FiltcfgR = crate::FieldReader;
#[doc = "Field `FILTCFG` writer - Reset Filter Configuration"]
pub type FiltcfgW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Filter Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filten {
    #[doc = "0: Disables"]
    Disabled = 0,
    #[doc = "1: Enables"]
    Enabled = 1,
}
impl From<Filten> for bool {
    #[inline(always)]
    fn from(variant: Filten) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FILTEN` reader - Filter Enable"]
pub type FiltenR = crate::BitReader<Filten>;
impl FiltenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Filten {
        match self.bits {
            false => Filten::Disabled,
            true => Filten::Enabled,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Filten::Disabled
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Filten::Enabled
    }
}
#[doc = "Field `FILTEN` writer - Filter Enable"]
pub type FiltenW<'a, REG> = crate::BitWriter<'a, REG, Filten>;
impl<'a, REG> FiltenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Filten::Disabled)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Filten::Enabled)
    }
}
#[doc = "Low-Power Filter Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lpfen {
    #[doc = "0: Disables"]
    Disabled = 0,
    #[doc = "1: Enables"]
    Enabled = 1,
}
impl From<Lpfen> for bool {
    #[inline(always)]
    fn from(variant: Lpfen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPFEN` reader - Low-Power Filter Enable"]
pub type LpfenR = crate::BitReader<Lpfen>;
impl LpfenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lpfen {
        match self.bits {
            false => Lpfen::Disabled,
            true => Lpfen::Enabled,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Lpfen::Disabled
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Lpfen::Enabled
    }
}
#[doc = "Field `LPFEN` writer - Low-Power Filter Enable"]
pub type LpfenW<'a, REG> = crate::BitWriter<'a, REG, Lpfen>;
impl<'a, REG> LpfenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpfen::Disabled)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Lpfen::Enabled)
    }
}
impl R {
    #[doc = "Bits 0:4 - Reset Filter Configuration"]
    #[inline(always)]
    pub fn filtcfg(&self) -> FiltcfgR {
        FiltcfgR::new((self.bits & 0x1f) as u8)
    }
    #[doc = "Bit 8 - Filter Enable"]
    #[inline(always)]
    pub fn filten(&self) -> FiltenR {
        FiltenR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Low-Power Filter Enable"]
    #[inline(always)]
    pub fn lpfen(&self) -> LpfenR {
        LpfenR::new(((self.bits >> 9) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:4 - Reset Filter Configuration"]
    #[inline(always)]
    pub fn filtcfg(&mut self) -> FiltcfgW<RpcSpec> {
        FiltcfgW::new(self, 0)
    }
    #[doc = "Bit 8 - Filter Enable"]
    #[inline(always)]
    pub fn filten(&mut self) -> FiltenW<RpcSpec> {
        FiltenW::new(self, 8)
    }
    #[doc = "Bit 9 - Low-Power Filter Enable"]
    #[inline(always)]
    pub fn lpfen(&mut self) -> LpfenW<RpcSpec> {
        LpfenW::new(self, 9)
    }
}
#[doc = "Reset Pin Control\n\nYou can [`read`](crate::Reg::read) this register and get [`rpc::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rpc::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RpcSpec;
impl crate::RegisterSpec for RpcSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rpc::R`](R) reader structure"]
impl crate::Readable for RpcSpec {}
#[doc = "`write(|w| ..)` method takes [`rpc::W`](W) writer structure"]
impl crate::Writable for RpcSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RPC to value 0"]
impl crate::Resettable for RpcSpec {}
