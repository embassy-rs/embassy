#[doc = "Register `REMAP` reader"]
pub type R = crate::R<RemapSpec>;
#[doc = "Register `REMAP` writer"]
pub type W = crate::W<RemapSpec>;
#[doc = "RAMX0 address remap for CPU System bus\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Cpu0Sbus {
    #[doc = "0: RAMX0: alias space is disabled."]
    Cpu0Sbus0 = 0,
    #[doc = "1: RAMX0: alias space is enabled. It's linear address space from bottom of system ram. The start address is 0x20000000 + (system ram size - RAMX size)*1024."]
    Cpu0Sbus1 = 1,
}
impl From<Cpu0Sbus> for u8 {
    #[inline(always)]
    fn from(variant: Cpu0Sbus) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Cpu0Sbus {
    type Ux = u8;
}
impl crate::IsEnum for Cpu0Sbus {}
#[doc = "Field `CPU0_SBUS` reader - RAMX0 address remap for CPU System bus"]
pub type Cpu0SbusR = crate::FieldReader<Cpu0Sbus>;
impl Cpu0SbusR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Cpu0Sbus> {
        match self.bits {
            0 => Some(Cpu0Sbus::Cpu0Sbus0),
            1 => Some(Cpu0Sbus::Cpu0Sbus1),
            _ => None,
        }
    }
    #[doc = "RAMX0: alias space is disabled."]
    #[inline(always)]
    pub fn is_cpu0_sbus_0(&self) -> bool {
        *self == Cpu0Sbus::Cpu0Sbus0
    }
    #[doc = "RAMX0: alias space is enabled. It's linear address space from bottom of system ram. The start address is 0x20000000 + (system ram size - RAMX size)*1024."]
    #[inline(always)]
    pub fn is_cpu0_sbus_1(&self) -> bool {
        *self == Cpu0Sbus::Cpu0Sbus1
    }
}
#[doc = "Field `CPU0_SBUS` writer - RAMX0 address remap for CPU System bus"]
pub type Cpu0SbusW<'a, REG> = crate::FieldWriter<'a, REG, 2, Cpu0Sbus>;
impl<'a, REG> Cpu0SbusW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "RAMX0: alias space is disabled."]
    #[inline(always)]
    pub fn cpu0_sbus_0(self) -> &'a mut crate::W<REG> {
        self.variant(Cpu0Sbus::Cpu0Sbus0)
    }
    #[doc = "RAMX0: alias space is enabled. It's linear address space from bottom of system ram. The start address is 0x20000000 + (system ram size - RAMX size)*1024."]
    #[inline(always)]
    pub fn cpu0_sbus_1(self) -> &'a mut crate::W<REG> {
        self.variant(Cpu0Sbus::Cpu0Sbus1)
    }
}
#[doc = "RAMX0 address remap for SmartDMA D-BUS\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SmartDmaD {
    #[doc = "0: RAMX0: alias space is disabled."]
    SmartDmaD0 = 0,
    #[doc = "1: RAMX0: same alias space as CPU0_SBUS"]
    SmartDmaD1 = 1,
}
impl From<SmartDmaD> for u8 {
    #[inline(always)]
    fn from(variant: SmartDmaD) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for SmartDmaD {
    type Ux = u8;
}
impl crate::IsEnum for SmartDmaD {}
#[doc = "Field `SmartDMA_D` reader - RAMX0 address remap for SmartDMA D-BUS"]
pub type SmartDmaDR = crate::FieldReader<SmartDmaD>;
impl SmartDmaDR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<SmartDmaD> {
        match self.bits {
            0 => Some(SmartDmaD::SmartDmaD0),
            1 => Some(SmartDmaD::SmartDmaD1),
            _ => None,
        }
    }
    #[doc = "RAMX0: alias space is disabled."]
    #[inline(always)]
    pub fn is_smart_dma_d_0(&self) -> bool {
        *self == SmartDmaD::SmartDmaD0
    }
    #[doc = "RAMX0: same alias space as CPU0_SBUS"]
    #[inline(always)]
    pub fn is_smart_dma_d_1(&self) -> bool {
        *self == SmartDmaD::SmartDmaD1
    }
}
#[doc = "Field `SmartDMA_D` writer - RAMX0 address remap for SmartDMA D-BUS"]
pub type SmartDmaDW<'a, REG> = crate::FieldWriter<'a, REG, 2, SmartDmaD>;
impl<'a, REG> SmartDmaDW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "RAMX0: alias space is disabled."]
    #[inline(always)]
    pub fn smart_dma_d_0(self) -> &'a mut crate::W<REG> {
        self.variant(SmartDmaD::SmartDmaD0)
    }
    #[doc = "RAMX0: same alias space as CPU0_SBUS"]
    #[inline(always)]
    pub fn smart_dma_d_1(self) -> &'a mut crate::W<REG> {
        self.variant(SmartDmaD::SmartDmaD1)
    }
}
#[doc = "RAMX0 address remap for SmartDMA I-BUS\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SmartDmaI {
    #[doc = "0: RAMX0: alias space is disabled."]
    SmartDmaI0 = 0,
    #[doc = "1: RAMX0: same alias space as CPU0_SBUS"]
    SmartDmaI1 = 1,
}
impl From<SmartDmaI> for u8 {
    #[inline(always)]
    fn from(variant: SmartDmaI) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for SmartDmaI {
    type Ux = u8;
}
impl crate::IsEnum for SmartDmaI {}
#[doc = "Field `SmartDMA_I` reader - RAMX0 address remap for SmartDMA I-BUS"]
pub type SmartDmaIR = crate::FieldReader<SmartDmaI>;
impl SmartDmaIR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<SmartDmaI> {
        match self.bits {
            0 => Some(SmartDmaI::SmartDmaI0),
            1 => Some(SmartDmaI::SmartDmaI1),
            _ => None,
        }
    }
    #[doc = "RAMX0: alias space is disabled."]
    #[inline(always)]
    pub fn is_smart_dma_i_0(&self) -> bool {
        *self == SmartDmaI::SmartDmaI0
    }
    #[doc = "RAMX0: same alias space as CPU0_SBUS"]
    #[inline(always)]
    pub fn is_smart_dma_i_1(&self) -> bool {
        *self == SmartDmaI::SmartDmaI1
    }
}
#[doc = "Field `SmartDMA_I` writer - RAMX0 address remap for SmartDMA I-BUS"]
pub type SmartDmaIW<'a, REG> = crate::FieldWriter<'a, REG, 2, SmartDmaI>;
impl<'a, REG> SmartDmaIW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "RAMX0: alias space is disabled."]
    #[inline(always)]
    pub fn smart_dma_i_0(self) -> &'a mut crate::W<REG> {
        self.variant(SmartDmaI::SmartDmaI0)
    }
    #[doc = "RAMX0: same alias space as CPU0_SBUS"]
    #[inline(always)]
    pub fn smart_dma_i_1(self) -> &'a mut crate::W<REG> {
        self.variant(SmartDmaI::SmartDmaI1)
    }
}
#[doc = "RAMX0 address remap for DMA0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Dma0 {
    #[doc = "0: RAMX0: alias space is disabled."]
    Dma0_0 = 0,
    #[doc = "1: RAMX0: same alias space as CPU0_SBUS"]
    Dma0_1 = 1,
}
impl From<Dma0> for u8 {
    #[inline(always)]
    fn from(variant: Dma0) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Dma0 {
    type Ux = u8;
}
impl crate::IsEnum for Dma0 {}
#[doc = "Field `DMA0` reader - RAMX0 address remap for DMA0"]
pub type Dma0R = crate::FieldReader<Dma0>;
impl Dma0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Dma0> {
        match self.bits {
            0 => Some(Dma0::Dma0_0),
            1 => Some(Dma0::Dma0_1),
            _ => None,
        }
    }
    #[doc = "RAMX0: alias space is disabled."]
    #[inline(always)]
    pub fn is_dma0_0(&self) -> bool {
        *self == Dma0::Dma0_0
    }
    #[doc = "RAMX0: same alias space as CPU0_SBUS"]
    #[inline(always)]
    pub fn is_dma0_1(&self) -> bool {
        *self == Dma0::Dma0_1
    }
}
#[doc = "Field `DMA0` writer - RAMX0 address remap for DMA0"]
pub type Dma0W<'a, REG> = crate::FieldWriter<'a, REG, 2, Dma0>;
impl<'a, REG> Dma0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "RAMX0: alias space is disabled."]
    #[inline(always)]
    pub fn dma0_0(self) -> &'a mut crate::W<REG> {
        self.variant(Dma0::Dma0_0)
    }
    #[doc = "RAMX0: same alias space as CPU0_SBUS"]
    #[inline(always)]
    pub fn dma0_1(self) -> &'a mut crate::W<REG> {
        self.variant(Dma0::Dma0_1)
    }
}
#[doc = "RAMX0 address remap for PKC\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pkc {
    #[doc = "0: RAMX0: alias space is disabled."]
    Pkc0 = 0,
    #[doc = "1: RAMX0: same alias space as CPU0_SBUS"]
    Pkc1 = 1,
}
impl From<Pkc> for u8 {
    #[inline(always)]
    fn from(variant: Pkc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Pkc {
    type Ux = u8;
}
impl crate::IsEnum for Pkc {}
#[doc = "Field `PKC` reader - RAMX0 address remap for PKC"]
pub type PkcR = crate::FieldReader<Pkc>;
impl PkcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Pkc> {
        match self.bits {
            0 => Some(Pkc::Pkc0),
            1 => Some(Pkc::Pkc1),
            _ => None,
        }
    }
    #[doc = "RAMX0: alias space is disabled."]
    #[inline(always)]
    pub fn is_pkc_0(&self) -> bool {
        *self == Pkc::Pkc0
    }
    #[doc = "RAMX0: same alias space as CPU0_SBUS"]
    #[inline(always)]
    pub fn is_pkc_1(&self) -> bool {
        *self == Pkc::Pkc1
    }
}
#[doc = "Field `PKC` writer - RAMX0 address remap for PKC"]
pub type PkcW<'a, REG> = crate::FieldWriter<'a, REG, 2, Pkc>;
impl<'a, REG> PkcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "RAMX0: alias space is disabled."]
    #[inline(always)]
    pub fn pkc_0(self) -> &'a mut crate::W<REG> {
        self.variant(Pkc::Pkc0)
    }
    #[doc = "RAMX0: same alias space as CPU0_SBUS"]
    #[inline(always)]
    pub fn pkc_1(self) -> &'a mut crate::W<REG> {
        self.variant(Pkc::Pkc1)
    }
}
#[doc = "RAMX0 address remap for USB0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Usb0 {
    #[doc = "0: RAMX0: alias space is disabled."]
    Usb0_0 = 0,
    #[doc = "1: RAMX0: same alias space as CPU0_SBUS"]
    Usb0_1 = 1,
}
impl From<Usb0> for u8 {
    #[inline(always)]
    fn from(variant: Usb0) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Usb0 {
    type Ux = u8;
}
impl crate::IsEnum for Usb0 {}
#[doc = "Field `USB0` reader - RAMX0 address remap for USB0"]
pub type Usb0R = crate::FieldReader<Usb0>;
impl Usb0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Usb0> {
        match self.bits {
            0 => Some(Usb0::Usb0_0),
            1 => Some(Usb0::Usb0_1),
            _ => None,
        }
    }
    #[doc = "RAMX0: alias space is disabled."]
    #[inline(always)]
    pub fn is_usb0_0(&self) -> bool {
        *self == Usb0::Usb0_0
    }
    #[doc = "RAMX0: same alias space as CPU0_SBUS"]
    #[inline(always)]
    pub fn is_usb0_1(&self) -> bool {
        *self == Usb0::Usb0_1
    }
}
#[doc = "Field `USB0` writer - RAMX0 address remap for USB0"]
pub type Usb0W<'a, REG> = crate::FieldWriter<'a, REG, 2, Usb0>;
impl<'a, REG> Usb0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "RAMX0: alias space is disabled."]
    #[inline(always)]
    pub fn usb0_0(self) -> &'a mut crate::W<REG> {
        self.variant(Usb0::Usb0_0)
    }
    #[doc = "RAMX0: same alias space as CPU0_SBUS"]
    #[inline(always)]
    pub fn usb0_1(self) -> &'a mut crate::W<REG> {
        self.variant(Usb0::Usb0_1)
    }
}
#[doc = "This 1-bit field provides a mechanism to limit writes to the this register to protect its contents. Once set, this bit remains asserted until a system reset.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lock {
    #[doc = "0: This register is not locked and can be altered."]
    Lock0 = 0,
    #[doc = "1: This register is locked and cannot be altered until a system reset."]
    Lock1 = 1,
}
impl From<Lock> for bool {
    #[inline(always)]
    fn from(variant: Lock) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LOCK` reader - This 1-bit field provides a mechanism to limit writes to the this register to protect its contents. Once set, this bit remains asserted until a system reset."]
pub type LockR = crate::BitReader<Lock>;
impl LockR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lock {
        match self.bits {
            false => Lock::Lock0,
            true => Lock::Lock1,
        }
    }
    #[doc = "This register is not locked and can be altered."]
    #[inline(always)]
    pub fn is_lock_0(&self) -> bool {
        *self == Lock::Lock0
    }
    #[doc = "This register is locked and cannot be altered until a system reset."]
    #[inline(always)]
    pub fn is_lock_1(&self) -> bool {
        *self == Lock::Lock1
    }
}
#[doc = "Field `LOCK` writer - This 1-bit field provides a mechanism to limit writes to the this register to protect its contents. Once set, this bit remains asserted until a system reset."]
pub type LockW<'a, REG> = crate::BitWriter<'a, REG, Lock>;
impl<'a, REG> LockW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "This register is not locked and can be altered."]
    #[inline(always)]
    pub fn lock_0(self) -> &'a mut crate::W<REG> {
        self.variant(Lock::Lock0)
    }
    #[doc = "This register is locked and cannot be altered until a system reset."]
    #[inline(always)]
    pub fn lock_1(self) -> &'a mut crate::W<REG> {
        self.variant(Lock::Lock1)
    }
}
impl R {
    #[doc = "Bits 2:3 - RAMX0 address remap for CPU System bus"]
    #[inline(always)]
    pub fn cpu0_sbus(&self) -> Cpu0SbusR {
        Cpu0SbusR::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:5 - RAMX0 address remap for SmartDMA D-BUS"]
    #[inline(always)]
    pub fn smart_dma_d(&self) -> SmartDmaDR {
        SmartDmaDR::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 6:7 - RAMX0 address remap for SmartDMA I-BUS"]
    #[inline(always)]
    pub fn smart_dma_i(&self) -> SmartDmaIR {
        SmartDmaIR::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bits 8:9 - RAMX0 address remap for DMA0"]
    #[inline(always)]
    pub fn dma0(&self) -> Dma0R {
        Dma0R::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bits 12:13 - RAMX0 address remap for PKC"]
    #[inline(always)]
    pub fn pkc(&self) -> PkcR {
        PkcR::new(((self.bits >> 12) & 3) as u8)
    }
    #[doc = "Bits 24:25 - RAMX0 address remap for USB0"]
    #[inline(always)]
    pub fn usb0(&self) -> Usb0R {
        Usb0R::new(((self.bits >> 24) & 3) as u8)
    }
    #[doc = "Bit 31 - This 1-bit field provides a mechanism to limit writes to the this register to protect its contents. Once set, this bit remains asserted until a system reset."]
    #[inline(always)]
    pub fn lock(&self) -> LockR {
        LockR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 2:3 - RAMX0 address remap for CPU System bus"]
    #[inline(always)]
    pub fn cpu0_sbus(&mut self) -> Cpu0SbusW<RemapSpec> {
        Cpu0SbusW::new(self, 2)
    }
    #[doc = "Bits 4:5 - RAMX0 address remap for SmartDMA D-BUS"]
    #[inline(always)]
    pub fn smart_dma_d(&mut self) -> SmartDmaDW<RemapSpec> {
        SmartDmaDW::new(self, 4)
    }
    #[doc = "Bits 6:7 - RAMX0 address remap for SmartDMA I-BUS"]
    #[inline(always)]
    pub fn smart_dma_i(&mut self) -> SmartDmaIW<RemapSpec> {
        SmartDmaIW::new(self, 6)
    }
    #[doc = "Bits 8:9 - RAMX0 address remap for DMA0"]
    #[inline(always)]
    pub fn dma0(&mut self) -> Dma0W<RemapSpec> {
        Dma0W::new(self, 8)
    }
    #[doc = "Bits 12:13 - RAMX0 address remap for PKC"]
    #[inline(always)]
    pub fn pkc(&mut self) -> PkcW<RemapSpec> {
        PkcW::new(self, 12)
    }
    #[doc = "Bits 24:25 - RAMX0 address remap for USB0"]
    #[inline(always)]
    pub fn usb0(&mut self) -> Usb0W<RemapSpec> {
        Usb0W::new(self, 24)
    }
    #[doc = "Bit 31 - This 1-bit field provides a mechanism to limit writes to the this register to protect its contents. Once set, this bit remains asserted until a system reset."]
    #[inline(always)]
    pub fn lock(&mut self) -> LockW<RemapSpec> {
        LockW::new(self, 31)
    }
}
#[doc = "AHB Matrix Remap Control\n\nYou can [`read`](crate::Reg::read) this register and get [`remap::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`remap::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RemapSpec;
impl crate::RegisterSpec for RemapSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`remap::R`](R) reader structure"]
impl crate::Readable for RemapSpec {}
#[doc = "`write(|w| ..)` method takes [`remap::W`](W) writer structure"]
impl crate::Writable for RemapSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets REMAP to value 0"]
impl crate::Resettable for RemapSpec {}
