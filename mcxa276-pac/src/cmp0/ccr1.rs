#[doc = "Register `CCR1` reader"]
pub type R = crate::R<Ccr1Spec>;
#[doc = "Register `CCR1` writer"]
pub type W = crate::W<Ccr1Spec>;
#[doc = "Windowing Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowEn {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<WindowEn> for bool {
    #[inline(always)]
    fn from(variant: WindowEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WINDOW_EN` reader - Windowing Enable"]
pub type WindowEnR = crate::BitReader<WindowEn>;
impl WindowEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> WindowEn {
        match self.bits {
            false => WindowEn::Disable,
            true => WindowEn::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == WindowEn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == WindowEn::Enable
    }
}
#[doc = "Field `WINDOW_EN` writer - Windowing Enable"]
pub type WindowEnW<'a, REG> = crate::BitWriter<'a, REG, WindowEn>;
impl<'a, REG> WindowEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(WindowEn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(WindowEn::Enable)
    }
}
#[doc = "Sampling Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleEn {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<SampleEn> for bool {
    #[inline(always)]
    fn from(variant: SampleEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SAMPLE_EN` reader - Sampling Enable"]
pub type SampleEnR = crate::BitReader<SampleEn>;
impl SampleEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SampleEn {
        match self.bits {
            false => SampleEn::Disable,
            true => SampleEn::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == SampleEn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == SampleEn::Enable
    }
}
#[doc = "Field `SAMPLE_EN` writer - Sampling Enable"]
pub type SampleEnW<'a, REG> = crate::BitWriter<'a, REG, SampleEn>;
impl<'a, REG> SampleEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(SampleEn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(SampleEn::Enable)
    }
}
#[doc = "DMA Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DmaEn {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<DmaEn> for bool {
    #[inline(always)]
    fn from(variant: DmaEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DMA_EN` reader - DMA Enable"]
pub type DmaEnR = crate::BitReader<DmaEn>;
impl DmaEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> DmaEn {
        match self.bits {
            false => DmaEn::Disable,
            true => DmaEn::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == DmaEn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == DmaEn::Enable
    }
}
#[doc = "Field `DMA_EN` writer - DMA Enable"]
pub type DmaEnW<'a, REG> = crate::BitWriter<'a, REG, DmaEn>;
impl<'a, REG> DmaEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(DmaEn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(DmaEn::Enable)
    }
}
#[doc = "Comparator Invert\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoutInv {
    #[doc = "0: Do not invert"]
    NoInvert = 0,
    #[doc = "1: Invert"]
    Invert = 1,
}
impl From<CoutInv> for bool {
    #[inline(always)]
    fn from(variant: CoutInv) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `COUT_INV` reader - Comparator Invert"]
pub type CoutInvR = crate::BitReader<CoutInv>;
impl CoutInvR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CoutInv {
        match self.bits {
            false => CoutInv::NoInvert,
            true => CoutInv::Invert,
        }
    }
    #[doc = "Do not invert"]
    #[inline(always)]
    pub fn is_no_invert(&self) -> bool {
        *self == CoutInv::NoInvert
    }
    #[doc = "Invert"]
    #[inline(always)]
    pub fn is_invert(&self) -> bool {
        *self == CoutInv::Invert
    }
}
#[doc = "Field `COUT_INV` writer - Comparator Invert"]
pub type CoutInvW<'a, REG> = crate::BitWriter<'a, REG, CoutInv>;
impl<'a, REG> CoutInvW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Do not invert"]
    #[inline(always)]
    pub fn no_invert(self) -> &'a mut crate::W<REG> {
        self.variant(CoutInv::NoInvert)
    }
    #[doc = "Invert"]
    #[inline(always)]
    pub fn invert(self) -> &'a mut crate::W<REG> {
        self.variant(CoutInv::Invert)
    }
}
#[doc = "Comparator Output Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoutSel {
    #[doc = "0: Use COUT (filtered)"]
    Cout = 0,
    #[doc = "1: Use COUTA (unfiltered)"]
    Couta = 1,
}
impl From<CoutSel> for bool {
    #[inline(always)]
    fn from(variant: CoutSel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `COUT_SEL` reader - Comparator Output Select"]
pub type CoutSelR = crate::BitReader<CoutSel>;
impl CoutSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CoutSel {
        match self.bits {
            false => CoutSel::Cout,
            true => CoutSel::Couta,
        }
    }
    #[doc = "Use COUT (filtered)"]
    #[inline(always)]
    pub fn is_cout(&self) -> bool {
        *self == CoutSel::Cout
    }
    #[doc = "Use COUTA (unfiltered)"]
    #[inline(always)]
    pub fn is_couta(&self) -> bool {
        *self == CoutSel::Couta
    }
}
#[doc = "Field `COUT_SEL` writer - Comparator Output Select"]
pub type CoutSelW<'a, REG> = crate::BitWriter<'a, REG, CoutSel>;
impl<'a, REG> CoutSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Use COUT (filtered)"]
    #[inline(always)]
    pub fn cout(self) -> &'a mut crate::W<REG> {
        self.variant(CoutSel::Cout)
    }
    #[doc = "Use COUTA (unfiltered)"]
    #[inline(always)]
    pub fn couta(self) -> &'a mut crate::W<REG> {
        self.variant(CoutSel::Couta)
    }
}
#[doc = "Comparator Output Pin Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoutPen {
    #[doc = "0: Not available"]
    Unavailable = 0,
    #[doc = "1: Available"]
    Available = 1,
}
impl From<CoutPen> for bool {
    #[inline(always)]
    fn from(variant: CoutPen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `COUT_PEN` reader - Comparator Output Pin Enable"]
pub type CoutPenR = crate::BitReader<CoutPen>;
impl CoutPenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CoutPen {
        match self.bits {
            false => CoutPen::Unavailable,
            true => CoutPen::Available,
        }
    }
    #[doc = "Not available"]
    #[inline(always)]
    pub fn is_unavailable(&self) -> bool {
        *self == CoutPen::Unavailable
    }
    #[doc = "Available"]
    #[inline(always)]
    pub fn is_available(&self) -> bool {
        *self == CoutPen::Available
    }
}
#[doc = "Field `COUT_PEN` writer - Comparator Output Pin Enable"]
pub type CoutPenW<'a, REG> = crate::BitWriter<'a, REG, CoutPen>;
impl<'a, REG> CoutPenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not available"]
    #[inline(always)]
    pub fn unavailable(self) -> &'a mut crate::W<REG> {
        self.variant(CoutPen::Unavailable)
    }
    #[doc = "Available"]
    #[inline(always)]
    pub fn available(self) -> &'a mut crate::W<REG> {
        self.variant(CoutPen::Available)
    }
}
#[doc = "COUTA_OW Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoutaOwen {
    #[doc = "0: COUTA holds the last sampled value."]
    Sampled = 0,
    #[doc = "1: Enables the COUTA signal value to be defined by COUTA_OW."]
    CoutaOw = 1,
}
impl From<CoutaOwen> for bool {
    #[inline(always)]
    fn from(variant: CoutaOwen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `COUTA_OWEN` reader - COUTA_OW Enable"]
pub type CoutaOwenR = crate::BitReader<CoutaOwen>;
impl CoutaOwenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CoutaOwen {
        match self.bits {
            false => CoutaOwen::Sampled,
            true => CoutaOwen::CoutaOw,
        }
    }
    #[doc = "COUTA holds the last sampled value."]
    #[inline(always)]
    pub fn is_sampled(&self) -> bool {
        *self == CoutaOwen::Sampled
    }
    #[doc = "Enables the COUTA signal value to be defined by COUTA_OW."]
    #[inline(always)]
    pub fn is_couta_ow(&self) -> bool {
        *self == CoutaOwen::CoutaOw
    }
}
#[doc = "Field `COUTA_OWEN` writer - COUTA_OW Enable"]
pub type CoutaOwenW<'a, REG> = crate::BitWriter<'a, REG, CoutaOwen>;
impl<'a, REG> CoutaOwenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "COUTA holds the last sampled value."]
    #[inline(always)]
    pub fn sampled(self) -> &'a mut crate::W<REG> {
        self.variant(CoutaOwen::Sampled)
    }
    #[doc = "Enables the COUTA signal value to be defined by COUTA_OW."]
    #[inline(always)]
    pub fn couta_ow(self) -> &'a mut crate::W<REG> {
        self.variant(CoutaOwen::CoutaOw)
    }
}
#[doc = "COUTA Output Level for Closed Window\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoutaOw {
    #[doc = "0: COUTA is 0"]
    Couta0 = 0,
    #[doc = "1: COUTA is 1"]
    Couta1 = 1,
}
impl From<CoutaOw> for bool {
    #[inline(always)]
    fn from(variant: CoutaOw) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `COUTA_OW` reader - COUTA Output Level for Closed Window"]
pub type CoutaOwR = crate::BitReader<CoutaOw>;
impl CoutaOwR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CoutaOw {
        match self.bits {
            false => CoutaOw::Couta0,
            true => CoutaOw::Couta1,
        }
    }
    #[doc = "COUTA is 0"]
    #[inline(always)]
    pub fn is_couta_0(&self) -> bool {
        *self == CoutaOw::Couta0
    }
    #[doc = "COUTA is 1"]
    #[inline(always)]
    pub fn is_couta_1(&self) -> bool {
        *self == CoutaOw::Couta1
    }
}
#[doc = "Field `COUTA_OW` writer - COUTA Output Level for Closed Window"]
pub type CoutaOwW<'a, REG> = crate::BitWriter<'a, REG, CoutaOw>;
impl<'a, REG> CoutaOwW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "COUTA is 0"]
    #[inline(always)]
    pub fn couta_0(self) -> &'a mut crate::W<REG> {
        self.variant(CoutaOw::Couta0)
    }
    #[doc = "COUTA is 1"]
    #[inline(always)]
    pub fn couta_1(self) -> &'a mut crate::W<REG> {
        self.variant(CoutaOw::Couta1)
    }
}
#[doc = "WINDOW/SAMPLE Signal Invert\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowInv {
    #[doc = "0: Do not invert"]
    NoInvert = 0,
    #[doc = "1: Invert"]
    Invert = 1,
}
impl From<WindowInv> for bool {
    #[inline(always)]
    fn from(variant: WindowInv) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WINDOW_INV` reader - WINDOW/SAMPLE Signal Invert"]
pub type WindowInvR = crate::BitReader<WindowInv>;
impl WindowInvR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> WindowInv {
        match self.bits {
            false => WindowInv::NoInvert,
            true => WindowInv::Invert,
        }
    }
    #[doc = "Do not invert"]
    #[inline(always)]
    pub fn is_no_invert(&self) -> bool {
        *self == WindowInv::NoInvert
    }
    #[doc = "Invert"]
    #[inline(always)]
    pub fn is_invert(&self) -> bool {
        *self == WindowInv::Invert
    }
}
#[doc = "Field `WINDOW_INV` writer - WINDOW/SAMPLE Signal Invert"]
pub type WindowInvW<'a, REG> = crate::BitWriter<'a, REG, WindowInv>;
impl<'a, REG> WindowInvW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Do not invert"]
    #[inline(always)]
    pub fn no_invert(self) -> &'a mut crate::W<REG> {
        self.variant(WindowInv::NoInvert)
    }
    #[doc = "Invert"]
    #[inline(always)]
    pub fn invert(self) -> &'a mut crate::W<REG> {
        self.variant(WindowInv::Invert)
    }
}
#[doc = "COUT Event Window Close\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowCls {
    #[doc = "0: COUT event cannot close the window"]
    NoClose = 0,
    #[doc = "1: COUT event can close the window"]
    Close = 1,
}
impl From<WindowCls> for bool {
    #[inline(always)]
    fn from(variant: WindowCls) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WINDOW_CLS` reader - COUT Event Window Close"]
pub type WindowClsR = crate::BitReader<WindowCls>;
impl WindowClsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> WindowCls {
        match self.bits {
            false => WindowCls::NoClose,
            true => WindowCls::Close,
        }
    }
    #[doc = "COUT event cannot close the window"]
    #[inline(always)]
    pub fn is_no_close(&self) -> bool {
        *self == WindowCls::NoClose
    }
    #[doc = "COUT event can close the window"]
    #[inline(always)]
    pub fn is_close(&self) -> bool {
        *self == WindowCls::Close
    }
}
#[doc = "Field `WINDOW_CLS` writer - COUT Event Window Close"]
pub type WindowClsW<'a, REG> = crate::BitWriter<'a, REG, WindowCls>;
impl<'a, REG> WindowClsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "COUT event cannot close the window"]
    #[inline(always)]
    pub fn no_close(self) -> &'a mut crate::W<REG> {
        self.variant(WindowCls::NoClose)
    }
    #[doc = "COUT event can close the window"]
    #[inline(always)]
    pub fn close(self) -> &'a mut crate::W<REG> {
        self.variant(WindowCls::Close)
    }
}
#[doc = "COUT Event Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum EvtSel {
    #[doc = "0: Rising edge"]
    Rising = 0,
    #[doc = "1: Falling edge"]
    Falling = 1,
    #[doc = "2: Both edges"]
    Both = 2,
}
impl From<EvtSel> for u8 {
    #[inline(always)]
    fn from(variant: EvtSel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for EvtSel {
    type Ux = u8;
}
impl crate::IsEnum for EvtSel {}
#[doc = "Field `EVT_SEL` reader - COUT Event Select"]
pub type EvtSelR = crate::FieldReader<EvtSel>;
impl EvtSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<EvtSel> {
        match self.bits {
            0 => Some(EvtSel::Rising),
            1 => Some(EvtSel::Falling),
            2 => Some(EvtSel::Both),
            _ => None,
        }
    }
    #[doc = "Rising edge"]
    #[inline(always)]
    pub fn is_rising(&self) -> bool {
        *self == EvtSel::Rising
    }
    #[doc = "Falling edge"]
    #[inline(always)]
    pub fn is_falling(&self) -> bool {
        *self == EvtSel::Falling
    }
    #[doc = "Both edges"]
    #[inline(always)]
    pub fn is_both(&self) -> bool {
        *self == EvtSel::Both
    }
}
#[doc = "Field `EVT_SEL` writer - COUT Event Select"]
pub type EvtSelW<'a, REG> = crate::FieldWriter<'a, REG, 2, EvtSel>;
impl<'a, REG> EvtSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Rising edge"]
    #[inline(always)]
    pub fn rising(self) -> &'a mut crate::W<REG> {
        self.variant(EvtSel::Rising)
    }
    #[doc = "Falling edge"]
    #[inline(always)]
    pub fn falling(self) -> &'a mut crate::W<REG> {
        self.variant(EvtSel::Falling)
    }
    #[doc = "Both edges"]
    #[inline(always)]
    pub fn both(self) -> &'a mut crate::W<REG> {
        self.variant(EvtSel::Both)
    }
}
#[doc = "Functional Clock Source Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FuncClkSel {
    #[doc = "0: Select functional clock source 0"]
    Func0 = 0,
    #[doc = "1: Select functional clock source 1"]
    Func1 = 1,
    #[doc = "2: Select functional clock source 2"]
    Func2 = 2,
    #[doc = "3: Select functional clock source 3"]
    Func3 = 3,
}
impl From<FuncClkSel> for u8 {
    #[inline(always)]
    fn from(variant: FuncClkSel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for FuncClkSel {
    type Ux = u8;
}
impl crate::IsEnum for FuncClkSel {}
#[doc = "Field `FUNC_CLK_SEL` reader - Functional Clock Source Select"]
pub type FuncClkSelR = crate::FieldReader<FuncClkSel>;
impl FuncClkSelR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FuncClkSel {
        match self.bits {
            0 => FuncClkSel::Func0,
            1 => FuncClkSel::Func1,
            2 => FuncClkSel::Func2,
            3 => FuncClkSel::Func3,
            _ => unreachable!(),
        }
    }
    #[doc = "Select functional clock source 0"]
    #[inline(always)]
    pub fn is_func0(&self) -> bool {
        *self == FuncClkSel::Func0
    }
    #[doc = "Select functional clock source 1"]
    #[inline(always)]
    pub fn is_func1(&self) -> bool {
        *self == FuncClkSel::Func1
    }
    #[doc = "Select functional clock source 2"]
    #[inline(always)]
    pub fn is_func2(&self) -> bool {
        *self == FuncClkSel::Func2
    }
    #[doc = "Select functional clock source 3"]
    #[inline(always)]
    pub fn is_func3(&self) -> bool {
        *self == FuncClkSel::Func3
    }
}
#[doc = "Field `FUNC_CLK_SEL` writer - Functional Clock Source Select"]
pub type FuncClkSelW<'a, REG> = crate::FieldWriter<'a, REG, 2, FuncClkSel, crate::Safe>;
impl<'a, REG> FuncClkSelW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Select functional clock source 0"]
    #[inline(always)]
    pub fn func0(self) -> &'a mut crate::W<REG> {
        self.variant(FuncClkSel::Func0)
    }
    #[doc = "Select functional clock source 1"]
    #[inline(always)]
    pub fn func1(self) -> &'a mut crate::W<REG> {
        self.variant(FuncClkSel::Func1)
    }
    #[doc = "Select functional clock source 2"]
    #[inline(always)]
    pub fn func2(self) -> &'a mut crate::W<REG> {
        self.variant(FuncClkSel::Func2)
    }
    #[doc = "Select functional clock source 3"]
    #[inline(always)]
    pub fn func3(self) -> &'a mut crate::W<REG> {
        self.variant(FuncClkSel::Func3)
    }
}
#[doc = "Filter Sample Count\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FiltCnt {
    #[doc = "0: Filter is bypassed: COUT = COUTA"]
    Bypassed = 0,
    #[doc = "1: 1 consecutive sample (Comparator output is simply sampled.)"]
    Sample1 = 1,
    #[doc = "2: 2 consecutive samples"]
    Sample2 = 2,
    #[doc = "3: 3 consecutive samples"]
    Sample3 = 3,
    #[doc = "4: 4 consecutive samples"]
    Sample4 = 4,
    #[doc = "5: 5 consecutive samples"]
    Sample5 = 5,
    #[doc = "6: 6 consecutive samples"]
    Sample6 = 6,
    #[doc = "7: 7 consecutive samples"]
    Sample7 = 7,
}
impl From<FiltCnt> for u8 {
    #[inline(always)]
    fn from(variant: FiltCnt) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for FiltCnt {
    type Ux = u8;
}
impl crate::IsEnum for FiltCnt {}
#[doc = "Field `FILT_CNT` reader - Filter Sample Count"]
pub type FiltCntR = crate::FieldReader<FiltCnt>;
impl FiltCntR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FiltCnt {
        match self.bits {
            0 => FiltCnt::Bypassed,
            1 => FiltCnt::Sample1,
            2 => FiltCnt::Sample2,
            3 => FiltCnt::Sample3,
            4 => FiltCnt::Sample4,
            5 => FiltCnt::Sample5,
            6 => FiltCnt::Sample6,
            7 => FiltCnt::Sample7,
            _ => unreachable!(),
        }
    }
    #[doc = "Filter is bypassed: COUT = COUTA"]
    #[inline(always)]
    pub fn is_bypassed(&self) -> bool {
        *self == FiltCnt::Bypassed
    }
    #[doc = "1 consecutive sample (Comparator output is simply sampled.)"]
    #[inline(always)]
    pub fn is_sample_1(&self) -> bool {
        *self == FiltCnt::Sample1
    }
    #[doc = "2 consecutive samples"]
    #[inline(always)]
    pub fn is_sample_2(&self) -> bool {
        *self == FiltCnt::Sample2
    }
    #[doc = "3 consecutive samples"]
    #[inline(always)]
    pub fn is_sample_3(&self) -> bool {
        *self == FiltCnt::Sample3
    }
    #[doc = "4 consecutive samples"]
    #[inline(always)]
    pub fn is_sample_4(&self) -> bool {
        *self == FiltCnt::Sample4
    }
    #[doc = "5 consecutive samples"]
    #[inline(always)]
    pub fn is_sample_5(&self) -> bool {
        *self == FiltCnt::Sample5
    }
    #[doc = "6 consecutive samples"]
    #[inline(always)]
    pub fn is_sample_6(&self) -> bool {
        *self == FiltCnt::Sample6
    }
    #[doc = "7 consecutive samples"]
    #[inline(always)]
    pub fn is_sample_7(&self) -> bool {
        *self == FiltCnt::Sample7
    }
}
#[doc = "Field `FILT_CNT` writer - Filter Sample Count"]
pub type FiltCntW<'a, REG> = crate::FieldWriter<'a, REG, 3, FiltCnt, crate::Safe>;
impl<'a, REG> FiltCntW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Filter is bypassed: COUT = COUTA"]
    #[inline(always)]
    pub fn bypassed(self) -> &'a mut crate::W<REG> {
        self.variant(FiltCnt::Bypassed)
    }
    #[doc = "1 consecutive sample (Comparator output is simply sampled.)"]
    #[inline(always)]
    pub fn sample_1(self) -> &'a mut crate::W<REG> {
        self.variant(FiltCnt::Sample1)
    }
    #[doc = "2 consecutive samples"]
    #[inline(always)]
    pub fn sample_2(self) -> &'a mut crate::W<REG> {
        self.variant(FiltCnt::Sample2)
    }
    #[doc = "3 consecutive samples"]
    #[inline(always)]
    pub fn sample_3(self) -> &'a mut crate::W<REG> {
        self.variant(FiltCnt::Sample3)
    }
    #[doc = "4 consecutive samples"]
    #[inline(always)]
    pub fn sample_4(self) -> &'a mut crate::W<REG> {
        self.variant(FiltCnt::Sample4)
    }
    #[doc = "5 consecutive samples"]
    #[inline(always)]
    pub fn sample_5(self) -> &'a mut crate::W<REG> {
        self.variant(FiltCnt::Sample5)
    }
    #[doc = "6 consecutive samples"]
    #[inline(always)]
    pub fn sample_6(self) -> &'a mut crate::W<REG> {
        self.variant(FiltCnt::Sample6)
    }
    #[doc = "7 consecutive samples"]
    #[inline(always)]
    pub fn sample_7(self) -> &'a mut crate::W<REG> {
        self.variant(FiltCnt::Sample7)
    }
}
#[doc = "Field `FILT_PER` reader - Filter Sample Period"]
pub type FiltPerR = crate::FieldReader;
#[doc = "Field `FILT_PER` writer - Filter Sample Period"]
pub type FiltPerW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bit 0 - Windowing Enable"]
    #[inline(always)]
    pub fn window_en(&self) -> WindowEnR {
        WindowEnR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Sampling Enable"]
    #[inline(always)]
    pub fn sample_en(&self) -> SampleEnR {
        SampleEnR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - DMA Enable"]
    #[inline(always)]
    pub fn dma_en(&self) -> DmaEnR {
        DmaEnR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Comparator Invert"]
    #[inline(always)]
    pub fn cout_inv(&self) -> CoutInvR {
        CoutInvR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Comparator Output Select"]
    #[inline(always)]
    pub fn cout_sel(&self) -> CoutSelR {
        CoutSelR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Comparator Output Pin Enable"]
    #[inline(always)]
    pub fn cout_pen(&self) -> CoutPenR {
        CoutPenR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - COUTA_OW Enable"]
    #[inline(always)]
    pub fn couta_owen(&self) -> CoutaOwenR {
        CoutaOwenR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - COUTA Output Level for Closed Window"]
    #[inline(always)]
    pub fn couta_ow(&self) -> CoutaOwR {
        CoutaOwR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - WINDOW/SAMPLE Signal Invert"]
    #[inline(always)]
    pub fn window_inv(&self) -> WindowInvR {
        WindowInvR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - COUT Event Window Close"]
    #[inline(always)]
    pub fn window_cls(&self) -> WindowClsR {
        WindowClsR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bits 10:11 - COUT Event Select"]
    #[inline(always)]
    pub fn evt_sel(&self) -> EvtSelR {
        EvtSelR::new(((self.bits >> 10) & 3) as u8)
    }
    #[doc = "Bits 12:13 - Functional Clock Source Select"]
    #[inline(always)]
    pub fn func_clk_sel(&self) -> FuncClkSelR {
        FuncClkSelR::new(((self.bits >> 12) & 3) as u8)
    }
    #[doc = "Bits 16:18 - Filter Sample Count"]
    #[inline(always)]
    pub fn filt_cnt(&self) -> FiltCntR {
        FiltCntR::new(((self.bits >> 16) & 7) as u8)
    }
    #[doc = "Bits 24:31 - Filter Sample Period"]
    #[inline(always)]
    pub fn filt_per(&self) -> FiltPerR {
        FiltPerR::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Windowing Enable"]
    #[inline(always)]
    pub fn window_en(&mut self) -> WindowEnW<Ccr1Spec> {
        WindowEnW::new(self, 0)
    }
    #[doc = "Bit 1 - Sampling Enable"]
    #[inline(always)]
    pub fn sample_en(&mut self) -> SampleEnW<Ccr1Spec> {
        SampleEnW::new(self, 1)
    }
    #[doc = "Bit 2 - DMA Enable"]
    #[inline(always)]
    pub fn dma_en(&mut self) -> DmaEnW<Ccr1Spec> {
        DmaEnW::new(self, 2)
    }
    #[doc = "Bit 3 - Comparator Invert"]
    #[inline(always)]
    pub fn cout_inv(&mut self) -> CoutInvW<Ccr1Spec> {
        CoutInvW::new(self, 3)
    }
    #[doc = "Bit 4 - Comparator Output Select"]
    #[inline(always)]
    pub fn cout_sel(&mut self) -> CoutSelW<Ccr1Spec> {
        CoutSelW::new(self, 4)
    }
    #[doc = "Bit 5 - Comparator Output Pin Enable"]
    #[inline(always)]
    pub fn cout_pen(&mut self) -> CoutPenW<Ccr1Spec> {
        CoutPenW::new(self, 5)
    }
    #[doc = "Bit 6 - COUTA_OW Enable"]
    #[inline(always)]
    pub fn couta_owen(&mut self) -> CoutaOwenW<Ccr1Spec> {
        CoutaOwenW::new(self, 6)
    }
    #[doc = "Bit 7 - COUTA Output Level for Closed Window"]
    #[inline(always)]
    pub fn couta_ow(&mut self) -> CoutaOwW<Ccr1Spec> {
        CoutaOwW::new(self, 7)
    }
    #[doc = "Bit 8 - WINDOW/SAMPLE Signal Invert"]
    #[inline(always)]
    pub fn window_inv(&mut self) -> WindowInvW<Ccr1Spec> {
        WindowInvW::new(self, 8)
    }
    #[doc = "Bit 9 - COUT Event Window Close"]
    #[inline(always)]
    pub fn window_cls(&mut self) -> WindowClsW<Ccr1Spec> {
        WindowClsW::new(self, 9)
    }
    #[doc = "Bits 10:11 - COUT Event Select"]
    #[inline(always)]
    pub fn evt_sel(&mut self) -> EvtSelW<Ccr1Spec> {
        EvtSelW::new(self, 10)
    }
    #[doc = "Bits 12:13 - Functional Clock Source Select"]
    #[inline(always)]
    pub fn func_clk_sel(&mut self) -> FuncClkSelW<Ccr1Spec> {
        FuncClkSelW::new(self, 12)
    }
    #[doc = "Bits 16:18 - Filter Sample Count"]
    #[inline(always)]
    pub fn filt_cnt(&mut self) -> FiltCntW<Ccr1Spec> {
        FiltCntW::new(self, 16)
    }
    #[doc = "Bits 24:31 - Filter Sample Period"]
    #[inline(always)]
    pub fn filt_per(&mut self) -> FiltPerW<Ccr1Spec> {
        FiltPerW::new(self, 24)
    }
}
#[doc = "Comparator Control Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`ccr1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ccr1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ccr1Spec;
impl crate::RegisterSpec for Ccr1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ccr1::R`](R) reader structure"]
impl crate::Readable for Ccr1Spec {}
#[doc = "`write(|w| ..)` method takes [`ccr1::W`](W) writer structure"]
impl crate::Writable for Ccr1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CCR1 to value 0"]
impl crate::Resettable for Ccr1Spec {}
