#[doc = "Register `MRCC_CLKOUT_CLKSEL` reader"]
pub type R = crate::R<MrccClkoutClkselSpec>;
#[doc = "Register `MRCC_CLKOUT_CLKSEL` writer"]
pub type W = crate::W<MrccClkoutClkselSpec>;
#[doc = "Functional Clock Mux Select\n\nValue on reset: 7"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mux {
    #[doc = "0: FRO_12M"]
    Clkroot12m = 0,
    #[doc = "1: FRO_HF_DIV"]
    ClkrootFircDiv = 1,
    #[doc = "2: CLK_IN"]
    ClkrootSosc = 2,
    #[doc = "3: CLK_16K"]
    Clkroot16k = 3,
    #[doc = "5: PLL1_CLK"]
    ClkrootSpll = 5,
    #[doc = "6: SLOW_CLK"]
    ClkrootSlow = 6,
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
            0 => Some(Mux::Clkroot12m),
            1 => Some(Mux::ClkrootFircDiv),
            2 => Some(Mux::ClkrootSosc),
            3 => Some(Mux::Clkroot16k),
            5 => Some(Mux::ClkrootSpll),
            6 => Some(Mux::ClkrootSlow),
            _ => None,
        }
    }
    #[doc = "FRO_12M"]
    #[inline(always)]
    pub fn is_clkroot_12m(&self) -> bool {
        *self == Mux::Clkroot12m
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
    #[doc = "CLK_16K"]
    #[inline(always)]
    pub fn is_clkroot_16k(&self) -> bool {
        *self == Mux::Clkroot16k
    }
    #[doc = "PLL1_CLK"]
    #[inline(always)]
    pub fn is_clkroot_spll(&self) -> bool {
        *self == Mux::ClkrootSpll
    }
    #[doc = "SLOW_CLK"]
    #[inline(always)]
    pub fn is_clkroot_slow(&self) -> bool {
        *self == Mux::ClkrootSlow
    }
}
#[doc = "Field `MUX` writer - Functional Clock Mux Select"]
pub type MuxW<'a, REG> = crate::FieldWriter<'a, REG, 3, Mux>;
impl<'a, REG> MuxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "FRO_12M"]
    #[inline(always)]
    pub fn clkroot_12m(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::Clkroot12m)
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
    #[doc = "CLK_16K"]
    #[inline(always)]
    pub fn clkroot_16k(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::Clkroot16k)
    }
    #[doc = "PLL1_CLK"]
    #[inline(always)]
    pub fn clkroot_spll(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::ClkrootSpll)
    }
    #[doc = "SLOW_CLK"]
    #[inline(always)]
    pub fn clkroot_slow(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::ClkrootSlow)
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
    pub fn mux(&mut self) -> MuxW<MrccClkoutClkselSpec> {
        MuxW::new(self, 0)
    }
}
#[doc = "CLKOUT clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_clkout_clksel::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_clkout_clksel::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrccClkoutClkselSpec;
impl crate::RegisterSpec for MrccClkoutClkselSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mrcc_clkout_clksel::R`](R) reader structure"]
impl crate::Readable for MrccClkoutClkselSpec {}
#[doc = "`write(|w| ..)` method takes [`mrcc_clkout_clksel::W`](W) writer structure"]
impl crate::Writable for MrccClkoutClkselSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MRCC_CLKOUT_CLKSEL to value 0x07"]
impl crate::Resettable for MrccClkoutClkselSpec {
    const RESET_VALUE: u32 = 0x07;
}
