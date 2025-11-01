#[doc = "Register `MRCC_FLEXCAN1_CLKSEL` reader"]
pub type R = crate::R<MrccFlexcan1ClkselSpec>;
#[doc = "Register `MRCC_FLEXCAN1_CLKSEL` writer"]
pub type W = crate::W<MrccFlexcan1ClkselSpec>;
#[doc = "Functional Clock Mux Select\n\nValue on reset: 7"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mux {
    #[doc = "1: FRO_HF_GATED"]
    ClkrootFircGated = 1,
    #[doc = "2: FRO_HF_DIV"]
    ClkrootFircDiv = 2,
    #[doc = "3: CLK_IN"]
    ClkrootSosc = 3,
    #[doc = "6: PLL1_CLK"]
    ClkrootSpll = 6,
}
impl From<Mux> for u8 {
    #[inline(always)]
    fn from(variant: Mux) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Mux {
    type Ux = u8;
}
impl crate::IsEnum for Mux {}
#[doc = "Field `MUX` reader - Functional Clock Mux Select"]
pub type MuxR = crate::FieldReader<Mux>;
impl MuxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Mux> {
        match self.bits {
            1 => Some(Mux::ClkrootFircGated),
            2 => Some(Mux::ClkrootFircDiv),
            3 => Some(Mux::ClkrootSosc),
            6 => Some(Mux::ClkrootSpll),
            _ => None,
        }
    }
    #[doc = "FRO_HF_GATED"]
    #[inline(always)]
    pub fn is_clkroot_firc_gated(&self) -> bool {
        *self == Mux::ClkrootFircGated
    }
    #[doc = "FRO_HF_DIV"]
    #[inline(always)]
    pub fn is_clkroot_firc_div(&self) -> bool {
        *self == Mux::ClkrootFircDiv
    }
    #[doc = "CLK_IN"]
    #[inline(always)]
    pub fn is_clkroot_sosc(&self) -> bool {
        *self == Mux::ClkrootSosc
    }
    #[doc = "PLL1_CLK"]
    #[inline(always)]
    pub fn is_clkroot_spll(&self) -> bool {
        *self == Mux::ClkrootSpll
    }
}
#[doc = "Field `MUX` writer - Functional Clock Mux Select"]
pub type MuxW<'a, REG> = crate::FieldWriter<'a, REG, 3, Mux>;
impl<'a, REG> MuxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "FRO_HF_GATED"]
    #[inline(always)]
    pub fn clkroot_firc_gated(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::ClkrootFircGated)
    }
    #[doc = "FRO_HF_DIV"]
    #[inline(always)]
    pub fn clkroot_firc_div(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::ClkrootFircDiv)
    }
    #[doc = "CLK_IN"]
    #[inline(always)]
    pub fn clkroot_sosc(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::ClkrootSosc)
    }
    #[doc = "PLL1_CLK"]
    #[inline(always)]
    pub fn clkroot_spll(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::ClkrootSpll)
    }
}
impl R {
    #[doc = "Bits 0:2 - Functional Clock Mux Select"]
    #[inline(always)]
    pub fn mux(&self) -> MuxR {
        MuxR::new((self.bits & 7) as u8)
    }
}
impl W {
    #[doc = "Bits 0:2 - Functional Clock Mux Select"]
    #[inline(always)]
    pub fn mux(&mut self) -> MuxW<MrccFlexcan1ClkselSpec> {
        MuxW::new(self, 0)
    }
}
#[doc = "FLEXCAN1 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_flexcan1_clksel::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_flexcan1_clksel::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrccFlexcan1ClkselSpec;
impl crate::RegisterSpec for MrccFlexcan1ClkselSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mrcc_flexcan1_clksel::R`](R) reader structure"]
impl crate::Readable for MrccFlexcan1ClkselSpec {}
#[doc = "`write(|w| ..)` method takes [`mrcc_flexcan1_clksel::W`](W) writer structure"]
impl crate::Writable for MrccFlexcan1ClkselSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MRCC_FLEXCAN1_CLKSEL to value 0x07"]
impl crate::Resettable for MrccFlexcan1ClkselSpec {
    const RESET_VALUE: u32 = 0x07;
}
