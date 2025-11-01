#[doc = "Register `MRCC_USB0_CLKSEL` reader"]
pub type R = crate::R<MrccUsb0ClkselSpec>;
#[doc = "Register `MRCC_USB0_CLKSEL` writer"]
pub type W = crate::W<MrccUsb0ClkselSpec>;
#[doc = "Functional Clock Mux Select\n\nValue on reset: 3"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mux {
    #[doc = "0: PLL1_CLK"]
    ClkrootSpll = 0,
    #[doc = "1: CLK_48M"]
    ScgScgFirc48mhzClk = 1,
    #[doc = "2: CLK_IN"]
    ClkrootSosc = 2,
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
            0 => Some(Mux::ClkrootSpll),
            1 => Some(Mux::ScgScgFirc48mhzClk),
            2 => Some(Mux::ClkrootSosc),
            _ => None,
        }
    }
    #[doc = "PLL1_CLK"]
    #[inline(always)]
    pub fn is_clkroot_spll(&self) -> bool {
        *self == Mux::ClkrootSpll
    }
    #[doc = "CLK_48M"]
    #[inline(always)]
    pub fn is_scg_scg_firc_48mhz_clk(&self) -> bool {
        *self == Mux::ScgScgFirc48mhzClk
    }
    #[doc = "CLK_IN"]
    #[inline(always)]
    pub fn is_clkroot_sosc(&self) -> bool {
        *self == Mux::ClkrootSosc
    }
}
#[doc = "Field `MUX` writer - Functional Clock Mux Select"]
pub type MuxW<'a, REG> = crate::FieldWriter<'a, REG, 2, Mux>;
impl<'a, REG> MuxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "PLL1_CLK"]
    #[inline(always)]
    pub fn clkroot_spll(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::ClkrootSpll)
    }
    #[doc = "CLK_48M"]
    #[inline(always)]
    pub fn scg_scg_firc_48mhz_clk(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::ScgScgFirc48mhzClk)
    }
    #[doc = "CLK_IN"]
    #[inline(always)]
    pub fn clkroot_sosc(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::ClkrootSosc)
    }
}
impl R {
    #[doc = "Bits 0:1 - Functional Clock Mux Select"]
    #[inline(always)]
    pub fn mux(&self) -> MuxR {
        MuxR::new((self.bits & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Functional Clock Mux Select"]
    #[inline(always)]
    pub fn mux(&mut self) -> MuxW<MrccUsb0ClkselSpec> {
        MuxW::new(self, 0)
    }
}
#[doc = "USB0 clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_usb0_clksel::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_usb0_clksel::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrccUsb0ClkselSpec;
impl crate::RegisterSpec for MrccUsb0ClkselSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mrcc_usb0_clksel::R`](R) reader structure"]
impl crate::Readable for MrccUsb0ClkselSpec {}
#[doc = "`write(|w| ..)` method takes [`mrcc_usb0_clksel::W`](W) writer structure"]
impl crate::Writable for MrccUsb0ClkselSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MRCC_USB0_CLKSEL to value 0x03"]
impl crate::Resettable for MrccUsb0ClkselSpec {
    const RESET_VALUE: u32 = 0x03;
}
