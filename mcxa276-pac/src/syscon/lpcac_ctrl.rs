#[doc = "Register `LPCAC_CTRL` reader"]
pub type R = crate::R<LpcacCtrlSpec>;
#[doc = "Register `LPCAC_CTRL` writer"]
pub type W = crate::W<LpcacCtrlSpec>;
#[doc = "Disables/enables the cache function.\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisLpcac {
    #[doc = "0: Enabled"]
    Enable = 0,
    #[doc = "1: Disabled"]
    Disable = 1,
}
impl From<DisLpcac> for bool {
    #[inline(always)]
    fn from(variant: DisLpcac) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DIS_LPCAC` reader - Disables/enables the cache function."]
pub type DisLpcacR = crate::BitReader<DisLpcac>;
impl DisLpcacR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> DisLpcac {
        match self.bits {
            false => DisLpcac::Enable,
            true => DisLpcac::Disable,
        }
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == DisLpcac::Enable
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == DisLpcac::Disable
    }
}
#[doc = "Field `DIS_LPCAC` writer - Disables/enables the cache function."]
pub type DisLpcacW<'a, REG> = crate::BitWriter<'a, REG, DisLpcac>;
impl<'a, REG> DisLpcacW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(DisLpcac::Enable)
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(DisLpcac::Disable)
    }
}
#[doc = "Clears the cache function.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClrLpcac {
    #[doc = "0: Unclears the cache"]
    Enable = 0,
    #[doc = "1: Clears the cache"]
    Disable = 1,
}
impl From<ClrLpcac> for bool {
    #[inline(always)]
    fn from(variant: ClrLpcac) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CLR_LPCAC` reader - Clears the cache function."]
pub type ClrLpcacR = crate::BitReader<ClrLpcac>;
impl ClrLpcacR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> ClrLpcac {
        match self.bits {
            false => ClrLpcac::Enable,
            true => ClrLpcac::Disable,
        }
    }
    #[doc = "Unclears the cache"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == ClrLpcac::Enable
    }
    #[doc = "Clears the cache"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == ClrLpcac::Disable
    }
}
#[doc = "Field `CLR_LPCAC` writer - Clears the cache function."]
pub type ClrLpcacW<'a, REG> = crate::BitWriter<'a, REG, ClrLpcac>;
impl<'a, REG> ClrLpcacW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Unclears the cache"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(ClrLpcac::Enable)
    }
    #[doc = "Clears the cache"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(ClrLpcac::Disable)
    }
}
#[doc = "Forces no allocation.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrcNoAlloc {
    #[doc = "0: Forces allocation"]
    Enable = 0,
    #[doc = "1: Forces no allocation"]
    Disable = 1,
}
impl From<FrcNoAlloc> for bool {
    #[inline(always)]
    fn from(variant: FrcNoAlloc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FRC_NO_ALLOC` reader - Forces no allocation."]
pub type FrcNoAllocR = crate::BitReader<FrcNoAlloc>;
impl FrcNoAllocR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FrcNoAlloc {
        match self.bits {
            false => FrcNoAlloc::Enable,
            true => FrcNoAlloc::Disable,
        }
    }
    #[doc = "Forces allocation"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == FrcNoAlloc::Enable
    }
    #[doc = "Forces no allocation"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == FrcNoAlloc::Disable
    }
}
#[doc = "Field `FRC_NO_ALLOC` writer - Forces no allocation."]
pub type FrcNoAllocW<'a, REG> = crate::BitWriter<'a, REG, FrcNoAlloc>;
impl<'a, REG> FrcNoAllocW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Forces allocation"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(FrcNoAlloc::Enable)
    }
    #[doc = "Forces no allocation"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(FrcNoAlloc::Disable)
    }
}
#[doc = "Disable LPCAC Write Through Buffer.\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisLpcacWtbf {
    #[doc = "0: Enables write through buffer"]
    Disable = 0,
    #[doc = "1: Disables write through buffer"]
    Enable = 1,
}
impl From<DisLpcacWtbf> for bool {
    #[inline(always)]
    fn from(variant: DisLpcacWtbf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DIS_LPCAC_WTBF` reader - Disable LPCAC Write Through Buffer."]
pub type DisLpcacWtbfR = crate::BitReader<DisLpcacWtbf>;
impl DisLpcacWtbfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> DisLpcacWtbf {
        match self.bits {
            false => DisLpcacWtbf::Disable,
            true => DisLpcacWtbf::Enable,
        }
    }
    #[doc = "Enables write through buffer"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == DisLpcacWtbf::Disable
    }
    #[doc = "Disables write through buffer"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == DisLpcacWtbf::Enable
    }
}
#[doc = "Field `DIS_LPCAC_WTBF` writer - Disable LPCAC Write Through Buffer."]
pub type DisLpcacWtbfW<'a, REG> = crate::BitWriter<'a, REG, DisLpcacWtbf>;
impl<'a, REG> DisLpcacWtbfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enables write through buffer"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(DisLpcacWtbf::Disable)
    }
    #[doc = "Disables write through buffer"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(DisLpcacWtbf::Enable)
    }
}
#[doc = "Limit LPCAC Write Through Buffer.\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimLpcacWtbf {
    #[doc = "0: Write buffer enabled when transaction is bufferable."]
    Disable = 0,
    #[doc = "1: Write buffer enabled when transaction is cacheable and bufferable"]
    Enable = 1,
}
impl From<LimLpcacWtbf> for bool {
    #[inline(always)]
    fn from(variant: LimLpcacWtbf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LIM_LPCAC_WTBF` reader - Limit LPCAC Write Through Buffer."]
pub type LimLpcacWtbfR = crate::BitReader<LimLpcacWtbf>;
impl LimLpcacWtbfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> LimLpcacWtbf {
        match self.bits {
            false => LimLpcacWtbf::Disable,
            true => LimLpcacWtbf::Enable,
        }
    }
    #[doc = "Write buffer enabled when transaction is bufferable."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == LimLpcacWtbf::Disable
    }
    #[doc = "Write buffer enabled when transaction is cacheable and bufferable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == LimLpcacWtbf::Enable
    }
}
#[doc = "Field `LIM_LPCAC_WTBF` writer - Limit LPCAC Write Through Buffer."]
pub type LimLpcacWtbfW<'a, REG> = crate::BitWriter<'a, REG, LimLpcacWtbf>;
impl<'a, REG> LimLpcacWtbfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Write buffer enabled when transaction is bufferable."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(LimLpcacWtbf::Disable)
    }
    #[doc = "Write buffer enabled when transaction is cacheable and bufferable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(LimLpcacWtbf::Enable)
    }
}
#[doc = "LPCAC XOM(eXecute-Only-Memory) attribute control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LpcacXom {
    #[doc = "0: Disabled."]
    Disable = 0,
    #[doc = "1: Enabled."]
    Enable = 1,
}
impl From<LpcacXom> for bool {
    #[inline(always)]
    fn from(variant: LpcacXom) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPCAC_XOM` reader - LPCAC XOM(eXecute-Only-Memory) attribute control"]
pub type LpcacXomR = crate::BitReader<LpcacXom>;
impl LpcacXomR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> LpcacXom {
        match self.bits {
            false => LpcacXom::Disable,
            true => LpcacXom::Enable,
        }
    }
    #[doc = "Disabled."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == LpcacXom::Disable
    }
    #[doc = "Enabled."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == LpcacXom::Enable
    }
}
#[doc = "Field `LPCAC_XOM` writer - LPCAC XOM(eXecute-Only-Memory) attribute control"]
pub type LpcacXomW<'a, REG> = crate::BitWriter<'a, REG, LpcacXom>;
impl<'a, REG> LpcacXomW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(LpcacXom::Disable)
    }
    #[doc = "Enabled."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(LpcacXom::Enable)
    }
}
#[doc = "Request LPCAC memories.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LpcacMemReq {
    #[doc = "0: Configure shared memories RAMX1 as general memories."]
    Disable = 0,
    #[doc = "1: Configure shared memories RAMX1 as LPCAC memories, write one lock until a system reset."]
    Enable = 1,
}
impl From<LpcacMemReq> for bool {
    #[inline(always)]
    fn from(variant: LpcacMemReq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LPCAC_MEM_REQ` reader - Request LPCAC memories."]
pub type LpcacMemReqR = crate::BitReader<LpcacMemReq>;
impl LpcacMemReqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> LpcacMemReq {
        match self.bits {
            false => LpcacMemReq::Disable,
            true => LpcacMemReq::Enable,
        }
    }
    #[doc = "Configure shared memories RAMX1 as general memories."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == LpcacMemReq::Disable
    }
    #[doc = "Configure shared memories RAMX1 as LPCAC memories, write one lock until a system reset."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == LpcacMemReq::Enable
    }
}
#[doc = "Field `LPCAC_MEM_REQ` writer - Request LPCAC memories."]
pub type LpcacMemReqW<'a, REG> = crate::BitWriter<'a, REG, LpcacMemReq>;
impl<'a, REG> LpcacMemReqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Configure shared memories RAMX1 as general memories."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(LpcacMemReq::Disable)
    }
    #[doc = "Configure shared memories RAMX1 as LPCAC memories, write one lock until a system reset."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(LpcacMemReq::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - Disables/enables the cache function."]
    #[inline(always)]
    pub fn dis_lpcac(&self) -> DisLpcacR {
        DisLpcacR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Clears the cache function."]
    #[inline(always)]
    pub fn clr_lpcac(&self) -> ClrLpcacR {
        ClrLpcacR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Forces no allocation."]
    #[inline(always)]
    pub fn frc_no_alloc(&self) -> FrcNoAllocR {
        FrcNoAllocR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 4 - Disable LPCAC Write Through Buffer."]
    #[inline(always)]
    pub fn dis_lpcac_wtbf(&self) -> DisLpcacWtbfR {
        DisLpcacWtbfR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Limit LPCAC Write Through Buffer."]
    #[inline(always)]
    pub fn lim_lpcac_wtbf(&self) -> LimLpcacWtbfR {
        LimLpcacWtbfR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 7 - LPCAC XOM(eXecute-Only-Memory) attribute control"]
    #[inline(always)]
    pub fn lpcac_xom(&self) -> LpcacXomR {
        LpcacXomR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Request LPCAC memories."]
    #[inline(always)]
    pub fn lpcac_mem_req(&self) -> LpcacMemReqR {
        LpcacMemReqR::new(((self.bits >> 8) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Disables/enables the cache function."]
    #[inline(always)]
    pub fn dis_lpcac(&mut self) -> DisLpcacW<LpcacCtrlSpec> {
        DisLpcacW::new(self, 0)
    }
    #[doc = "Bit 1 - Clears the cache function."]
    #[inline(always)]
    pub fn clr_lpcac(&mut self) -> ClrLpcacW<LpcacCtrlSpec> {
        ClrLpcacW::new(self, 1)
    }
    #[doc = "Bit 2 - Forces no allocation."]
    #[inline(always)]
    pub fn frc_no_alloc(&mut self) -> FrcNoAllocW<LpcacCtrlSpec> {
        FrcNoAllocW::new(self, 2)
    }
    #[doc = "Bit 4 - Disable LPCAC Write Through Buffer."]
    #[inline(always)]
    pub fn dis_lpcac_wtbf(&mut self) -> DisLpcacWtbfW<LpcacCtrlSpec> {
        DisLpcacWtbfW::new(self, 4)
    }
    #[doc = "Bit 5 - Limit LPCAC Write Through Buffer."]
    #[inline(always)]
    pub fn lim_lpcac_wtbf(&mut self) -> LimLpcacWtbfW<LpcacCtrlSpec> {
        LimLpcacWtbfW::new(self, 5)
    }
    #[doc = "Bit 7 - LPCAC XOM(eXecute-Only-Memory) attribute control"]
    #[inline(always)]
    pub fn lpcac_xom(&mut self) -> LpcacXomW<LpcacCtrlSpec> {
        LpcacXomW::new(self, 7)
    }
    #[doc = "Bit 8 - Request LPCAC memories."]
    #[inline(always)]
    pub fn lpcac_mem_req(&mut self) -> LpcacMemReqW<LpcacCtrlSpec> {
        LpcacMemReqW::new(self, 8)
    }
}
#[doc = "LPCAC Control\n\nYou can [`read`](crate::Reg::read) this register and get [`lpcac_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lpcac_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LpcacCtrlSpec;
impl crate::RegisterSpec for LpcacCtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`lpcac_ctrl::R`](R) reader structure"]
impl crate::Readable for LpcacCtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`lpcac_ctrl::W`](W) writer structure"]
impl crate::Writable for LpcacCtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LPCAC_CTRL to value 0x31"]
impl crate::Resettable for LpcacCtrlSpec {
    const RESET_VALUE: u32 = 0x31;
}
