#[doc = "Register `CSR` reader"]
pub type R = crate::R<CsrSpec>;
#[doc = "Register `CSR` writer"]
pub type W = crate::W<CsrSpec>;
#[doc = "Timer Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ten {
    #[doc = "0: Disable"]
    Ten0 = 0,
    #[doc = "1: Enable"]
    Ten1 = 1,
}
impl From<Ten> for bool {
    #[inline(always)]
    fn from(variant: Ten) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TEN` reader - Timer Enable"]
pub type TenR = crate::BitReader<Ten>;
impl TenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ten {
        match self.bits {
            false => Ten::Ten0,
            true => Ten::Ten1,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_ten0(&self) -> bool {
        *self == Ten::Ten0
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_ten1(&self) -> bool {
        *self == Ten::Ten1
    }
}
#[doc = "Field `TEN` writer - Timer Enable"]
pub type TenW<'a, REG> = crate::BitWriter<'a, REG, Ten>;
impl<'a, REG> TenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn ten0(self) -> &'a mut crate::W<REG> {
        self.variant(Ten::Ten0)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn ten1(self) -> &'a mut crate::W<REG> {
        self.variant(Ten::Ten1)
    }
}
#[doc = "Timer Mode Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tms {
    #[doc = "0: Time Counter"]
    Tms0 = 0,
    #[doc = "1: Pulse Counter"]
    Tms1 = 1,
}
impl From<Tms> for bool {
    #[inline(always)]
    fn from(variant: Tms) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TMS` reader - Timer Mode Select"]
pub type TmsR = crate::BitReader<Tms>;
impl TmsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tms {
        match self.bits {
            false => Tms::Tms0,
            true => Tms::Tms1,
        }
    }
    #[doc = "Time Counter"]
    #[inline(always)]
    pub fn is_tms0(&self) -> bool {
        *self == Tms::Tms0
    }
    #[doc = "Pulse Counter"]
    #[inline(always)]
    pub fn is_tms1(&self) -> bool {
        *self == Tms::Tms1
    }
}
#[doc = "Field `TMS` writer - Timer Mode Select"]
pub type TmsW<'a, REG> = crate::BitWriter<'a, REG, Tms>;
impl<'a, REG> TmsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Time Counter"]
    #[inline(always)]
    pub fn tms0(self) -> &'a mut crate::W<REG> {
        self.variant(Tms::Tms0)
    }
    #[doc = "Pulse Counter"]
    #[inline(always)]
    pub fn tms1(self) -> &'a mut crate::W<REG> {
        self.variant(Tms::Tms1)
    }
}
#[doc = "Timer Free-Running Counter\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tfc {
    #[doc = "0: Reset when TCF asserts"]
    Tfc0 = 0,
    #[doc = "1: Reset on overflow"]
    Tfc1 = 1,
}
impl From<Tfc> for bool {
    #[inline(always)]
    fn from(variant: Tfc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TFC` reader - Timer Free-Running Counter"]
pub type TfcR = crate::BitReader<Tfc>;
impl TfcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tfc {
        match self.bits {
            false => Tfc::Tfc0,
            true => Tfc::Tfc1,
        }
    }
    #[doc = "Reset when TCF asserts"]
    #[inline(always)]
    pub fn is_tfc0(&self) -> bool {
        *self == Tfc::Tfc0
    }
    #[doc = "Reset on overflow"]
    #[inline(always)]
    pub fn is_tfc1(&self) -> bool {
        *self == Tfc::Tfc1
    }
}
#[doc = "Field `TFC` writer - Timer Free-Running Counter"]
pub type TfcW<'a, REG> = crate::BitWriter<'a, REG, Tfc>;
impl<'a, REG> TfcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reset when TCF asserts"]
    #[inline(always)]
    pub fn tfc0(self) -> &'a mut crate::W<REG> {
        self.variant(Tfc::Tfc0)
    }
    #[doc = "Reset on overflow"]
    #[inline(always)]
    pub fn tfc1(self) -> &'a mut crate::W<REG> {
        self.variant(Tfc::Tfc1)
    }
}
#[doc = "Timer Pin Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpp {
    #[doc = "0: Active-high"]
    Tpp0 = 0,
    #[doc = "1: Active-low"]
    Tpp1 = 1,
}
impl From<Tpp> for bool {
    #[inline(always)]
    fn from(variant: Tpp) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPP` reader - Timer Pin Polarity"]
pub type TppR = crate::BitReader<Tpp>;
impl TppR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpp {
        match self.bits {
            false => Tpp::Tpp0,
            true => Tpp::Tpp1,
        }
    }
    #[doc = "Active-high"]
    #[inline(always)]
    pub fn is_tpp0(&self) -> bool {
        *self == Tpp::Tpp0
    }
    #[doc = "Active-low"]
    #[inline(always)]
    pub fn is_tpp1(&self) -> bool {
        *self == Tpp::Tpp1
    }
}
#[doc = "Field `TPP` writer - Timer Pin Polarity"]
pub type TppW<'a, REG> = crate::BitWriter<'a, REG, Tpp>;
impl<'a, REG> TppW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Active-high"]
    #[inline(always)]
    pub fn tpp0(self) -> &'a mut crate::W<REG> {
        self.variant(Tpp::Tpp0)
    }
    #[doc = "Active-low"]
    #[inline(always)]
    pub fn tpp1(self) -> &'a mut crate::W<REG> {
        self.variant(Tpp::Tpp1)
    }
}
#[doc = "Timer Pin Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Tps {
    #[doc = "0: Input 0"]
    Tps00 = 0,
    #[doc = "1: Input 1"]
    Tps01 = 1,
    #[doc = "2: Input 2"]
    Tps10 = 2,
    #[doc = "3: Input 3"]
    Tps11 = 3,
}
impl From<Tps> for u8 {
    #[inline(always)]
    fn from(variant: Tps) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Tps {
    type Ux = u8;
}
impl crate::IsEnum for Tps {}
#[doc = "Field `TPS` reader - Timer Pin Select"]
pub type TpsR = crate::FieldReader<Tps>;
impl TpsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tps {
        match self.bits {
            0 => Tps::Tps00,
            1 => Tps::Tps01,
            2 => Tps::Tps10,
            3 => Tps::Tps11,
            _ => unreachable!(),
        }
    }
    #[doc = "Input 0"]
    #[inline(always)]
    pub fn is_tps00(&self) -> bool {
        *self == Tps::Tps00
    }
    #[doc = "Input 1"]
    #[inline(always)]
    pub fn is_tps01(&self) -> bool {
        *self == Tps::Tps01
    }
    #[doc = "Input 2"]
    #[inline(always)]
    pub fn is_tps10(&self) -> bool {
        *self == Tps::Tps10
    }
    #[doc = "Input 3"]
    #[inline(always)]
    pub fn is_tps11(&self) -> bool {
        *self == Tps::Tps11
    }
}
#[doc = "Field `TPS` writer - Timer Pin Select"]
pub type TpsW<'a, REG> = crate::FieldWriter<'a, REG, 2, Tps, crate::Safe>;
impl<'a, REG> TpsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Input 0"]
    #[inline(always)]
    pub fn tps00(self) -> &'a mut crate::W<REG> {
        self.variant(Tps::Tps00)
    }
    #[doc = "Input 1"]
    #[inline(always)]
    pub fn tps01(self) -> &'a mut crate::W<REG> {
        self.variant(Tps::Tps01)
    }
    #[doc = "Input 2"]
    #[inline(always)]
    pub fn tps10(self) -> &'a mut crate::W<REG> {
        self.variant(Tps::Tps10)
    }
    #[doc = "Input 3"]
    #[inline(always)]
    pub fn tps11(self) -> &'a mut crate::W<REG> {
        self.variant(Tps::Tps11)
    }
}
#[doc = "Timer Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tie {
    #[doc = "0: Disable"]
    Tie0 = 0,
    #[doc = "1: Enable"]
    Tie1 = 1,
}
impl From<Tie> for bool {
    #[inline(always)]
    fn from(variant: Tie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIE` reader - Timer Interrupt Enable"]
pub type TieR = crate::BitReader<Tie>;
impl TieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tie {
        match self.bits {
            false => Tie::Tie0,
            true => Tie::Tie1,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_tie0(&self) -> bool {
        *self == Tie::Tie0
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_tie1(&self) -> bool {
        *self == Tie::Tie1
    }
}
#[doc = "Field `TIE` writer - Timer Interrupt Enable"]
pub type TieW<'a, REG> = crate::BitWriter<'a, REG, Tie>;
impl<'a, REG> TieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn tie0(self) -> &'a mut crate::W<REG> {
        self.variant(Tie::Tie0)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn tie1(self) -> &'a mut crate::W<REG> {
        self.variant(Tie::Tie1)
    }
}
#[doc = "Timer Compare Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tcf {
    #[doc = "0: CNR != (CMR + 1)"]
    Tcf0 = 0,
    #[doc = "1: CNR = (CMR + 1)"]
    Tcf1 = 1,
}
impl From<Tcf> for bool {
    #[inline(always)]
    fn from(variant: Tcf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TCF` reader - Timer Compare Flag"]
pub type TcfR = crate::BitReader<Tcf>;
impl TcfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tcf {
        match self.bits {
            false => Tcf::Tcf0,
            true => Tcf::Tcf1,
        }
    }
    #[doc = "CNR != (CMR + 1)"]
    #[inline(always)]
    pub fn is_tcf0(&self) -> bool {
        *self == Tcf::Tcf0
    }
    #[doc = "CNR = (CMR + 1)"]
    #[inline(always)]
    pub fn is_tcf1(&self) -> bool {
        *self == Tcf::Tcf1
    }
}
#[doc = "Field `TCF` writer - Timer Compare Flag"]
pub type TcfW<'a, REG> = crate::BitWriter1C<'a, REG, Tcf>;
impl<'a, REG> TcfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "CNR != (CMR + 1)"]
    #[inline(always)]
    pub fn tcf0(self) -> &'a mut crate::W<REG> {
        self.variant(Tcf::Tcf0)
    }
    #[doc = "CNR = (CMR + 1)"]
    #[inline(always)]
    pub fn tcf1(self) -> &'a mut crate::W<REG> {
        self.variant(Tcf::Tcf1)
    }
}
#[doc = "Timer DMA Request Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdre {
    #[doc = "0: Disable"]
    Trde0 = 0,
    #[doc = "1: Enable"]
    Trde1 = 1,
}
impl From<Tdre> for bool {
    #[inline(always)]
    fn from(variant: Tdre) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TDRE` reader - Timer DMA Request Enable"]
pub type TdreR = crate::BitReader<Tdre>;
impl TdreR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tdre {
        match self.bits {
            false => Tdre::Trde0,
            true => Tdre::Trde1,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_trde0(&self) -> bool {
        *self == Tdre::Trde0
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_trde1(&self) -> bool {
        *self == Tdre::Trde1
    }
}
#[doc = "Field `TDRE` writer - Timer DMA Request Enable"]
pub type TdreW<'a, REG> = crate::BitWriter<'a, REG, Tdre>;
impl<'a, REG> TdreW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn trde0(self) -> &'a mut crate::W<REG> {
        self.variant(Tdre::Trde0)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn trde1(self) -> &'a mut crate::W<REG> {
        self.variant(Tdre::Trde1)
    }
}
impl R {
    #[doc = "Bit 0 - Timer Enable"]
    #[inline(always)]
    pub fn ten(&self) -> TenR {
        TenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Timer Mode Select"]
    #[inline(always)]
    pub fn tms(&self) -> TmsR {
        TmsR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Timer Free-Running Counter"]
    #[inline(always)]
    pub fn tfc(&self) -> TfcR {
        TfcR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Timer Pin Polarity"]
    #[inline(always)]
    pub fn tpp(&self) -> TppR {
        TppR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bits 4:5 - Timer Pin Select"]
    #[inline(always)]
    pub fn tps(&self) -> TpsR {
        TpsR::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bit 6 - Timer Interrupt Enable"]
    #[inline(always)]
    pub fn tie(&self) -> TieR {
        TieR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Timer Compare Flag"]
    #[inline(always)]
    pub fn tcf(&self) -> TcfR {
        TcfR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Timer DMA Request Enable"]
    #[inline(always)]
    pub fn tdre(&self) -> TdreR {
        TdreR::new(((self.bits >> 8) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Timer Enable"]
    #[inline(always)]
    pub fn ten(&mut self) -> TenW<CsrSpec> {
        TenW::new(self, 0)
    }
    #[doc = "Bit 1 - Timer Mode Select"]
    #[inline(always)]
    pub fn tms(&mut self) -> TmsW<CsrSpec> {
        TmsW::new(self, 1)
    }
    #[doc = "Bit 2 - Timer Free-Running Counter"]
    #[inline(always)]
    pub fn tfc(&mut self) -> TfcW<CsrSpec> {
        TfcW::new(self, 2)
    }
    #[doc = "Bit 3 - Timer Pin Polarity"]
    #[inline(always)]
    pub fn tpp(&mut self) -> TppW<CsrSpec> {
        TppW::new(self, 3)
    }
    #[doc = "Bits 4:5 - Timer Pin Select"]
    #[inline(always)]
    pub fn tps(&mut self) -> TpsW<CsrSpec> {
        TpsW::new(self, 4)
    }
    #[doc = "Bit 6 - Timer Interrupt Enable"]
    #[inline(always)]
    pub fn tie(&mut self) -> TieW<CsrSpec> {
        TieW::new(self, 6)
    }
    #[doc = "Bit 7 - Timer Compare Flag"]
    #[inline(always)]
    pub fn tcf(&mut self) -> TcfW<CsrSpec> {
        TcfW::new(self, 7)
    }
    #[doc = "Bit 8 - Timer DMA Request Enable"]
    #[inline(always)]
    pub fn tdre(&mut self) -> TdreW<CsrSpec> {
        TdreW::new(self, 8)
    }
}
#[doc = "Control Status\n\nYou can [`read`](crate::Reg::read) this register and get [`csr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`csr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CsrSpec;
impl crate::RegisterSpec for CsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`csr::R`](R) reader structure"]
impl crate::Readable for CsrSpec {}
#[doc = "`write(|w| ..)` method takes [`csr::W`](W) writer structure"]
impl crate::Writable for CsrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x80;
}
#[doc = "`reset()` method sets CSR to value 0"]
impl crate::Resettable for CsrSpec {}
