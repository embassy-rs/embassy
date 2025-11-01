#[doc = "Register `PGFR[%s]` reader"]
pub type R = crate::R<PgfrSpec>;
#[doc = "Register `PGFR[%s]` writer"]
pub type W = crate::W<PgfrSpec>;
#[doc = "Field `GFW` reader - Glitch Filter Width"]
pub type GfwR = crate::FieldReader;
#[doc = "Field `GFW` writer - Glitch Filter Width"]
pub type GfwW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Glitch Filter Prescaler\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gfp {
    #[doc = "0: 512 Hz prescaler clock"]
    Freq512Hz = 0,
    #[doc = "1: 32.768 kHz clock"]
    Freq32Khz = 1,
}
impl From<Gfp> for bool {
    #[inline(always)]
    fn from(variant: Gfp) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GFP` reader - Glitch Filter Prescaler"]
pub type GfpR = crate::BitReader<Gfp>;
impl GfpR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gfp {
        match self.bits {
            false => Gfp::Freq512Hz,
            true => Gfp::Freq32Khz,
        }
    }
    #[doc = "512 Hz prescaler clock"]
    #[inline(always)]
    pub fn is_freq_512_hz(&self) -> bool {
        *self == Gfp::Freq512Hz
    }
    #[doc = "32.768 kHz clock"]
    #[inline(always)]
    pub fn is_freq_32_khz(&self) -> bool {
        *self == Gfp::Freq32Khz
    }
}
#[doc = "Field `GFP` writer - Glitch Filter Prescaler"]
pub type GfpW<'a, REG> = crate::BitWriter<'a, REG, Gfp>;
impl<'a, REG> GfpW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "512 Hz prescaler clock"]
    #[inline(always)]
    pub fn freq_512_hz(self) -> &'a mut crate::W<REG> {
        self.variant(Gfp::Freq512Hz)
    }
    #[doc = "32.768 kHz clock"]
    #[inline(always)]
    pub fn freq_32_khz(self) -> &'a mut crate::W<REG> {
        self.variant(Gfp::Freq32Khz)
    }
}
#[doc = "Glitch Filter Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gfe {
    #[doc = "0: Bypasses"]
    Bypass = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Gfe> for bool {
    #[inline(always)]
    fn from(variant: Gfe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GFE` reader - Glitch Filter Enable"]
pub type GfeR = crate::BitReader<Gfe>;
impl GfeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gfe {
        match self.bits {
            false => Gfe::Bypass,
            true => Gfe::Enable,
        }
    }
    #[doc = "Bypasses"]
    #[inline(always)]
    pub fn is_bypass(&self) -> bool {
        *self == Gfe::Bypass
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Gfe::Enable
    }
}
#[doc = "Field `GFE` writer - Glitch Filter Enable"]
pub type GfeW<'a, REG> = crate::BitWriter<'a, REG, Gfe>;
impl<'a, REG> GfeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Bypasses"]
    #[inline(always)]
    pub fn bypass(self) -> &'a mut crate::W<REG> {
        self.variant(Gfe::Bypass)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Gfe::Enable)
    }
}
#[doc = "Tamper Pin Sample Width\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Tpsw {
    #[doc = "0: Continuous monitoring, pin sampling disabled"]
    Disable = 0,
    #[doc = "1: 2 cycles for pull enable and 1 cycle for input buffer enable"]
    Cycles2 = 1,
    #[doc = "2: 4 cycles for pull enable and 2 cycles for input buffer enable"]
    Cycles4 = 2,
    #[doc = "3: 8 cycles for pull enable and 4 cycles for input buffer enable"]
    Cycles8 = 3,
}
impl From<Tpsw> for u8 {
    #[inline(always)]
    fn from(variant: Tpsw) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Tpsw {
    type Ux = u8;
}
impl crate::IsEnum for Tpsw {}
#[doc = "Field `TPSW` reader - Tamper Pin Sample Width"]
pub type TpswR = crate::FieldReader<Tpsw>;
impl TpswR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpsw {
        match self.bits {
            0 => Tpsw::Disable,
            1 => Tpsw::Cycles2,
            2 => Tpsw::Cycles4,
            3 => Tpsw::Cycles8,
            _ => unreachable!(),
        }
    }
    #[doc = "Continuous monitoring, pin sampling disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tpsw::Disable
    }
    #[doc = "2 cycles for pull enable and 1 cycle for input buffer enable"]
    #[inline(always)]
    pub fn is_cycles_2(&self) -> bool {
        *self == Tpsw::Cycles2
    }
    #[doc = "4 cycles for pull enable and 2 cycles for input buffer enable"]
    #[inline(always)]
    pub fn is_cycles_4(&self) -> bool {
        *self == Tpsw::Cycles4
    }
    #[doc = "8 cycles for pull enable and 4 cycles for input buffer enable"]
    #[inline(always)]
    pub fn is_cycles_8(&self) -> bool {
        *self == Tpsw::Cycles8
    }
}
#[doc = "Field `TPSW` writer - Tamper Pin Sample Width"]
pub type TpswW<'a, REG> = crate::FieldWriter<'a, REG, 2, Tpsw, crate::Safe>;
impl<'a, REG> TpswW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Continuous monitoring, pin sampling disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpsw::Disable)
    }
    #[doc = "2 cycles for pull enable and 1 cycle for input buffer enable"]
    #[inline(always)]
    pub fn cycles_2(self) -> &'a mut crate::W<REG> {
        self.variant(Tpsw::Cycles2)
    }
    #[doc = "4 cycles for pull enable and 2 cycles for input buffer enable"]
    #[inline(always)]
    pub fn cycles_4(self) -> &'a mut crate::W<REG> {
        self.variant(Tpsw::Cycles4)
    }
    #[doc = "8 cycles for pull enable and 4 cycles for input buffer enable"]
    #[inline(always)]
    pub fn cycles_8(self) -> &'a mut crate::W<REG> {
        self.variant(Tpsw::Cycles8)
    }
}
#[doc = "Tamper Pin Sample Frequency\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Tpsf {
    #[doc = "0: Every 8 cycles"]
    Cycles8 = 0,
    #[doc = "1: Every 32 cycles"]
    Cycles32 = 1,
    #[doc = "2: Every 128 cycles"]
    Cycles128 = 2,
    #[doc = "3: Every 512 cycles"]
    Cycles512 = 3,
}
impl From<Tpsf> for u8 {
    #[inline(always)]
    fn from(variant: Tpsf) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Tpsf {
    type Ux = u8;
}
impl crate::IsEnum for Tpsf {}
#[doc = "Field `TPSF` reader - Tamper Pin Sample Frequency"]
pub type TpsfR = crate::FieldReader<Tpsf>;
impl TpsfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpsf {
        match self.bits {
            0 => Tpsf::Cycles8,
            1 => Tpsf::Cycles32,
            2 => Tpsf::Cycles128,
            3 => Tpsf::Cycles512,
            _ => unreachable!(),
        }
    }
    #[doc = "Every 8 cycles"]
    #[inline(always)]
    pub fn is_cycles_8(&self) -> bool {
        *self == Tpsf::Cycles8
    }
    #[doc = "Every 32 cycles"]
    #[inline(always)]
    pub fn is_cycles_32(&self) -> bool {
        *self == Tpsf::Cycles32
    }
    #[doc = "Every 128 cycles"]
    #[inline(always)]
    pub fn is_cycles_128(&self) -> bool {
        *self == Tpsf::Cycles128
    }
    #[doc = "Every 512 cycles"]
    #[inline(always)]
    pub fn is_cycles_512(&self) -> bool {
        *self == Tpsf::Cycles512
    }
}
#[doc = "Field `TPSF` writer - Tamper Pin Sample Frequency"]
pub type TpsfW<'a, REG> = crate::FieldWriter<'a, REG, 2, Tpsf, crate::Safe>;
impl<'a, REG> TpsfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Every 8 cycles"]
    #[inline(always)]
    pub fn cycles_8(self) -> &'a mut crate::W<REG> {
        self.variant(Tpsf::Cycles8)
    }
    #[doc = "Every 32 cycles"]
    #[inline(always)]
    pub fn cycles_32(self) -> &'a mut crate::W<REG> {
        self.variant(Tpsf::Cycles32)
    }
    #[doc = "Every 128 cycles"]
    #[inline(always)]
    pub fn cycles_128(self) -> &'a mut crate::W<REG> {
        self.variant(Tpsf::Cycles128)
    }
    #[doc = "Every 512 cycles"]
    #[inline(always)]
    pub fn cycles_512(self) -> &'a mut crate::W<REG> {
        self.variant(Tpsf::Cycles512)
    }
}
#[doc = "Tamper Pull Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpe {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tpe> for bool {
    #[inline(always)]
    fn from(variant: Tpe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPE` reader - Tamper Pull Enable"]
pub type TpeR = crate::BitReader<Tpe>;
impl TpeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpe {
        match self.bits {
            false => Tpe::Disable,
            true => Tpe::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tpe::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tpe::Enable
    }
}
#[doc = "Field `TPE` writer - Tamper Pull Enable"]
pub type TpeW<'a, REG> = crate::BitWriter<'a, REG, Tpe>;
impl<'a, REG> TpeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpe::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpe::Enable)
    }
}
#[doc = "Tamper Pull Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tps {
    #[doc = "0: Asserts"]
    Assert = 0,
    #[doc = "1: Negates"]
    Negate = 1,
}
impl From<Tps> for bool {
    #[inline(always)]
    fn from(variant: Tps) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPS` reader - Tamper Pull Select"]
pub type TpsR = crate::BitReader<Tps>;
impl TpsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tps {
        match self.bits {
            false => Tps::Assert,
            true => Tps::Negate,
        }
    }
    #[doc = "Asserts"]
    #[inline(always)]
    pub fn is_assert(&self) -> bool {
        *self == Tps::Assert
    }
    #[doc = "Negates"]
    #[inline(always)]
    pub fn is_negate(&self) -> bool {
        *self == Tps::Negate
    }
}
#[doc = "Field `TPS` writer - Tamper Pull Select"]
pub type TpsW<'a, REG> = crate::BitWriter<'a, REG, Tps>;
impl<'a, REG> TpsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Asserts"]
    #[inline(always)]
    pub fn assert(self) -> &'a mut crate::W<REG> {
        self.variant(Tps::Assert)
    }
    #[doc = "Negates"]
    #[inline(always)]
    pub fn negate(self) -> &'a mut crate::W<REG> {
        self.variant(Tps::Negate)
    }
}
#[doc = "Tamper Pull Value\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpv {
    #[doc = "0: Low"]
    Low = 0,
    #[doc = "1: High"]
    High = 1,
}
impl From<Tpv> for bool {
    #[inline(always)]
    fn from(variant: Tpv) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPV` reader - Tamper Pull Value"]
pub type TpvR = crate::BitReader<Tpv>;
impl TpvR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpv {
        match self.bits {
            false => Tpv::Low,
            true => Tpv::High,
        }
    }
    #[doc = "Low"]
    #[inline(always)]
    pub fn is_low(&self) -> bool {
        *self == Tpv::Low
    }
    #[doc = "High"]
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        *self == Tpv::High
    }
}
#[doc = "Field `TPV` writer - Tamper Pull Value"]
pub type TpvW<'a, REG> = crate::BitWriter<'a, REG, Tpv>;
impl<'a, REG> TpvW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Low"]
    #[inline(always)]
    pub fn low(self) -> &'a mut crate::W<REG> {
        self.variant(Tpv::Low)
    }
    #[doc = "High"]
    #[inline(always)]
    pub fn high(self) -> &'a mut crate::W<REG> {
        self.variant(Tpv::High)
    }
}
#[doc = "Tamper Passive Filter\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpf {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tpf> for bool {
    #[inline(always)]
    fn from(variant: Tpf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPF` reader - Tamper Passive Filter"]
pub type TpfR = crate::BitReader<Tpf>;
impl TpfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpf {
        match self.bits {
            false => Tpf::Disable,
            true => Tpf::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tpf::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tpf::Enable
    }
}
#[doc = "Field `TPF` writer - Tamper Passive Filter"]
pub type TpfW<'a, REG> = crate::BitWriter<'a, REG, Tpf>;
impl<'a, REG> TpfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpf::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpf::Enable)
    }
}
#[doc = "Input Buffer Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ibe {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Ibe> for bool {
    #[inline(always)]
    fn from(variant: Ibe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IBE` reader - Input Buffer Enable"]
pub type IbeR = crate::BitReader<Ibe>;
impl IbeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ibe {
        match self.bits {
            false => Ibe::Disable,
            true => Ibe::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Ibe::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Ibe::Enable
    }
}
#[doc = "Field `IBE` writer - Input Buffer Enable"]
pub type IbeW<'a, REG> = crate::BitWriter<'a, REG, Ibe>;
impl<'a, REG> IbeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Ibe::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Ibe::Enable)
    }
}
impl R {
    #[doc = "Bits 0:5 - Glitch Filter Width"]
    #[inline(always)]
    pub fn gfw(&self) -> GfwR {
        GfwR::new((self.bits & 0x3f) as u8)
    }
    #[doc = "Bit 6 - Glitch Filter Prescaler"]
    #[inline(always)]
    pub fn gfp(&self) -> GfpR {
        GfpR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Glitch Filter Enable"]
    #[inline(always)]
    pub fn gfe(&self) -> GfeR {
        GfeR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bits 8:9 - Tamper Pin Sample Width"]
    #[inline(always)]
    pub fn tpsw(&self) -> TpswR {
        TpswR::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bits 10:11 - Tamper Pin Sample Frequency"]
    #[inline(always)]
    pub fn tpsf(&self) -> TpsfR {
        TpsfR::new(((self.bits >> 10) & 3) as u8)
    }
    #[doc = "Bit 24 - Tamper Pull Enable"]
    #[inline(always)]
    pub fn tpe(&self) -> TpeR {
        TpeR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Tamper Pull Select"]
    #[inline(always)]
    pub fn tps(&self) -> TpsR {
        TpsR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Tamper Pull Value"]
    #[inline(always)]
    pub fn tpv(&self) -> TpvR {
        TpvR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Tamper Passive Filter"]
    #[inline(always)]
    pub fn tpf(&self) -> TpfR {
        TpfR::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 31 - Input Buffer Enable"]
    #[inline(always)]
    pub fn ibe(&self) -> IbeR {
        IbeR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:5 - Glitch Filter Width"]
    #[inline(always)]
    pub fn gfw(&mut self) -> GfwW<PgfrSpec> {
        GfwW::new(self, 0)
    }
    #[doc = "Bit 6 - Glitch Filter Prescaler"]
    #[inline(always)]
    pub fn gfp(&mut self) -> GfpW<PgfrSpec> {
        GfpW::new(self, 6)
    }
    #[doc = "Bit 7 - Glitch Filter Enable"]
    #[inline(always)]
    pub fn gfe(&mut self) -> GfeW<PgfrSpec> {
        GfeW::new(self, 7)
    }
    #[doc = "Bits 8:9 - Tamper Pin Sample Width"]
    #[inline(always)]
    pub fn tpsw(&mut self) -> TpswW<PgfrSpec> {
        TpswW::new(self, 8)
    }
    #[doc = "Bits 10:11 - Tamper Pin Sample Frequency"]
    #[inline(always)]
    pub fn tpsf(&mut self) -> TpsfW<PgfrSpec> {
        TpsfW::new(self, 10)
    }
    #[doc = "Bit 24 - Tamper Pull Enable"]
    #[inline(always)]
    pub fn tpe(&mut self) -> TpeW<PgfrSpec> {
        TpeW::new(self, 24)
    }
    #[doc = "Bit 25 - Tamper Pull Select"]
    #[inline(always)]
    pub fn tps(&mut self) -> TpsW<PgfrSpec> {
        TpsW::new(self, 25)
    }
    #[doc = "Bit 26 - Tamper Pull Value"]
    #[inline(always)]
    pub fn tpv(&mut self) -> TpvW<PgfrSpec> {
        TpvW::new(self, 26)
    }
    #[doc = "Bit 27 - Tamper Passive Filter"]
    #[inline(always)]
    pub fn tpf(&mut self) -> TpfW<PgfrSpec> {
        TpfW::new(self, 27)
    }
    #[doc = "Bit 31 - Input Buffer Enable"]
    #[inline(always)]
    pub fn ibe(&mut self) -> IbeW<PgfrSpec> {
        IbeW::new(self, 31)
    }
}
#[doc = "Pin Glitch Filter\n\nYou can [`read`](crate::Reg::read) this register and get [`pgfr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pgfr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PgfrSpec;
impl crate::RegisterSpec for PgfrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pgfr::R`](R) reader structure"]
impl crate::Readable for PgfrSpec {}
#[doc = "`write(|w| ..)` method takes [`pgfr::W`](W) writer structure"]
impl crate::Writable for PgfrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PGFR[%s] to value 0"]
impl crate::Resettable for PgfrSpec {}
