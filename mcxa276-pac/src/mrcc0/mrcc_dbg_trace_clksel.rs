#[doc = "Register `MRCC_DBG_TRACE_CLKSEL` reader"]
pub type R = crate::R<MrccDbgTraceClkselSpec>;
#[doc = "Register `MRCC_DBG_TRACE_CLKSEL` writer"]
pub type W = crate::W<MrccDbgTraceClkselSpec>;
#[doc = "Functional Clock Mux Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mux {
    #[doc = "0: CPU_CLK"]
    ClkrootCpu = 0,
    #[doc = "1: CLK_1M"]
    Clkroot1m = 1,
    #[doc = "2: CLK_16K"]
    Clkroot16k = 2,
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
            0 => Some(Mux::ClkrootCpu),
            1 => Some(Mux::Clkroot1m),
            2 => Some(Mux::Clkroot16k),
            _ => None,
        }
    }
    #[doc = "CPU_CLK"]
    #[inline(always)]
    pub fn is_clkroot_cpu(&self) -> bool {
        *self == Mux::ClkrootCpu
    }
    #[doc = "CLK_1M"]
    #[inline(always)]
    pub fn is_clkroot_1m(&self) -> bool {
        *self == Mux::Clkroot1m
    }
    #[doc = "CLK_16K"]
    #[inline(always)]
    pub fn is_clkroot_16k(&self) -> bool {
        *self == Mux::Clkroot16k
    }
}
#[doc = "Field `MUX` writer - Functional Clock Mux Select"]
pub type MuxW<'a, REG> = crate::FieldWriter<'a, REG, 2, Mux>;
impl<'a, REG> MuxW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "CPU_CLK"]
    #[inline(always)]
    pub fn clkroot_cpu(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::ClkrootCpu)
    }
    #[doc = "CLK_1M"]
    #[inline(always)]
    pub fn clkroot_1m(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::Clkroot1m)
    }
    #[doc = "CLK_16K"]
    #[inline(always)]
    pub fn clkroot_16k(self) -> &'a mut crate::W<REG> {
        self.variant(Mux::Clkroot16k)
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
    pub fn mux(&mut self) -> MuxW<MrccDbgTraceClkselSpec> {
        MuxW::new(self, 0)
    }
}
#[doc = "DBG_TRACE clock selection control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_dbg_trace_clksel::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_dbg_trace_clksel::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrccDbgTraceClkselSpec;
impl crate::RegisterSpec for MrccDbgTraceClkselSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mrcc_dbg_trace_clksel::R`](R) reader structure"]
impl crate::Readable for MrccDbgTraceClkselSpec {}
#[doc = "`write(|w| ..)` method takes [`mrcc_dbg_trace_clksel::W`](W) writer structure"]
impl crate::Writable for MrccDbgTraceClkselSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MRCC_DBG_TRACE_CLKSEL to value 0"]
impl crate::Resettable for MrccDbgTraceClkselSpec {}
