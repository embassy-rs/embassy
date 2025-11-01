#[doc = "Register `MCTRL` reader"]
pub type R = crate::R<MctrlSpec>;
#[doc = "Register `MCTRL` writer"]
pub type W = crate::W<MctrlSpec>;
#[doc = "Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Request {
    #[doc = "0: NONE"]
    None = 0,
    #[doc = "1: EMITSTARTADDR"]
    Emitstartaddr = 1,
    #[doc = "2: EMITSTOP"]
    Emitstop = 2,
    #[doc = "3: IBIACKNACK"]
    Ibiacknack = 3,
    #[doc = "4: PROCESSDAA"]
    Processdaa = 4,
    #[doc = "6: Force Exit and Target Reset"]
    Forceexit = 6,
    #[doc = "7: AUTOIBI"]
    Autoibi = 7,
}
impl From<Request> for u8 {
    #[inline(always)]
    fn from(variant: Request) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Request {
    type Ux = u8;
}
impl crate::IsEnum for Request {}
#[doc = "Field `REQUEST` reader - Request"]
pub type RequestR = crate::FieldReader<Request>;
impl RequestR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Request> {
        match self.bits {
            0 => Some(Request::None),
            1 => Some(Request::Emitstartaddr),
            2 => Some(Request::Emitstop),
            3 => Some(Request::Ibiacknack),
            4 => Some(Request::Processdaa),
            6 => Some(Request::Forceexit),
            7 => Some(Request::Autoibi),
            _ => None,
        }
    }
    #[doc = "NONE"]
    #[inline(always)]
    pub fn is_none(&self) -> bool {
        *self == Request::None
    }
    #[doc = "EMITSTARTADDR"]
    #[inline(always)]
    pub fn is_emitstartaddr(&self) -> bool {
        *self == Request::Emitstartaddr
    }
    #[doc = "EMITSTOP"]
    #[inline(always)]
    pub fn is_emitstop(&self) -> bool {
        *self == Request::Emitstop
    }
    #[doc = "IBIACKNACK"]
    #[inline(always)]
    pub fn is_ibiacknack(&self) -> bool {
        *self == Request::Ibiacknack
    }
    #[doc = "PROCESSDAA"]
    #[inline(always)]
    pub fn is_processdaa(&self) -> bool {
        *self == Request::Processdaa
    }
    #[doc = "Force Exit and Target Reset"]
    #[inline(always)]
    pub fn is_forceexit(&self) -> bool {
        *self == Request::Forceexit
    }
    #[doc = "AUTOIBI"]
    #[inline(always)]
    pub fn is_autoibi(&self) -> bool {
        *self == Request::Autoibi
    }
}
#[doc = "Field `REQUEST` writer - Request"]
pub type RequestW<'a, REG> = crate::FieldWriter<'a, REG, 3, Request>;
impl<'a, REG> RequestW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "NONE"]
    #[inline(always)]
    pub fn none(self) -> &'a mut crate::W<REG> {
        self.variant(Request::None)
    }
    #[doc = "EMITSTARTADDR"]
    #[inline(always)]
    pub fn emitstartaddr(self) -> &'a mut crate::W<REG> {
        self.variant(Request::Emitstartaddr)
    }
    #[doc = "EMITSTOP"]
    #[inline(always)]
    pub fn emitstop(self) -> &'a mut crate::W<REG> {
        self.variant(Request::Emitstop)
    }
    #[doc = "IBIACKNACK"]
    #[inline(always)]
    pub fn ibiacknack(self) -> &'a mut crate::W<REG> {
        self.variant(Request::Ibiacknack)
    }
    #[doc = "PROCESSDAA"]
    #[inline(always)]
    pub fn processdaa(self) -> &'a mut crate::W<REG> {
        self.variant(Request::Processdaa)
    }
    #[doc = "Force Exit and Target Reset"]
    #[inline(always)]
    pub fn forceexit(self) -> &'a mut crate::W<REG> {
        self.variant(Request::Forceexit)
    }
    #[doc = "AUTOIBI"]
    #[inline(always)]
    pub fn autoibi(self) -> &'a mut crate::W<REG> {
        self.variant(Request::Autoibi)
    }
}
#[doc = "Bus Type with EmitStartAddr\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Type {
    #[doc = "0: I3C"]
    I3c = 0,
    #[doc = "1: I2C"]
    I2c = 1,
    #[doc = "2: DDR"]
    Ddr = 2,
}
impl From<Type> for u8 {
    #[inline(always)]
    fn from(variant: Type) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Type {
    type Ux = u8;
}
impl crate::IsEnum for Type {}
#[doc = "Field `TYPE` reader - Bus Type with EmitStartAddr"]
pub type TypeR = crate::FieldReader<Type>;
impl TypeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Type> {
        match self.bits {
            0 => Some(Type::I3c),
            1 => Some(Type::I2c),
            2 => Some(Type::Ddr),
            _ => None,
        }
    }
    #[doc = "I3C"]
    #[inline(always)]
    pub fn is_i3c(&self) -> bool {
        *self == Type::I3c
    }
    #[doc = "I2C"]
    #[inline(always)]
    pub fn is_i2c(&self) -> bool {
        *self == Type::I2c
    }
    #[doc = "DDR"]
    #[inline(always)]
    pub fn is_ddr(&self) -> bool {
        *self == Type::Ddr
    }
}
#[doc = "Field `TYPE` writer - Bus Type with EmitStartAddr"]
pub type TypeW<'a, REG> = crate::FieldWriter<'a, REG, 2, Type>;
impl<'a, REG> TypeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "I3C"]
    #[inline(always)]
    pub fn i3c(self) -> &'a mut crate::W<REG> {
        self.variant(Type::I3c)
    }
    #[doc = "I2C"]
    #[inline(always)]
    pub fn i2c(self) -> &'a mut crate::W<REG> {
        self.variant(Type::I2c)
    }
    #[doc = "DDR"]
    #[inline(always)]
    pub fn ddr(self) -> &'a mut crate::W<REG> {
        self.variant(Type::Ddr)
    }
}
#[doc = "In-Band Interrupt Response\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Ibiresp {
    #[doc = "0: ACK (acknowledge)"]
    Ack = 0,
    #[doc = "1: NACK (reject)"]
    Nack = 1,
    #[doc = "2: Acknowledge with mandatory byte"]
    AckWithMandatory = 2,
    #[doc = "3: Manual"]
    Manual = 3,
}
impl From<Ibiresp> for u8 {
    #[inline(always)]
    fn from(variant: Ibiresp) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Ibiresp {
    type Ux = u8;
}
impl crate::IsEnum for Ibiresp {}
#[doc = "Field `IBIRESP` reader - In-Band Interrupt Response"]
pub type IbirespR = crate::FieldReader<Ibiresp>;
impl IbirespR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ibiresp {
        match self.bits {
            0 => Ibiresp::Ack,
            1 => Ibiresp::Nack,
            2 => Ibiresp::AckWithMandatory,
            3 => Ibiresp::Manual,
            _ => unreachable!(),
        }
    }
    #[doc = "ACK (acknowledge)"]
    #[inline(always)]
    pub fn is_ack(&self) -> bool {
        *self == Ibiresp::Ack
    }
    #[doc = "NACK (reject)"]
    #[inline(always)]
    pub fn is_nack(&self) -> bool {
        *self == Ibiresp::Nack
    }
    #[doc = "Acknowledge with mandatory byte"]
    #[inline(always)]
    pub fn is_ack_with_mandatory(&self) -> bool {
        *self == Ibiresp::AckWithMandatory
    }
    #[doc = "Manual"]
    #[inline(always)]
    pub fn is_manual(&self) -> bool {
        *self == Ibiresp::Manual
    }
}
#[doc = "Field `IBIRESP` writer - In-Band Interrupt Response"]
pub type IbirespW<'a, REG> = crate::FieldWriter<'a, REG, 2, Ibiresp, crate::Safe>;
impl<'a, REG> IbirespW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "ACK (acknowledge)"]
    #[inline(always)]
    pub fn ack(self) -> &'a mut crate::W<REG> {
        self.variant(Ibiresp::Ack)
    }
    #[doc = "NACK (reject)"]
    #[inline(always)]
    pub fn nack(self) -> &'a mut crate::W<REG> {
        self.variant(Ibiresp::Nack)
    }
    #[doc = "Acknowledge with mandatory byte"]
    #[inline(always)]
    pub fn ack_with_mandatory(self) -> &'a mut crate::W<REG> {
        self.variant(Ibiresp::AckWithMandatory)
    }
    #[doc = "Manual"]
    #[inline(always)]
    pub fn manual(self) -> &'a mut crate::W<REG> {
        self.variant(Ibiresp::Manual)
    }
}
#[doc = "Direction\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    #[doc = "0: Write"]
    Dirwrite = 0,
    #[doc = "1: Read"]
    Dirread = 1,
}
impl From<Dir> for bool {
    #[inline(always)]
    fn from(variant: Dir) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DIR` reader - Direction"]
pub type DirR = crate::BitReader<Dir>;
impl DirR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dir {
        match self.bits {
            false => Dir::Dirwrite,
            true => Dir::Dirread,
        }
    }
    #[doc = "Write"]
    #[inline(always)]
    pub fn is_dirwrite(&self) -> bool {
        *self == Dir::Dirwrite
    }
    #[doc = "Read"]
    #[inline(always)]
    pub fn is_dirread(&self) -> bool {
        *self == Dir::Dirread
    }
}
#[doc = "Field `DIR` writer - Direction"]
pub type DirW<'a, REG> = crate::BitWriter<'a, REG, Dir>;
impl<'a, REG> DirW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Write"]
    #[inline(always)]
    pub fn dirwrite(self) -> &'a mut crate::W<REG> {
        self.variant(Dir::Dirwrite)
    }
    #[doc = "Read"]
    #[inline(always)]
    pub fn dirread(self) -> &'a mut crate::W<REG> {
        self.variant(Dir::Dirread)
    }
}
#[doc = "Field `ADDR` reader - Address"]
pub type AddrR = crate::FieldReader;
#[doc = "Field `ADDR` writer - Address"]
pub type AddrW<'a, REG> = crate::FieldWriter<'a, REG, 7>;
#[doc = "Field `RDTERM` reader - Read Terminate Counter"]
pub type RdtermR = crate::FieldReader;
#[doc = "Field `RDTERM` writer - Read Terminate Counter"]
pub type RdtermW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:2 - Request"]
    #[inline(always)]
    pub fn request(&self) -> RequestR {
        RequestR::new((self.bits & 7) as u8)
    }
    #[doc = "Bits 4:5 - Bus Type with EmitStartAddr"]
    #[inline(always)]
    pub fn type_(&self) -> TypeR {
        TypeR::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 6:7 - In-Band Interrupt Response"]
    #[inline(always)]
    pub fn ibiresp(&self) -> IbirespR {
        IbirespR::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bit 8 - Direction"]
    #[inline(always)]
    pub fn dir(&self) -> DirR {
        DirR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bits 9:15 - Address"]
    #[inline(always)]
    pub fn addr(&self) -> AddrR {
        AddrR::new(((self.bits >> 9) & 0x7f) as u8)
    }
    #[doc = "Bits 16:23 - Read Terminate Counter"]
    #[inline(always)]
    pub fn rdterm(&self) -> RdtermR {
        RdtermR::new(((self.bits >> 16) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:2 - Request"]
    #[inline(always)]
    pub fn request(&mut self) -> RequestW<MctrlSpec> {
        RequestW::new(self, 0)
    }
    #[doc = "Bits 4:5 - Bus Type with EmitStartAddr"]
    #[inline(always)]
    pub fn type_(&mut self) -> TypeW<MctrlSpec> {
        TypeW::new(self, 4)
    }
    #[doc = "Bits 6:7 - In-Band Interrupt Response"]
    #[inline(always)]
    pub fn ibiresp(&mut self) -> IbirespW<MctrlSpec> {
        IbirespW::new(self, 6)
    }
    #[doc = "Bit 8 - Direction"]
    #[inline(always)]
    pub fn dir(&mut self) -> DirW<MctrlSpec> {
        DirW::new(self, 8)
    }
    #[doc = "Bits 9:15 - Address"]
    #[inline(always)]
    pub fn addr(&mut self) -> AddrW<MctrlSpec> {
        AddrW::new(self, 9)
    }
    #[doc = "Bits 16:23 - Read Terminate Counter"]
    #[inline(always)]
    pub fn rdterm(&mut self) -> RdtermW<MctrlSpec> {
        RdtermW::new(self, 16)
    }
}
#[doc = "Controller Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MctrlSpec;
impl crate::RegisterSpec for MctrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mctrl::R`](R) reader structure"]
impl crate::Readable for MctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`mctrl::W`](W) writer structure"]
impl crate::Writable for MctrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MCTRL to value 0"]
impl crate::Resettable for MctrlSpec {}
