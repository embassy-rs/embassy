#[doc = "Register `SPLLCTRL` reader"]
pub type R = crate::R<SpllctrlSpec>;
#[doc = "Register `SPLLCTRL` writer"]
pub type W = crate::W<SpllctrlSpec>;
#[doc = "Field `SELR` reader - Bandwidth select R (resistor) value."]
pub type SelrR = crate::FieldReader;
#[doc = "Field `SELR` writer - Bandwidth select R (resistor) value."]
pub type SelrW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `SELI` reader - Bandwidth select I (interation) value."]
pub type SeliR = crate::FieldReader;
#[doc = "Field `SELI` writer - Bandwidth select I (interation) value."]
pub type SeliW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `SELP` reader - Bandwidth select P (proportional) value."]
pub type SelpR = crate::FieldReader;
#[doc = "Field `SELP` writer - Bandwidth select P (proportional) value."]
pub type SelpW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Bypass of the divide-by-2 divider\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bypasspostdiv2 {
    #[doc = "0: Use the divide-by-2 divider in the post-divider."]
    NotBypassed = 0,
    #[doc = "1: Bypass of the divide-by-2 divider in the post-divider."]
    Bypassed = 1,
}
impl From<Bypasspostdiv2> for bool {
    #[inline(always)]
    fn from(variant: Bypasspostdiv2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BYPASSPOSTDIV2` reader - Bypass of the divide-by-2 divider"]
pub type Bypasspostdiv2R = crate::BitReader<Bypasspostdiv2>;
impl Bypasspostdiv2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Bypasspostdiv2 {
        match self.bits {
            false => Bypasspostdiv2::NotBypassed,
            true => Bypasspostdiv2::Bypassed,
        }
    }
    #[doc = "Use the divide-by-2 divider in the post-divider."]
    #[inline(always)]
    pub fn is_not_bypassed(&self) -> bool {
        *self == Bypasspostdiv2::NotBypassed
    }
    #[doc = "Bypass of the divide-by-2 divider in the post-divider."]
    #[inline(always)]
    pub fn is_bypassed(&self) -> bool {
        *self == Bypasspostdiv2::Bypassed
    }
}
#[doc = "Field `BYPASSPOSTDIV2` writer - Bypass of the divide-by-2 divider"]
pub type Bypasspostdiv2W<'a, REG> = crate::BitWriter<'a, REG, Bypasspostdiv2>;
impl<'a, REG> Bypasspostdiv2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Use the divide-by-2 divider in the post-divider."]
    #[inline(always)]
    pub fn not_bypassed(self) -> &'a mut crate::W<REG> {
        self.variant(Bypasspostdiv2::NotBypassed)
    }
    #[doc = "Bypass of the divide-by-2 divider in the post-divider."]
    #[inline(always)]
    pub fn bypassed(self) -> &'a mut crate::W<REG> {
        self.variant(Bypasspostdiv2::Bypassed)
    }
}
#[doc = "Up Limiter.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limupoff {
    #[doc = "0: Application set to non Spectrum and Fractional applications."]
    Disabled = 0,
    #[doc = "1: Application set to Spectrum and Fractional applications."]
    Enabled = 1,
}
impl From<Limupoff> for bool {
    #[inline(always)]
    fn from(variant: Limupoff) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LIMUPOFF` reader - Up Limiter."]
pub type LimupoffR = crate::BitReader<Limupoff>;
impl LimupoffR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Limupoff {
        match self.bits {
            false => Limupoff::Disabled,
            true => Limupoff::Enabled,
        }
    }
    #[doc = "Application set to non Spectrum and Fractional applications."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Limupoff::Disabled
    }
    #[doc = "Application set to Spectrum and Fractional applications."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Limupoff::Enabled
    }
}
#[doc = "Field `LIMUPOFF` writer - Up Limiter."]
pub type LimupoffW<'a, REG> = crate::BitWriter<'a, REG, Limupoff>;
impl<'a, REG> LimupoffW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Application set to non Spectrum and Fractional applications."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Limupoff::Disabled)
    }
    #[doc = "Application set to Spectrum and Fractional applications."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Limupoff::Enabled)
    }
}
#[doc = "Control of the bandwidth of the PLL.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Banddirect {
    #[doc = "0: The bandwidth is changed synchronously with the feedback-divider"]
    Disabled = 0,
    #[doc = "1: Modifies the bandwidth of the PLL directly"]
    Enabled = 1,
}
impl From<Banddirect> for bool {
    #[inline(always)]
    fn from(variant: Banddirect) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BANDDIRECT` reader - Control of the bandwidth of the PLL."]
pub type BanddirectR = crate::BitReader<Banddirect>;
impl BanddirectR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Banddirect {
        match self.bits {
            false => Banddirect::Disabled,
            true => Banddirect::Enabled,
        }
    }
    #[doc = "The bandwidth is changed synchronously with the feedback-divider"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Banddirect::Disabled
    }
    #[doc = "Modifies the bandwidth of the PLL directly"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Banddirect::Enabled
    }
}
#[doc = "Field `BANDDIRECT` writer - Control of the bandwidth of the PLL."]
pub type BanddirectW<'a, REG> = crate::BitWriter<'a, REG, Banddirect>;
impl<'a, REG> BanddirectW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The bandwidth is changed synchronously with the feedback-divider"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Banddirect::Disabled)
    }
    #[doc = "Modifies the bandwidth of the PLL directly"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Banddirect::Enabled)
    }
}
#[doc = "Bypass of the pre-divider.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bypassprediv {
    #[doc = "0: Use the pre-divider"]
    Disabled = 0,
    #[doc = "1: Bypass of the pre-divider"]
    Enabled = 1,
}
impl From<Bypassprediv> for bool {
    #[inline(always)]
    fn from(variant: Bypassprediv) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BYPASSPREDIV` reader - Bypass of the pre-divider."]
pub type BypasspredivR = crate::BitReader<Bypassprediv>;
impl BypasspredivR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Bypassprediv {
        match self.bits {
            false => Bypassprediv::Disabled,
            true => Bypassprediv::Enabled,
        }
    }
    #[doc = "Use the pre-divider"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Bypassprediv::Disabled
    }
    #[doc = "Bypass of the pre-divider"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Bypassprediv::Enabled
    }
}
#[doc = "Field `BYPASSPREDIV` writer - Bypass of the pre-divider."]
pub type BypasspredivW<'a, REG> = crate::BitWriter<'a, REG, Bypassprediv>;
impl<'a, REG> BypasspredivW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Use the pre-divider"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Bypassprediv::Disabled)
    }
    #[doc = "Bypass of the pre-divider"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Bypassprediv::Enabled)
    }
}
#[doc = "Bypass of the post-divider.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bypasspostdiv {
    #[doc = "0: Use the post-divider"]
    Disabled = 0,
    #[doc = "1: Bypass of the post-divider"]
    Enabled = 1,
}
impl From<Bypasspostdiv> for bool {
    #[inline(always)]
    fn from(variant: Bypasspostdiv) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BYPASSPOSTDIV` reader - Bypass of the post-divider."]
pub type BypasspostdivR = crate::BitReader<Bypasspostdiv>;
impl BypasspostdivR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Bypasspostdiv {
        match self.bits {
            false => Bypasspostdiv::Disabled,
            true => Bypasspostdiv::Enabled,
        }
    }
    #[doc = "Use the post-divider"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Bypasspostdiv::Disabled
    }
    #[doc = "Bypass of the post-divider"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Bypasspostdiv::Enabled
    }
}
#[doc = "Field `BYPASSPOSTDIV` writer - Bypass of the post-divider."]
pub type BypasspostdivW<'a, REG> = crate::BitWriter<'a, REG, Bypasspostdiv>;
impl<'a, REG> BypasspostdivW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Use the post-divider"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Bypasspostdiv::Disabled)
    }
    #[doc = "Bypass of the post-divider"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Bypasspostdiv::Enabled)
    }
}
#[doc = "Free Running Mode Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Frm {
    #[doc = "0: Free running mode disabled"]
    Disabled = 0,
    #[doc = "1: Free running mode enabled"]
    Enabled = 1,
}
impl From<Frm> for bool {
    #[inline(always)]
    fn from(variant: Frm) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FRM` reader - Free Running Mode Enable"]
pub type FrmR = crate::BitReader<Frm>;
impl FrmR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Frm {
        match self.bits {
            false => Frm::Disabled,
            true => Frm::Enabled,
        }
    }
    #[doc = "Free running mode disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Frm::Disabled
    }
    #[doc = "Free running mode enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Frm::Enabled
    }
}
#[doc = "Field `FRM` writer - Free Running Mode Enable"]
pub type FrmW<'a, REG> = crate::BitWriter<'a, REG, Frm>;
impl<'a, REG> FrmW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Free running mode disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Frm::Disabled)
    }
    #[doc = "Free running mode enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Frm::Enabled)
    }
}
#[doc = "Clock Source\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Source {
    #[doc = "0: SOSC"]
    Sosc = 0,
    #[doc = "1: FIRC 45 MHz clock. FIRC_SCLK_PERIPH_EN needs to be set to use FIRC 45 MHz clock."]
    Firc = 1,
    #[doc = "2: ROSC"]
    Rsvd = 2,
    #[doc = "3: SIRC 12 MHz clock"]
    Sirc = 3,
}
impl From<Source> for u8 {
    #[inline(always)]
    fn from(variant: Source) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Source {
    type Ux = u8;
}
impl crate::IsEnum for Source {}
#[doc = "Field `SOURCE` reader - Clock Source"]
pub type SourceR = crate::FieldReader<Source>;
impl SourceR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Source {
        match self.bits {
            0 => Source::Sosc,
            1 => Source::Firc,
            2 => Source::Rsvd,
            3 => Source::Sirc,
            _ => unreachable!(),
        }
    }
    #[doc = "SOSC"]
    #[inline(always)]
    pub fn is_sosc(&self) -> bool {
        *self == Source::Sosc
    }
    #[doc = "FIRC 45 MHz clock. FIRC_SCLK_PERIPH_EN needs to be set to use FIRC 45 MHz clock."]
    #[inline(always)]
    pub fn is_firc(&self) -> bool {
        *self == Source::Firc
    }
    #[doc = "ROSC"]
    #[inline(always)]
    pub fn is_rsvd(&self) -> bool {
        *self == Source::Rsvd
    }
    #[doc = "SIRC 12 MHz clock"]
    #[inline(always)]
    pub fn is_sirc(&self) -> bool {
        *self == Source::Sirc
    }
}
#[doc = "Field `SOURCE` writer - Clock Source"]
pub type SourceW<'a, REG> = crate::FieldWriter<'a, REG, 2, Source, crate::Safe>;
impl<'a, REG> SourceW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "SOSC"]
    #[inline(always)]
    pub fn sosc(self) -> &'a mut crate::W<REG> {
        self.variant(Source::Sosc)
    }
    #[doc = "FIRC 45 MHz clock. FIRC_SCLK_PERIPH_EN needs to be set to use FIRC 45 MHz clock."]
    #[inline(always)]
    pub fn firc(self) -> &'a mut crate::W<REG> {
        self.variant(Source::Firc)
    }
    #[doc = "ROSC"]
    #[inline(always)]
    pub fn rsvd(self) -> &'a mut crate::W<REG> {
        self.variant(Source::Rsvd)
    }
    #[doc = "SIRC 12 MHz clock"]
    #[inline(always)]
    pub fn sirc(self) -> &'a mut crate::W<REG> {
        self.variant(Source::Sirc)
    }
}
impl R {
    #[doc = "Bits 0:3 - Bandwidth select R (resistor) value."]
    #[inline(always)]
    pub fn selr(&self) -> SelrR {
        SelrR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 4:9 - Bandwidth select I (interation) value."]
    #[inline(always)]
    pub fn seli(&self) -> SeliR {
        SeliR::new(((self.bits >> 4) & 0x3f) as u8)
    }
    #[doc = "Bits 10:14 - Bandwidth select P (proportional) value."]
    #[inline(always)]
    pub fn selp(&self) -> SelpR {
        SelpR::new(((self.bits >> 10) & 0x1f) as u8)
    }
    #[doc = "Bit 16 - Bypass of the divide-by-2 divider"]
    #[inline(always)]
    pub fn bypasspostdiv2(&self) -> Bypasspostdiv2R {
        Bypasspostdiv2R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Up Limiter."]
    #[inline(always)]
    pub fn limupoff(&self) -> LimupoffR {
        LimupoffR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Control of the bandwidth of the PLL."]
    #[inline(always)]
    pub fn banddirect(&self) -> BanddirectR {
        BanddirectR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Bypass of the pre-divider."]
    #[inline(always)]
    pub fn bypassprediv(&self) -> BypasspredivR {
        BypasspredivR::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Bypass of the post-divider."]
    #[inline(always)]
    pub fn bypasspostdiv(&self) -> BypasspostdivR {
        BypasspostdivR::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 22 - Free Running Mode Enable"]
    #[inline(always)]
    pub fn frm(&self) -> FrmR {
        FrmR::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bits 25:26 - Clock Source"]
    #[inline(always)]
    pub fn source(&self) -> SourceR {
        SourceR::new(((self.bits >> 25) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Bandwidth select R (resistor) value."]
    #[inline(always)]
    pub fn selr(&mut self) -> SelrW<SpllctrlSpec> {
        SelrW::new(self, 0)
    }
    #[doc = "Bits 4:9 - Bandwidth select I (interation) value."]
    #[inline(always)]
    pub fn seli(&mut self) -> SeliW<SpllctrlSpec> {
        SeliW::new(self, 4)
    }
    #[doc = "Bits 10:14 - Bandwidth select P (proportional) value."]
    #[inline(always)]
    pub fn selp(&mut self) -> SelpW<SpllctrlSpec> {
        SelpW::new(self, 10)
    }
    #[doc = "Bit 16 - Bypass of the divide-by-2 divider"]
    #[inline(always)]
    pub fn bypasspostdiv2(&mut self) -> Bypasspostdiv2W<SpllctrlSpec> {
        Bypasspostdiv2W::new(self, 16)
    }
    #[doc = "Bit 17 - Up Limiter."]
    #[inline(always)]
    pub fn limupoff(&mut self) -> LimupoffW<SpllctrlSpec> {
        LimupoffW::new(self, 17)
    }
    #[doc = "Bit 18 - Control of the bandwidth of the PLL."]
    #[inline(always)]
    pub fn banddirect(&mut self) -> BanddirectW<SpllctrlSpec> {
        BanddirectW::new(self, 18)
    }
    #[doc = "Bit 19 - Bypass of the pre-divider."]
    #[inline(always)]
    pub fn bypassprediv(&mut self) -> BypasspredivW<SpllctrlSpec> {
        BypasspredivW::new(self, 19)
    }
    #[doc = "Bit 20 - Bypass of the post-divider."]
    #[inline(always)]
    pub fn bypasspostdiv(&mut self) -> BypasspostdivW<SpllctrlSpec> {
        BypasspostdivW::new(self, 20)
    }
    #[doc = "Bit 22 - Free Running Mode Enable"]
    #[inline(always)]
    pub fn frm(&mut self) -> FrmW<SpllctrlSpec> {
        FrmW::new(self, 22)
    }
    #[doc = "Bits 25:26 - Clock Source"]
    #[inline(always)]
    pub fn source(&mut self) -> SourceW<SpllctrlSpec> {
        SourceW::new(self, 25)
    }
}
#[doc = "SPLL Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`spllctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SpllctrlSpec;
impl crate::RegisterSpec for SpllctrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`spllctrl::R`](R) reader structure"]
impl crate::Readable for SpllctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`spllctrl::W`](W) writer structure"]
impl crate::Writable for SpllctrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SPLLCTRL to value 0"]
impl crate::Resettable for SpllctrlSpec {}
