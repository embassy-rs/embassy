#[doc = "Register `VID1` reader"]
pub type R = crate::R<Vid1Spec>;
#[doc = "Shows the IP's Minor revision of the TRNG.\n\nValue on reset: 12"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MinRev {
    #[doc = "12: Minor revision number for TRNG."]
    MinRevVal = 12,
}
impl From<MinRev> for u8 {
    #[inline(always)]
    fn from(variant: MinRev) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for MinRev {
    type Ux = u8;
}
impl crate::IsEnum for MinRev {}
#[doc = "Field `MIN_REV` reader - Shows the IP's Minor revision of the TRNG."]
pub type MinRevR = crate::FieldReader<MinRev>;
impl MinRevR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<MinRev> {
        match self.bits {
            12 => Some(MinRev::MinRevVal),
            _ => None,
        }
    }
    #[doc = "Minor revision number for TRNG."]
    #[inline(always)]
    pub fn is_min_rev_val(&self) -> bool {
        *self == MinRev::MinRevVal
    }
}
#[doc = "Shows the IP's Major revision of the TRNG\n\nValue on reset: 20"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MajRev {
    #[doc = "20: Major revision number for TRNG."]
    MajRevVal = 20,
}
impl From<MajRev> for u8 {
    #[inline(always)]
    fn from(variant: MajRev) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for MajRev {
    type Ux = u8;
}
impl crate::IsEnum for MajRev {}
#[doc = "Field `MAJ_REV` reader - Shows the IP's Major revision of the TRNG"]
pub type MajRevR = crate::FieldReader<MajRev>;
impl MajRevR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<MajRev> {
        match self.bits {
            20 => Some(MajRev::MajRevVal),
            _ => None,
        }
    }
    #[doc = "Major revision number for TRNG."]
    #[inline(always)]
    pub fn is_maj_rev_val(&self) -> bool {
        *self == MajRev::MajRevVal
    }
}
#[doc = "Shows the IP ID.\n\nValue on reset: 48"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum IpId {
    #[doc = "48: ID for TRNG."]
    IpIdVal = 48,
}
impl From<IpId> for u16 {
    #[inline(always)]
    fn from(variant: IpId) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for IpId {
    type Ux = u16;
}
impl crate::IsEnum for IpId {}
#[doc = "Field `IP_ID` reader - Shows the IP ID."]
pub type IpIdR = crate::FieldReader<IpId>;
impl IpIdR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<IpId> {
        match self.bits {
            48 => Some(IpId::IpIdVal),
            _ => None,
        }
    }
    #[doc = "ID for TRNG."]
    #[inline(always)]
    pub fn is_ip_id_val(&self) -> bool {
        *self == IpId::IpIdVal
    }
}
impl R {
    #[doc = "Bits 0:7 - Shows the IP's Minor revision of the TRNG."]
    #[inline(always)]
    pub fn min_rev(&self) -> MinRevR {
        MinRevR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Shows the IP's Major revision of the TRNG"]
    #[inline(always)]
    pub fn maj_rev(&self) -> MajRevR {
        MajRevR::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:31 - Shows the IP ID."]
    #[inline(always)]
    pub fn ip_id(&self) -> IpIdR {
        IpIdR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
#[doc = "Version ID Register (MS)\n\nYou can [`read`](crate::Reg::read) this register and get [`vid1::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Vid1Spec;
impl crate::RegisterSpec for Vid1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`vid1::R`](R) reader structure"]
impl crate::Readable for Vid1Spec {}
#[doc = "`reset()` method sets VID1 to value 0x0030_140c"]
impl crate::Resettable for Vid1Spec {
    const RESET_VALUE: u32 = 0x0030_140c;
}
