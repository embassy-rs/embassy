/*
Generated with the following snippet.
Switch to a proper script when https://github.com/YuhanLiin/micropb/issues/30 is done

    let mut g = micropb_gen::Generator::new();
    g.use_container_heapless();

    g.configure(
        ".",
        micropb_gen::Config::new()
            .max_bytes(32) // For ssid, mac, etc - strings
            .max_len(16) // For repeated fields
            .type_attributes("#[cfg_attr(feature = \"defmt\", derive(defmt::Format))]"),
    );

    // Special config for things that need to be larger
    g.configure(
        ".CtrlMsg_Req_OTAWrite.ota_data",
        micropb_gen::Config::new().max_bytes(256),
    );
    g.configure(
        ".CtrlMsg_Event_ESPInit.init_data",
        micropb_gen::Config::new().max_bytes(64),
    );
    g.configure(
        ".CtrlMsg_Req_VendorIEData.payload",
        micropb_gen::Config::new().max_bytes(64),
    );

    g.compile_protos(&["src/esp_hosted_config.proto"], format!("{}/proto.rs", out_dir))
        .unwrap();

*/

#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ScanResult {
    pub r#ssid: ::micropb::heapless::Vec<u8, 32>,
    pub r#chnl: u32,
    pub r#rssi: i32,
    pub r#bssid: ::micropb::heapless::Vec<u8, 32>,
    pub r#sec_prot: Ctrl_WifiSecProt,
}
impl ScanResult {
    ///Return a reference to `ssid`
    #[inline]
    pub fn r#ssid(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#ssid
    }
    ///Return a mutable reference to `ssid`
    #[inline]
    pub fn mut_ssid(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#ssid
    }
    ///Set the value of `ssid`
    #[inline]
    pub fn set_ssid(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#ssid = value.into();
        self
    }
    ///Builder method that sets the value of `ssid`. Useful for initializing the message.
    #[inline]
    pub fn init_ssid(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#ssid = value.into();
        self
    }
    ///Return a reference to `chnl`
    #[inline]
    pub fn r#chnl(&self) -> &u32 {
        &self.r#chnl
    }
    ///Return a mutable reference to `chnl`
    #[inline]
    pub fn mut_chnl(&mut self) -> &mut u32 {
        &mut self.r#chnl
    }
    ///Set the value of `chnl`
    #[inline]
    pub fn set_chnl(&mut self, value: u32) -> &mut Self {
        self.r#chnl = value.into();
        self
    }
    ///Builder method that sets the value of `chnl`. Useful for initializing the message.
    #[inline]
    pub fn init_chnl(mut self, value: u32) -> Self {
        self.r#chnl = value.into();
        self
    }
    ///Return a reference to `rssi`
    #[inline]
    pub fn r#rssi(&self) -> &i32 {
        &self.r#rssi
    }
    ///Return a mutable reference to `rssi`
    #[inline]
    pub fn mut_rssi(&mut self) -> &mut i32 {
        &mut self.r#rssi
    }
    ///Set the value of `rssi`
    #[inline]
    pub fn set_rssi(&mut self, value: i32) -> &mut Self {
        self.r#rssi = value.into();
        self
    }
    ///Builder method that sets the value of `rssi`. Useful for initializing the message.
    #[inline]
    pub fn init_rssi(mut self, value: i32) -> Self {
        self.r#rssi = value.into();
        self
    }
    ///Return a reference to `bssid`
    #[inline]
    pub fn r#bssid(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#bssid
    }
    ///Return a mutable reference to `bssid`
    #[inline]
    pub fn mut_bssid(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#bssid
    }
    ///Set the value of `bssid`
    #[inline]
    pub fn set_bssid(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#bssid = value.into();
        self
    }
    ///Builder method that sets the value of `bssid`. Useful for initializing the message.
    #[inline]
    pub fn init_bssid(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#bssid = value.into();
        self
    }
    ///Return a reference to `sec_prot`
    #[inline]
    pub fn r#sec_prot(&self) -> &Ctrl_WifiSecProt {
        &self.r#sec_prot
    }
    ///Return a mutable reference to `sec_prot`
    #[inline]
    pub fn mut_sec_prot(&mut self) -> &mut Ctrl_WifiSecProt {
        &mut self.r#sec_prot
    }
    ///Set the value of `sec_prot`
    #[inline]
    pub fn set_sec_prot(&mut self, value: Ctrl_WifiSecProt) -> &mut Self {
        self.r#sec_prot = value.into();
        self
    }
    ///Builder method that sets the value of `sec_prot`. Useful for initializing the message.
    #[inline]
    pub fn init_sec_prot(mut self, value: Ctrl_WifiSecProt) -> Self {
        self.r#sec_prot = value.into();
        self
    }
}
impl ::micropb::MessageDecode for ScanResult {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#ssid;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#chnl;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#rssi;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                4u32 => {
                    let mut_ref = &mut self.r#bssid;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                5u32 => {
                    let mut_ref = &mut self.r#sec_prot;
                    {
                        let val = decoder.decode_int32().map(|n| Ctrl_WifiSecProt(n as _))?;
                        let val_ref = &val;
                        if val_ref.0 != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for ScanResult {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(Ctrl_WifiSecProt::_MAX_SIZE), |size| size
                + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#ssid;
            if !val_ref.is_empty() {
                encoder.encode_varint32(10u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#chnl;
            if *val_ref != 0 {
                encoder.encode_varint32(16u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#rssi;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#bssid;
            if !val_ref.is_empty() {
                encoder.encode_varint32(34u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#sec_prot;
            if val_ref.0 != 0 {
                encoder.encode_varint32(40u32)?;
                encoder.encode_int32(val_ref.0 as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#ssid;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#chnl;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#rssi;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#bssid;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#sec_prot;
            if val_ref.0 != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(val_ref.0 as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConnectedSTAList {
    pub r#mac: ::micropb::heapless::Vec<u8, 32>,
    pub r#rssi: i32,
}
impl ConnectedSTAList {
    ///Return a reference to `mac`
    #[inline]
    pub fn r#mac(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#mac
    }
    ///Return a mutable reference to `mac`
    #[inline]
    pub fn mut_mac(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#mac
    }
    ///Set the value of `mac`
    #[inline]
    pub fn set_mac(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#mac = value.into();
        self
    }
    ///Builder method that sets the value of `mac`. Useful for initializing the message.
    #[inline]
    pub fn init_mac(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#mac = value.into();
        self
    }
    ///Return a reference to `rssi`
    #[inline]
    pub fn r#rssi(&self) -> &i32 {
        &self.r#rssi
    }
    ///Return a mutable reference to `rssi`
    #[inline]
    pub fn mut_rssi(&mut self) -> &mut i32 {
        &mut self.r#rssi
    }
    ///Set the value of `rssi`
    #[inline]
    pub fn set_rssi(&mut self, value: i32) -> &mut Self {
        self.r#rssi = value.into();
        self
    }
    ///Builder method that sets the value of `rssi`. Useful for initializing the message.
    #[inline]
    pub fn init_rssi(mut self, value: i32) -> Self {
        self.r#rssi = value.into();
        self
    }
}
impl ::micropb::MessageDecode for ConnectedSTAList {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#mac;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#rssi;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for ConnectedSTAList {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#mac;
            if !val_ref.is_empty() {
                encoder.encode_varint32(10u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#rssi;
            if *val_ref != 0 {
                encoder.encode_varint32(16u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#mac;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#rssi;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_GetMacAddress {
    pub r#mode: i32,
}
impl CtrlMsg_Req_GetMacAddress {
    ///Return a reference to `mode`
    #[inline]
    pub fn r#mode(&self) -> &i32 {
        &self.r#mode
    }
    ///Return a mutable reference to `mode`
    #[inline]
    pub fn mut_mode(&mut self) -> &mut i32 {
        &mut self.r#mode
    }
    ///Set the value of `mode`
    #[inline]
    pub fn set_mode(&mut self, value: i32) -> &mut Self {
        self.r#mode = value.into();
        self
    }
    ///Builder method that sets the value of `mode`. Useful for initializing the message.
    #[inline]
    pub fn init_mode(mut self, value: i32) -> Self {
        self.r#mode = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Req_GetMacAddress {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#mode;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_GetMacAddress {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#mode;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#mode;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_GetMacAddress {
    pub r#mac: ::micropb::heapless::Vec<u8, 32>,
    pub r#resp: i32,
}
impl CtrlMsg_Resp_GetMacAddress {
    ///Return a reference to `mac`
    #[inline]
    pub fn r#mac(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#mac
    }
    ///Return a mutable reference to `mac`
    #[inline]
    pub fn mut_mac(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#mac
    }
    ///Set the value of `mac`
    #[inline]
    pub fn set_mac(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#mac = value.into();
        self
    }
    ///Builder method that sets the value of `mac`. Useful for initializing the message.
    #[inline]
    pub fn init_mac(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#mac = value.into();
        self
    }
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_GetMacAddress {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#mac;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_GetMacAddress {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#mac;
            if !val_ref.is_empty() {
                encoder.encode_varint32(10u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(16u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#mac;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_GetMode {}
impl CtrlMsg_Req_GetMode {}
impl ::micropb::MessageDecode for CtrlMsg_Req_GetMode {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_GetMode {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_GetMode {
    pub r#mode: i32,
    pub r#resp: i32,
}
impl CtrlMsg_Resp_GetMode {
    ///Return a reference to `mode`
    #[inline]
    pub fn r#mode(&self) -> &i32 {
        &self.r#mode
    }
    ///Return a mutable reference to `mode`
    #[inline]
    pub fn mut_mode(&mut self) -> &mut i32 {
        &mut self.r#mode
    }
    ///Set the value of `mode`
    #[inline]
    pub fn set_mode(&mut self, value: i32) -> &mut Self {
        self.r#mode = value.into();
        self
    }
    ///Builder method that sets the value of `mode`. Useful for initializing the message.
    #[inline]
    pub fn init_mode(mut self, value: i32) -> Self {
        self.r#mode = value.into();
        self
    }
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_GetMode {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#mode;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_GetMode {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#mode;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(16u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#mode;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_SetMode {
    pub r#mode: i32,
}
impl CtrlMsg_Req_SetMode {
    ///Return a reference to `mode`
    #[inline]
    pub fn r#mode(&self) -> &i32 {
        &self.r#mode
    }
    ///Return a mutable reference to `mode`
    #[inline]
    pub fn mut_mode(&mut self) -> &mut i32 {
        &mut self.r#mode
    }
    ///Set the value of `mode`
    #[inline]
    pub fn set_mode(&mut self, value: i32) -> &mut Self {
        self.r#mode = value.into();
        self
    }
    ///Builder method that sets the value of `mode`. Useful for initializing the message.
    #[inline]
    pub fn init_mode(mut self, value: i32) -> Self {
        self.r#mode = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Req_SetMode {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#mode;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_SetMode {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#mode;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#mode;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_SetMode {
    pub r#resp: i32,
}
impl CtrlMsg_Resp_SetMode {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_SetMode {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_SetMode {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_GetStatus {}
impl CtrlMsg_Req_GetStatus {}
impl ::micropb::MessageDecode for CtrlMsg_Req_GetStatus {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_GetStatus {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_GetStatus {
    pub r#resp: i32,
}
impl CtrlMsg_Resp_GetStatus {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_GetStatus {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_GetStatus {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_SetMacAddress {
    pub r#mac: ::micropb::heapless::Vec<u8, 32>,
    pub r#mode: i32,
}
impl CtrlMsg_Req_SetMacAddress {
    ///Return a reference to `mac`
    #[inline]
    pub fn r#mac(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#mac
    }
    ///Return a mutable reference to `mac`
    #[inline]
    pub fn mut_mac(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#mac
    }
    ///Set the value of `mac`
    #[inline]
    pub fn set_mac(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#mac = value.into();
        self
    }
    ///Builder method that sets the value of `mac`. Useful for initializing the message.
    #[inline]
    pub fn init_mac(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#mac = value.into();
        self
    }
    ///Return a reference to `mode`
    #[inline]
    pub fn r#mode(&self) -> &i32 {
        &self.r#mode
    }
    ///Return a mutable reference to `mode`
    #[inline]
    pub fn mut_mode(&mut self) -> &mut i32 {
        &mut self.r#mode
    }
    ///Set the value of `mode`
    #[inline]
    pub fn set_mode(&mut self, value: i32) -> &mut Self {
        self.r#mode = value.into();
        self
    }
    ///Builder method that sets the value of `mode`. Useful for initializing the message.
    #[inline]
    pub fn init_mode(mut self, value: i32) -> Self {
        self.r#mode = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Req_SetMacAddress {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#mac;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#mode;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_SetMacAddress {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#mac;
            if !val_ref.is_empty() {
                encoder.encode_varint32(10u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#mode;
            if *val_ref != 0 {
                encoder.encode_varint32(16u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#mac;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#mode;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_SetMacAddress {
    pub r#resp: i32,
}
impl CtrlMsg_Resp_SetMacAddress {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_SetMacAddress {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_SetMacAddress {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_GetAPConfig {}
impl CtrlMsg_Req_GetAPConfig {}
impl ::micropb::MessageDecode for CtrlMsg_Req_GetAPConfig {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_GetAPConfig {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_GetAPConfig {
    pub r#ssid: ::micropb::heapless::Vec<u8, 32>,
    pub r#bssid: ::micropb::heapless::Vec<u8, 32>,
    pub r#rssi: i32,
    pub r#chnl: i32,
    pub r#sec_prot: Ctrl_WifiSecProt,
    pub r#resp: i32,
    pub r#band_mode: i32,
}
impl CtrlMsg_Resp_GetAPConfig {
    ///Return a reference to `ssid`
    #[inline]
    pub fn r#ssid(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#ssid
    }
    ///Return a mutable reference to `ssid`
    #[inline]
    pub fn mut_ssid(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#ssid
    }
    ///Set the value of `ssid`
    #[inline]
    pub fn set_ssid(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#ssid = value.into();
        self
    }
    ///Builder method that sets the value of `ssid`. Useful for initializing the message.
    #[inline]
    pub fn init_ssid(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#ssid = value.into();
        self
    }
    ///Return a reference to `bssid`
    #[inline]
    pub fn r#bssid(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#bssid
    }
    ///Return a mutable reference to `bssid`
    #[inline]
    pub fn mut_bssid(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#bssid
    }
    ///Set the value of `bssid`
    #[inline]
    pub fn set_bssid(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#bssid = value.into();
        self
    }
    ///Builder method that sets the value of `bssid`. Useful for initializing the message.
    #[inline]
    pub fn init_bssid(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#bssid = value.into();
        self
    }
    ///Return a reference to `rssi`
    #[inline]
    pub fn r#rssi(&self) -> &i32 {
        &self.r#rssi
    }
    ///Return a mutable reference to `rssi`
    #[inline]
    pub fn mut_rssi(&mut self) -> &mut i32 {
        &mut self.r#rssi
    }
    ///Set the value of `rssi`
    #[inline]
    pub fn set_rssi(&mut self, value: i32) -> &mut Self {
        self.r#rssi = value.into();
        self
    }
    ///Builder method that sets the value of `rssi`. Useful for initializing the message.
    #[inline]
    pub fn init_rssi(mut self, value: i32) -> Self {
        self.r#rssi = value.into();
        self
    }
    ///Return a reference to `chnl`
    #[inline]
    pub fn r#chnl(&self) -> &i32 {
        &self.r#chnl
    }
    ///Return a mutable reference to `chnl`
    #[inline]
    pub fn mut_chnl(&mut self) -> &mut i32 {
        &mut self.r#chnl
    }
    ///Set the value of `chnl`
    #[inline]
    pub fn set_chnl(&mut self, value: i32) -> &mut Self {
        self.r#chnl = value.into();
        self
    }
    ///Builder method that sets the value of `chnl`. Useful for initializing the message.
    #[inline]
    pub fn init_chnl(mut self, value: i32) -> Self {
        self.r#chnl = value.into();
        self
    }
    ///Return a reference to `sec_prot`
    #[inline]
    pub fn r#sec_prot(&self) -> &Ctrl_WifiSecProt {
        &self.r#sec_prot
    }
    ///Return a mutable reference to `sec_prot`
    #[inline]
    pub fn mut_sec_prot(&mut self) -> &mut Ctrl_WifiSecProt {
        &mut self.r#sec_prot
    }
    ///Set the value of `sec_prot`
    #[inline]
    pub fn set_sec_prot(&mut self, value: Ctrl_WifiSecProt) -> &mut Self {
        self.r#sec_prot = value.into();
        self
    }
    ///Builder method that sets the value of `sec_prot`. Useful for initializing the message.
    #[inline]
    pub fn init_sec_prot(mut self, value: Ctrl_WifiSecProt) -> Self {
        self.r#sec_prot = value.into();
        self
    }
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
    ///Return a reference to `band_mode`
    #[inline]
    pub fn r#band_mode(&self) -> &i32 {
        &self.r#band_mode
    }
    ///Return a mutable reference to `band_mode`
    #[inline]
    pub fn mut_band_mode(&mut self) -> &mut i32 {
        &mut self.r#band_mode
    }
    ///Set the value of `band_mode`
    #[inline]
    pub fn set_band_mode(&mut self, value: i32) -> &mut Self {
        self.r#band_mode = value.into();
        self
    }
    ///Builder method that sets the value of `band_mode`. Useful for initializing the message.
    #[inline]
    pub fn init_band_mode(mut self, value: i32) -> Self {
        self.r#band_mode = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_GetAPConfig {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#ssid;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#bssid;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#rssi;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                4u32 => {
                    let mut_ref = &mut self.r#chnl;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                5u32 => {
                    let mut_ref = &mut self.r#sec_prot;
                    {
                        let val = decoder.decode_int32().map(|n| Ctrl_WifiSecProt(n as _))?;
                        let val_ref = &val;
                        if val_ref.0 != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                6u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                7u32 => {
                    let mut_ref = &mut self.r#band_mode;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_GetAPConfig {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(Ctrl_WifiSecProt::_MAX_SIZE), |size| size
                + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#ssid;
            if !val_ref.is_empty() {
                encoder.encode_varint32(10u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#bssid;
            if !val_ref.is_empty() {
                encoder.encode_varint32(18u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#rssi;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#chnl;
            if *val_ref != 0 {
                encoder.encode_varint32(32u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#sec_prot;
            if val_ref.0 != 0 {
                encoder.encode_varint32(40u32)?;
                encoder.encode_int32(val_ref.0 as _)?;
            }
        }
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(48u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#band_mode;
            if *val_ref != 0 {
                encoder.encode_varint32(56u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#ssid;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#bssid;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#rssi;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#chnl;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#sec_prot;
            if val_ref.0 != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(val_ref.0 as _);
            }
        }
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#band_mode;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_ConnectAP {
    pub r#ssid: ::micropb::heapless::String<32>,
    pub r#pwd: ::micropb::heapless::String<32>,
    pub r#bssid: ::micropb::heapless::String<32>,
    pub r#is_wpa3_supported: bool,
    pub r#listen_interval: i32,
    pub r#band_mode: i32,
}
impl CtrlMsg_Req_ConnectAP {
    ///Return a reference to `ssid`
    #[inline]
    pub fn r#ssid(&self) -> &::micropb::heapless::String<32> {
        &self.r#ssid
    }
    ///Return a mutable reference to `ssid`
    #[inline]
    pub fn mut_ssid(&mut self) -> &mut ::micropb::heapless::String<32> {
        &mut self.r#ssid
    }
    ///Set the value of `ssid`
    #[inline]
    pub fn set_ssid(&mut self, value: ::micropb::heapless::String<32>) -> &mut Self {
        self.r#ssid = value.into();
        self
    }
    ///Builder method that sets the value of `ssid`. Useful for initializing the message.
    #[inline]
    pub fn init_ssid(mut self, value: ::micropb::heapless::String<32>) -> Self {
        self.r#ssid = value.into();
        self
    }
    ///Return a reference to `pwd`
    #[inline]
    pub fn r#pwd(&self) -> &::micropb::heapless::String<32> {
        &self.r#pwd
    }
    ///Return a mutable reference to `pwd`
    #[inline]
    pub fn mut_pwd(&mut self) -> &mut ::micropb::heapless::String<32> {
        &mut self.r#pwd
    }
    ///Set the value of `pwd`
    #[inline]
    pub fn set_pwd(&mut self, value: ::micropb::heapless::String<32>) -> &mut Self {
        self.r#pwd = value.into();
        self
    }
    ///Builder method that sets the value of `pwd`. Useful for initializing the message.
    #[inline]
    pub fn init_pwd(mut self, value: ::micropb::heapless::String<32>) -> Self {
        self.r#pwd = value.into();
        self
    }
    ///Return a reference to `bssid`
    #[inline]
    pub fn r#bssid(&self) -> &::micropb::heapless::String<32> {
        &self.r#bssid
    }
    ///Return a mutable reference to `bssid`
    #[inline]
    pub fn mut_bssid(&mut self) -> &mut ::micropb::heapless::String<32> {
        &mut self.r#bssid
    }
    ///Set the value of `bssid`
    #[inline]
    pub fn set_bssid(&mut self, value: ::micropb::heapless::String<32>) -> &mut Self {
        self.r#bssid = value.into();
        self
    }
    ///Builder method that sets the value of `bssid`. Useful for initializing the message.
    #[inline]
    pub fn init_bssid(mut self, value: ::micropb::heapless::String<32>) -> Self {
        self.r#bssid = value.into();
        self
    }
    ///Return a reference to `is_wpa3_supported`
    #[inline]
    pub fn r#is_wpa3_supported(&self) -> &bool {
        &self.r#is_wpa3_supported
    }
    ///Return a mutable reference to `is_wpa3_supported`
    #[inline]
    pub fn mut_is_wpa3_supported(&mut self) -> &mut bool {
        &mut self.r#is_wpa3_supported
    }
    ///Set the value of `is_wpa3_supported`
    #[inline]
    pub fn set_is_wpa3_supported(&mut self, value: bool) -> &mut Self {
        self.r#is_wpa3_supported = value.into();
        self
    }
    ///Builder method that sets the value of `is_wpa3_supported`. Useful for initializing the message.
    #[inline]
    pub fn init_is_wpa3_supported(mut self, value: bool) -> Self {
        self.r#is_wpa3_supported = value.into();
        self
    }
    ///Return a reference to `listen_interval`
    #[inline]
    pub fn r#listen_interval(&self) -> &i32 {
        &self.r#listen_interval
    }
    ///Return a mutable reference to `listen_interval`
    #[inline]
    pub fn mut_listen_interval(&mut self) -> &mut i32 {
        &mut self.r#listen_interval
    }
    ///Set the value of `listen_interval`
    #[inline]
    pub fn set_listen_interval(&mut self, value: i32) -> &mut Self {
        self.r#listen_interval = value.into();
        self
    }
    ///Builder method that sets the value of `listen_interval`. Useful for initializing the message.
    #[inline]
    pub fn init_listen_interval(mut self, value: i32) -> Self {
        self.r#listen_interval = value.into();
        self
    }
    ///Return a reference to `band_mode`
    #[inline]
    pub fn r#band_mode(&self) -> &i32 {
        &self.r#band_mode
    }
    ///Return a mutable reference to `band_mode`
    #[inline]
    pub fn mut_band_mode(&mut self) -> &mut i32 {
        &mut self.r#band_mode
    }
    ///Set the value of `band_mode`
    #[inline]
    pub fn set_band_mode(&mut self, value: i32) -> &mut Self {
        self.r#band_mode = value.into();
        self
    }
    ///Builder method that sets the value of `band_mode`. Useful for initializing the message.
    #[inline]
    pub fn init_band_mode(mut self, value: i32) -> Self {
        self.r#band_mode = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Req_ConnectAP {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#ssid;
                    {
                        decoder.decode_string(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#pwd;
                    {
                        decoder.decode_string(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#bssid;
                    {
                        decoder.decode_string(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                4u32 => {
                    let mut_ref = &mut self.r#is_wpa3_supported;
                    {
                        let val = decoder.decode_bool()?;
                        let val_ref = &val;
                        if *val_ref {
                            *mut_ref = val as _;
                        }
                    };
                }
                5u32 => {
                    let mut_ref = &mut self.r#listen_interval;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                6u32 => {
                    let mut_ref = &mut self.r#band_mode;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_ConnectAP {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(1usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#ssid;
            if !val_ref.is_empty() {
                encoder.encode_varint32(10u32)?;
                encoder.encode_string(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#pwd;
            if !val_ref.is_empty() {
                encoder.encode_varint32(18u32)?;
                encoder.encode_string(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#bssid;
            if !val_ref.is_empty() {
                encoder.encode_varint32(26u32)?;
                encoder.encode_string(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#is_wpa3_supported;
            if *val_ref {
                encoder.encode_varint32(32u32)?;
                encoder.encode_bool(*val_ref)?;
            }
        }
        {
            let val_ref = &self.r#listen_interval;
            if *val_ref != 0 {
                encoder.encode_varint32(40u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#band_mode;
            if *val_ref != 0 {
                encoder.encode_varint32(48u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#ssid;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#pwd;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#bssid;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#is_wpa3_supported;
            if *val_ref {
                size += 1usize + 1;
            }
        }
        {
            let val_ref = &self.r#listen_interval;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#band_mode;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_ConnectAP {
    pub r#resp: i32,
    pub r#mac: ::micropb::heapless::Vec<u8, 32>,
    pub r#band_mode: i32,
}
impl CtrlMsg_Resp_ConnectAP {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
    ///Return a reference to `mac`
    #[inline]
    pub fn r#mac(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#mac
    }
    ///Return a mutable reference to `mac`
    #[inline]
    pub fn mut_mac(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#mac
    }
    ///Set the value of `mac`
    #[inline]
    pub fn set_mac(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#mac = value.into();
        self
    }
    ///Builder method that sets the value of `mac`. Useful for initializing the message.
    #[inline]
    pub fn init_mac(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#mac = value.into();
        self
    }
    ///Return a reference to `band_mode`
    #[inline]
    pub fn r#band_mode(&self) -> &i32 {
        &self.r#band_mode
    }
    ///Return a mutable reference to `band_mode`
    #[inline]
    pub fn mut_band_mode(&mut self) -> &mut i32 {
        &mut self.r#band_mode
    }
    ///Set the value of `band_mode`
    #[inline]
    pub fn set_band_mode(&mut self, value: i32) -> &mut Self {
        self.r#band_mode = value.into();
        self
    }
    ///Builder method that sets the value of `band_mode`. Useful for initializing the message.
    #[inline]
    pub fn init_band_mode(mut self, value: i32) -> Self {
        self.r#band_mode = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_ConnectAP {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#mac;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#band_mode;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_ConnectAP {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#mac;
            if !val_ref.is_empty() {
                encoder.encode_varint32(18u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#band_mode;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#mac;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#band_mode;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_GetSoftAPConfig {}
impl CtrlMsg_Req_GetSoftAPConfig {}
impl ::micropb::MessageDecode for CtrlMsg_Req_GetSoftAPConfig {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_GetSoftAPConfig {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_GetSoftAPConfig {
    pub r#ssid: ::micropb::heapless::Vec<u8, 32>,
    pub r#pwd: ::micropb::heapless::Vec<u8, 32>,
    pub r#chnl: i32,
    pub r#sec_prot: Ctrl_WifiSecProt,
    pub r#max_conn: i32,
    pub r#ssid_hidden: bool,
    pub r#bw: i32,
    pub r#resp: i32,
    pub r#band_mode: i32,
}
impl CtrlMsg_Resp_GetSoftAPConfig {
    ///Return a reference to `ssid`
    #[inline]
    pub fn r#ssid(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#ssid
    }
    ///Return a mutable reference to `ssid`
    #[inline]
    pub fn mut_ssid(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#ssid
    }
    ///Set the value of `ssid`
    #[inline]
    pub fn set_ssid(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#ssid = value.into();
        self
    }
    ///Builder method that sets the value of `ssid`. Useful for initializing the message.
    #[inline]
    pub fn init_ssid(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#ssid = value.into();
        self
    }
    ///Return a reference to `pwd`
    #[inline]
    pub fn r#pwd(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#pwd
    }
    ///Return a mutable reference to `pwd`
    #[inline]
    pub fn mut_pwd(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#pwd
    }
    ///Set the value of `pwd`
    #[inline]
    pub fn set_pwd(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#pwd = value.into();
        self
    }
    ///Builder method that sets the value of `pwd`. Useful for initializing the message.
    #[inline]
    pub fn init_pwd(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#pwd = value.into();
        self
    }
    ///Return a reference to `chnl`
    #[inline]
    pub fn r#chnl(&self) -> &i32 {
        &self.r#chnl
    }
    ///Return a mutable reference to `chnl`
    #[inline]
    pub fn mut_chnl(&mut self) -> &mut i32 {
        &mut self.r#chnl
    }
    ///Set the value of `chnl`
    #[inline]
    pub fn set_chnl(&mut self, value: i32) -> &mut Self {
        self.r#chnl = value.into();
        self
    }
    ///Builder method that sets the value of `chnl`. Useful for initializing the message.
    #[inline]
    pub fn init_chnl(mut self, value: i32) -> Self {
        self.r#chnl = value.into();
        self
    }
    ///Return a reference to `sec_prot`
    #[inline]
    pub fn r#sec_prot(&self) -> &Ctrl_WifiSecProt {
        &self.r#sec_prot
    }
    ///Return a mutable reference to `sec_prot`
    #[inline]
    pub fn mut_sec_prot(&mut self) -> &mut Ctrl_WifiSecProt {
        &mut self.r#sec_prot
    }
    ///Set the value of `sec_prot`
    #[inline]
    pub fn set_sec_prot(&mut self, value: Ctrl_WifiSecProt) -> &mut Self {
        self.r#sec_prot = value.into();
        self
    }
    ///Builder method that sets the value of `sec_prot`. Useful for initializing the message.
    #[inline]
    pub fn init_sec_prot(mut self, value: Ctrl_WifiSecProt) -> Self {
        self.r#sec_prot = value.into();
        self
    }
    ///Return a reference to `max_conn`
    #[inline]
    pub fn r#max_conn(&self) -> &i32 {
        &self.r#max_conn
    }
    ///Return a mutable reference to `max_conn`
    #[inline]
    pub fn mut_max_conn(&mut self) -> &mut i32 {
        &mut self.r#max_conn
    }
    ///Set the value of `max_conn`
    #[inline]
    pub fn set_max_conn(&mut self, value: i32) -> &mut Self {
        self.r#max_conn = value.into();
        self
    }
    ///Builder method that sets the value of `max_conn`. Useful for initializing the message.
    #[inline]
    pub fn init_max_conn(mut self, value: i32) -> Self {
        self.r#max_conn = value.into();
        self
    }
    ///Return a reference to `ssid_hidden`
    #[inline]
    pub fn r#ssid_hidden(&self) -> &bool {
        &self.r#ssid_hidden
    }
    ///Return a mutable reference to `ssid_hidden`
    #[inline]
    pub fn mut_ssid_hidden(&mut self) -> &mut bool {
        &mut self.r#ssid_hidden
    }
    ///Set the value of `ssid_hidden`
    #[inline]
    pub fn set_ssid_hidden(&mut self, value: bool) -> &mut Self {
        self.r#ssid_hidden = value.into();
        self
    }
    ///Builder method that sets the value of `ssid_hidden`. Useful for initializing the message.
    #[inline]
    pub fn init_ssid_hidden(mut self, value: bool) -> Self {
        self.r#ssid_hidden = value.into();
        self
    }
    ///Return a reference to `bw`
    #[inline]
    pub fn r#bw(&self) -> &i32 {
        &self.r#bw
    }
    ///Return a mutable reference to `bw`
    #[inline]
    pub fn mut_bw(&mut self) -> &mut i32 {
        &mut self.r#bw
    }
    ///Set the value of `bw`
    #[inline]
    pub fn set_bw(&mut self, value: i32) -> &mut Self {
        self.r#bw = value.into();
        self
    }
    ///Builder method that sets the value of `bw`. Useful for initializing the message.
    #[inline]
    pub fn init_bw(mut self, value: i32) -> Self {
        self.r#bw = value.into();
        self
    }
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
    ///Return a reference to `band_mode`
    #[inline]
    pub fn r#band_mode(&self) -> &i32 {
        &self.r#band_mode
    }
    ///Return a mutable reference to `band_mode`
    #[inline]
    pub fn mut_band_mode(&mut self) -> &mut i32 {
        &mut self.r#band_mode
    }
    ///Set the value of `band_mode`
    #[inline]
    pub fn set_band_mode(&mut self, value: i32) -> &mut Self {
        self.r#band_mode = value.into();
        self
    }
    ///Builder method that sets the value of `band_mode`. Useful for initializing the message.
    #[inline]
    pub fn init_band_mode(mut self, value: i32) -> Self {
        self.r#band_mode = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_GetSoftAPConfig {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#ssid;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#pwd;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#chnl;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                4u32 => {
                    let mut_ref = &mut self.r#sec_prot;
                    {
                        let val = decoder.decode_int32().map(|n| Ctrl_WifiSecProt(n as _))?;
                        let val_ref = &val;
                        if val_ref.0 != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                5u32 => {
                    let mut_ref = &mut self.r#max_conn;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                6u32 => {
                    let mut_ref = &mut self.r#ssid_hidden;
                    {
                        let val = decoder.decode_bool()?;
                        let val_ref = &val;
                        if *val_ref {
                            *mut_ref = val as _;
                        }
                    };
                }
                7u32 => {
                    let mut_ref = &mut self.r#bw;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                8u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                9u32 => {
                    let mut_ref = &mut self.r#band_mode;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_GetSoftAPConfig {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(Ctrl_WifiSecProt::_MAX_SIZE), |size| size
                + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(1usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#ssid;
            if !val_ref.is_empty() {
                encoder.encode_varint32(10u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#pwd;
            if !val_ref.is_empty() {
                encoder.encode_varint32(18u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#chnl;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#sec_prot;
            if val_ref.0 != 0 {
                encoder.encode_varint32(32u32)?;
                encoder.encode_int32(val_ref.0 as _)?;
            }
        }
        {
            let val_ref = &self.r#max_conn;
            if *val_ref != 0 {
                encoder.encode_varint32(40u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#ssid_hidden;
            if *val_ref {
                encoder.encode_varint32(48u32)?;
                encoder.encode_bool(*val_ref)?;
            }
        }
        {
            let val_ref = &self.r#bw;
            if *val_ref != 0 {
                encoder.encode_varint32(56u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(64u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#band_mode;
            if *val_ref != 0 {
                encoder.encode_varint32(72u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#ssid;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#pwd;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#chnl;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#sec_prot;
            if val_ref.0 != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(val_ref.0 as _);
            }
        }
        {
            let val_ref = &self.r#max_conn;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#ssid_hidden;
            if *val_ref {
                size += 1usize + 1;
            }
        }
        {
            let val_ref = &self.r#bw;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#band_mode;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_StartSoftAP {
    pub r#ssid: ::micropb::heapless::String<32>,
    pub r#pwd: ::micropb::heapless::String<32>,
    pub r#chnl: i32,
    pub r#sec_prot: Ctrl_WifiSecProt,
    pub r#max_conn: i32,
    pub r#ssid_hidden: bool,
    pub r#bw: i32,
    pub r#band_mode: i32,
}
impl CtrlMsg_Req_StartSoftAP {
    ///Return a reference to `ssid`
    #[inline]
    pub fn r#ssid(&self) -> &::micropb::heapless::String<32> {
        &self.r#ssid
    }
    ///Return a mutable reference to `ssid`
    #[inline]
    pub fn mut_ssid(&mut self) -> &mut ::micropb::heapless::String<32> {
        &mut self.r#ssid
    }
    ///Set the value of `ssid`
    #[inline]
    pub fn set_ssid(&mut self, value: ::micropb::heapless::String<32>) -> &mut Self {
        self.r#ssid = value.into();
        self
    }
    ///Builder method that sets the value of `ssid`. Useful for initializing the message.
    #[inline]
    pub fn init_ssid(mut self, value: ::micropb::heapless::String<32>) -> Self {
        self.r#ssid = value.into();
        self
    }
    ///Return a reference to `pwd`
    #[inline]
    pub fn r#pwd(&self) -> &::micropb::heapless::String<32> {
        &self.r#pwd
    }
    ///Return a mutable reference to `pwd`
    #[inline]
    pub fn mut_pwd(&mut self) -> &mut ::micropb::heapless::String<32> {
        &mut self.r#pwd
    }
    ///Set the value of `pwd`
    #[inline]
    pub fn set_pwd(&mut self, value: ::micropb::heapless::String<32>) -> &mut Self {
        self.r#pwd = value.into();
        self
    }
    ///Builder method that sets the value of `pwd`. Useful for initializing the message.
    #[inline]
    pub fn init_pwd(mut self, value: ::micropb::heapless::String<32>) -> Self {
        self.r#pwd = value.into();
        self
    }
    ///Return a reference to `chnl`
    #[inline]
    pub fn r#chnl(&self) -> &i32 {
        &self.r#chnl
    }
    ///Return a mutable reference to `chnl`
    #[inline]
    pub fn mut_chnl(&mut self) -> &mut i32 {
        &mut self.r#chnl
    }
    ///Set the value of `chnl`
    #[inline]
    pub fn set_chnl(&mut self, value: i32) -> &mut Self {
        self.r#chnl = value.into();
        self
    }
    ///Builder method that sets the value of `chnl`. Useful for initializing the message.
    #[inline]
    pub fn init_chnl(mut self, value: i32) -> Self {
        self.r#chnl = value.into();
        self
    }
    ///Return a reference to `sec_prot`
    #[inline]
    pub fn r#sec_prot(&self) -> &Ctrl_WifiSecProt {
        &self.r#sec_prot
    }
    ///Return a mutable reference to `sec_prot`
    #[inline]
    pub fn mut_sec_prot(&mut self) -> &mut Ctrl_WifiSecProt {
        &mut self.r#sec_prot
    }
    ///Set the value of `sec_prot`
    #[inline]
    pub fn set_sec_prot(&mut self, value: Ctrl_WifiSecProt) -> &mut Self {
        self.r#sec_prot = value.into();
        self
    }
    ///Builder method that sets the value of `sec_prot`. Useful for initializing the message.
    #[inline]
    pub fn init_sec_prot(mut self, value: Ctrl_WifiSecProt) -> Self {
        self.r#sec_prot = value.into();
        self
    }
    ///Return a reference to `max_conn`
    #[inline]
    pub fn r#max_conn(&self) -> &i32 {
        &self.r#max_conn
    }
    ///Return a mutable reference to `max_conn`
    #[inline]
    pub fn mut_max_conn(&mut self) -> &mut i32 {
        &mut self.r#max_conn
    }
    ///Set the value of `max_conn`
    #[inline]
    pub fn set_max_conn(&mut self, value: i32) -> &mut Self {
        self.r#max_conn = value.into();
        self
    }
    ///Builder method that sets the value of `max_conn`. Useful for initializing the message.
    #[inline]
    pub fn init_max_conn(mut self, value: i32) -> Self {
        self.r#max_conn = value.into();
        self
    }
    ///Return a reference to `ssid_hidden`
    #[inline]
    pub fn r#ssid_hidden(&self) -> &bool {
        &self.r#ssid_hidden
    }
    ///Return a mutable reference to `ssid_hidden`
    #[inline]
    pub fn mut_ssid_hidden(&mut self) -> &mut bool {
        &mut self.r#ssid_hidden
    }
    ///Set the value of `ssid_hidden`
    #[inline]
    pub fn set_ssid_hidden(&mut self, value: bool) -> &mut Self {
        self.r#ssid_hidden = value.into();
        self
    }
    ///Builder method that sets the value of `ssid_hidden`. Useful for initializing the message.
    #[inline]
    pub fn init_ssid_hidden(mut self, value: bool) -> Self {
        self.r#ssid_hidden = value.into();
        self
    }
    ///Return a reference to `bw`
    #[inline]
    pub fn r#bw(&self) -> &i32 {
        &self.r#bw
    }
    ///Return a mutable reference to `bw`
    #[inline]
    pub fn mut_bw(&mut self) -> &mut i32 {
        &mut self.r#bw
    }
    ///Set the value of `bw`
    #[inline]
    pub fn set_bw(&mut self, value: i32) -> &mut Self {
        self.r#bw = value.into();
        self
    }
    ///Builder method that sets the value of `bw`. Useful for initializing the message.
    #[inline]
    pub fn init_bw(mut self, value: i32) -> Self {
        self.r#bw = value.into();
        self
    }
    ///Return a reference to `band_mode`
    #[inline]
    pub fn r#band_mode(&self) -> &i32 {
        &self.r#band_mode
    }
    ///Return a mutable reference to `band_mode`
    #[inline]
    pub fn mut_band_mode(&mut self) -> &mut i32 {
        &mut self.r#band_mode
    }
    ///Set the value of `band_mode`
    #[inline]
    pub fn set_band_mode(&mut self, value: i32) -> &mut Self {
        self.r#band_mode = value.into();
        self
    }
    ///Builder method that sets the value of `band_mode`. Useful for initializing the message.
    #[inline]
    pub fn init_band_mode(mut self, value: i32) -> Self {
        self.r#band_mode = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Req_StartSoftAP {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#ssid;
                    {
                        decoder.decode_string(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#pwd;
                    {
                        decoder.decode_string(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#chnl;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                4u32 => {
                    let mut_ref = &mut self.r#sec_prot;
                    {
                        let val = decoder.decode_int32().map(|n| Ctrl_WifiSecProt(n as _))?;
                        let val_ref = &val;
                        if val_ref.0 != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                5u32 => {
                    let mut_ref = &mut self.r#max_conn;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                6u32 => {
                    let mut_ref = &mut self.r#ssid_hidden;
                    {
                        let val = decoder.decode_bool()?;
                        let val_ref = &val;
                        if *val_ref {
                            *mut_ref = val as _;
                        }
                    };
                }
                7u32 => {
                    let mut_ref = &mut self.r#bw;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                8u32 => {
                    let mut_ref = &mut self.r#band_mode;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_StartSoftAP {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(Ctrl_WifiSecProt::_MAX_SIZE), |size| size
                + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(1usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#ssid;
            if !val_ref.is_empty() {
                encoder.encode_varint32(10u32)?;
                encoder.encode_string(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#pwd;
            if !val_ref.is_empty() {
                encoder.encode_varint32(18u32)?;
                encoder.encode_string(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#chnl;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#sec_prot;
            if val_ref.0 != 0 {
                encoder.encode_varint32(32u32)?;
                encoder.encode_int32(val_ref.0 as _)?;
            }
        }
        {
            let val_ref = &self.r#max_conn;
            if *val_ref != 0 {
                encoder.encode_varint32(40u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#ssid_hidden;
            if *val_ref {
                encoder.encode_varint32(48u32)?;
                encoder.encode_bool(*val_ref)?;
            }
        }
        {
            let val_ref = &self.r#bw;
            if *val_ref != 0 {
                encoder.encode_varint32(56u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#band_mode;
            if *val_ref != 0 {
                encoder.encode_varint32(64u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#ssid;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#pwd;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#chnl;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#sec_prot;
            if val_ref.0 != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(val_ref.0 as _);
            }
        }
        {
            let val_ref = &self.r#max_conn;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#ssid_hidden;
            if *val_ref {
                size += 1usize + 1;
            }
        }
        {
            let val_ref = &self.r#bw;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#band_mode;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_StartSoftAP {
    pub r#resp: i32,
    pub r#mac: ::micropb::heapless::Vec<u8, 32>,
    pub r#band_mode: i32,
}
impl CtrlMsg_Resp_StartSoftAP {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
    ///Return a reference to `mac`
    #[inline]
    pub fn r#mac(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#mac
    }
    ///Return a mutable reference to `mac`
    #[inline]
    pub fn mut_mac(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#mac
    }
    ///Set the value of `mac`
    #[inline]
    pub fn set_mac(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#mac = value.into();
        self
    }
    ///Builder method that sets the value of `mac`. Useful for initializing the message.
    #[inline]
    pub fn init_mac(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#mac = value.into();
        self
    }
    ///Return a reference to `band_mode`
    #[inline]
    pub fn r#band_mode(&self) -> &i32 {
        &self.r#band_mode
    }
    ///Return a mutable reference to `band_mode`
    #[inline]
    pub fn mut_band_mode(&mut self) -> &mut i32 {
        &mut self.r#band_mode
    }
    ///Set the value of `band_mode`
    #[inline]
    pub fn set_band_mode(&mut self, value: i32) -> &mut Self {
        self.r#band_mode = value.into();
        self
    }
    ///Builder method that sets the value of `band_mode`. Useful for initializing the message.
    #[inline]
    pub fn init_band_mode(mut self, value: i32) -> Self {
        self.r#band_mode = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_StartSoftAP {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#mac;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#band_mode;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_StartSoftAP {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#mac;
            if !val_ref.is_empty() {
                encoder.encode_varint32(18u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#band_mode;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#mac;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#band_mode;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_ScanResult {}
impl CtrlMsg_Req_ScanResult {}
impl ::micropb::MessageDecode for CtrlMsg_Req_ScanResult {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_ScanResult {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_ScanResult {
    pub r#count: u32,
    pub r#entries: ::micropb::heapless::Vec<ScanResult, 16>,
    pub r#resp: i32,
}
impl CtrlMsg_Resp_ScanResult {
    ///Return a reference to `count`
    #[inline]
    pub fn r#count(&self) -> &u32 {
        &self.r#count
    }
    ///Return a mutable reference to `count`
    #[inline]
    pub fn mut_count(&mut self) -> &mut u32 {
        &mut self.r#count
    }
    ///Set the value of `count`
    #[inline]
    pub fn set_count(&mut self, value: u32) -> &mut Self {
        self.r#count = value.into();
        self
    }
    ///Builder method that sets the value of `count`. Useful for initializing the message.
    #[inline]
    pub fn init_count(mut self, value: u32) -> Self {
        self.r#count = value.into();
        self
    }
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_ScanResult {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#count;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut val: ScanResult = ::core::default::Default::default();
                    let mut_ref = &mut val;
                    {
                        mut_ref.decode_len_delimited(decoder)?;
                    };
                    if let (Err(_), false) = (self.r#entries.pb_push(val), decoder.ignore_repeated_cap_err) {
                        return Err(::micropb::DecodeError::Capacity);
                    }
                }
                3u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_ScanResult {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) = ::micropb::const_map!(
            ::micropb::const_map!(<ScanResult as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                ::micropb::size::sizeof_len_record(size)
            }),
            |size| (size + 1usize) * 16usize
        ) {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#count;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            for val_ref in self.r#entries.iter() {
                encoder.encode_varint32(18u32)?;
                val_ref.encode_len_delimited(encoder)?;
            }
        }
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#count;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            for val_ref in self.r#entries.iter() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
            }
        }
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_SoftAPConnectedSTA {}
impl CtrlMsg_Req_SoftAPConnectedSTA {}
impl ::micropb::MessageDecode for CtrlMsg_Req_SoftAPConnectedSTA {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_SoftAPConnectedSTA {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_SoftAPConnectedSTA {
    pub r#num: u32,
    pub r#stations: ::micropb::heapless::Vec<ConnectedSTAList, 16>,
    pub r#resp: i32,
}
impl CtrlMsg_Resp_SoftAPConnectedSTA {
    ///Return a reference to `num`
    #[inline]
    pub fn r#num(&self) -> &u32 {
        &self.r#num
    }
    ///Return a mutable reference to `num`
    #[inline]
    pub fn mut_num(&mut self) -> &mut u32 {
        &mut self.r#num
    }
    ///Set the value of `num`
    #[inline]
    pub fn set_num(&mut self, value: u32) -> &mut Self {
        self.r#num = value.into();
        self
    }
    ///Builder method that sets the value of `num`. Useful for initializing the message.
    #[inline]
    pub fn init_num(mut self, value: u32) -> Self {
        self.r#num = value.into();
        self
    }
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_SoftAPConnectedSTA {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#num;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut val: ConnectedSTAList = ::core::default::Default::default();
                    let mut_ref = &mut val;
                    {
                        mut_ref.decode_len_delimited(decoder)?;
                    };
                    if let (Err(_), false) = (self.r#stations.pb_push(val), decoder.ignore_repeated_cap_err) {
                        return Err(::micropb::DecodeError::Capacity);
                    }
                }
                3u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_SoftAPConnectedSTA {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) = ::micropb::const_map!(
            ::micropb::const_map!(<ConnectedSTAList as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                ::micropb::size::sizeof_len_record(size)
            }),
            |size| (size + 1usize) * 16usize
        ) {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#num;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            for val_ref in self.r#stations.iter() {
                encoder.encode_varint32(18u32)?;
                val_ref.encode_len_delimited(encoder)?;
            }
        }
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#num;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            for val_ref in self.r#stations.iter() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
            }
        }
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_OTABegin {}
impl CtrlMsg_Req_OTABegin {}
impl ::micropb::MessageDecode for CtrlMsg_Req_OTABegin {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_OTABegin {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_OTABegin {
    pub r#resp: i32,
}
impl CtrlMsg_Resp_OTABegin {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_OTABegin {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_OTABegin {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_OTAWrite {
    pub r#ota_data: ::micropb::heapless::Vec<u8, 256>,
}
impl CtrlMsg_Req_OTAWrite {
    ///Return a reference to `ota_data`
    #[inline]
    pub fn r#ota_data(&self) -> &::micropb::heapless::Vec<u8, 256> {
        &self.r#ota_data
    }
    ///Return a mutable reference to `ota_data`
    #[inline]
    pub fn mut_ota_data(&mut self) -> &mut ::micropb::heapless::Vec<u8, 256> {
        &mut self.r#ota_data
    }
    ///Set the value of `ota_data`
    #[inline]
    pub fn set_ota_data(&mut self, value: ::micropb::heapless::Vec<u8, 256>) -> &mut Self {
        self.r#ota_data = value.into();
        self
    }
    ///Builder method that sets the value of `ota_data`. Useful for initializing the message.
    #[inline]
    pub fn init_ota_data(mut self, value: ::micropb::heapless::Vec<u8, 256>) -> Self {
        self.r#ota_data = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Req_OTAWrite {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#ota_data;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_OTAWrite {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(1026usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#ota_data;
            if !val_ref.is_empty() {
                encoder.encode_varint32(10u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#ota_data;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_OTAWrite {
    pub r#resp: i32,
}
impl CtrlMsg_Resp_OTAWrite {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_OTAWrite {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_OTAWrite {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_OTAEnd {}
impl CtrlMsg_Req_OTAEnd {}
impl ::micropb::MessageDecode for CtrlMsg_Req_OTAEnd {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_OTAEnd {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_OTAEnd {
    pub r#resp: i32,
}
impl CtrlMsg_Resp_OTAEnd {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_OTAEnd {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_OTAEnd {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_VendorIEData {
    pub r#element_id: i32,
    pub r#length: i32,
    pub r#vendor_oui: ::micropb::heapless::Vec<u8, 32>,
    pub r#vendor_oui_type: i32,
    pub r#payload: ::micropb::heapless::Vec<u8, 64>,
}
impl CtrlMsg_Req_VendorIEData {
    ///Return a reference to `element_id`
    #[inline]
    pub fn r#element_id(&self) -> &i32 {
        &self.r#element_id
    }
    ///Return a mutable reference to `element_id`
    #[inline]
    pub fn mut_element_id(&mut self) -> &mut i32 {
        &mut self.r#element_id
    }
    ///Set the value of `element_id`
    #[inline]
    pub fn set_element_id(&mut self, value: i32) -> &mut Self {
        self.r#element_id = value.into();
        self
    }
    ///Builder method that sets the value of `element_id`. Useful for initializing the message.
    #[inline]
    pub fn init_element_id(mut self, value: i32) -> Self {
        self.r#element_id = value.into();
        self
    }
    ///Return a reference to `length`
    #[inline]
    pub fn r#length(&self) -> &i32 {
        &self.r#length
    }
    ///Return a mutable reference to `length`
    #[inline]
    pub fn mut_length(&mut self) -> &mut i32 {
        &mut self.r#length
    }
    ///Set the value of `length`
    #[inline]
    pub fn set_length(&mut self, value: i32) -> &mut Self {
        self.r#length = value.into();
        self
    }
    ///Builder method that sets the value of `length`. Useful for initializing the message.
    #[inline]
    pub fn init_length(mut self, value: i32) -> Self {
        self.r#length = value.into();
        self
    }
    ///Return a reference to `vendor_oui`
    #[inline]
    pub fn r#vendor_oui(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#vendor_oui
    }
    ///Return a mutable reference to `vendor_oui`
    #[inline]
    pub fn mut_vendor_oui(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#vendor_oui
    }
    ///Set the value of `vendor_oui`
    #[inline]
    pub fn set_vendor_oui(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#vendor_oui = value.into();
        self
    }
    ///Builder method that sets the value of `vendor_oui`. Useful for initializing the message.
    #[inline]
    pub fn init_vendor_oui(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#vendor_oui = value.into();
        self
    }
    ///Return a reference to `vendor_oui_type`
    #[inline]
    pub fn r#vendor_oui_type(&self) -> &i32 {
        &self.r#vendor_oui_type
    }
    ///Return a mutable reference to `vendor_oui_type`
    #[inline]
    pub fn mut_vendor_oui_type(&mut self) -> &mut i32 {
        &mut self.r#vendor_oui_type
    }
    ///Set the value of `vendor_oui_type`
    #[inline]
    pub fn set_vendor_oui_type(&mut self, value: i32) -> &mut Self {
        self.r#vendor_oui_type = value.into();
        self
    }
    ///Builder method that sets the value of `vendor_oui_type`. Useful for initializing the message.
    #[inline]
    pub fn init_vendor_oui_type(mut self, value: i32) -> Self {
        self.r#vendor_oui_type = value.into();
        self
    }
    ///Return a reference to `payload`
    #[inline]
    pub fn r#payload(&self) -> &::micropb::heapless::Vec<u8, 64> {
        &self.r#payload
    }
    ///Return a mutable reference to `payload`
    #[inline]
    pub fn mut_payload(&mut self) -> &mut ::micropb::heapless::Vec<u8, 64> {
        &mut self.r#payload
    }
    ///Set the value of `payload`
    #[inline]
    pub fn set_payload(&mut self, value: ::micropb::heapless::Vec<u8, 64>) -> &mut Self {
        self.r#payload = value.into();
        self
    }
    ///Builder method that sets the value of `payload`. Useful for initializing the message.
    #[inline]
    pub fn init_payload(mut self, value: ::micropb::heapless::Vec<u8, 64>) -> Self {
        self.r#payload = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Req_VendorIEData {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#element_id;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#length;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#vendor_oui;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                4u32 => {
                    let mut_ref = &mut self.r#vendor_oui_type;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                5u32 => {
                    let mut_ref = &mut self.r#payload;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_VendorIEData {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(65usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#element_id;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#length;
            if *val_ref != 0 {
                encoder.encode_varint32(16u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#vendor_oui;
            if !val_ref.is_empty() {
                encoder.encode_varint32(26u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#vendor_oui_type;
            if *val_ref != 0 {
                encoder.encode_varint32(32u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#payload;
            if !val_ref.is_empty() {
                encoder.encode_varint32(42u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#element_id;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#length;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#vendor_oui;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#vendor_oui_type;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#payload;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        size
    }
}
pub mod CtrlMsg_Req_SetSoftAPVendorSpecificIE_ {
    #[derive(Debug, Default, PartialEq, Clone)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub struct _Hazzer([u8; 1]);
    impl _Hazzer {
        ///New hazzer with all fields set to off
        #[inline]
        pub const fn _new() -> Self {
            Self([0; 1])
        }
        ///Query presence of `vendor_ie_data`
        #[inline]
        pub const fn r#vendor_ie_data(&self) -> bool {
            (self.0[0] & 1) != 0
        }
        ///Set presence of `vendor_ie_data`
        #[inline]
        pub const fn set_vendor_ie_data(&mut self) -> &mut Self {
            let elem = &mut self.0[0];
            *elem |= 1;
            self
        }
        ///Clear presence of `vendor_ie_data`
        #[inline]
        pub const fn clear_vendor_ie_data(&mut self) -> &mut Self {
            let elem = &mut self.0[0];
            *elem &= !1;
            self
        }
        ///Builder method that sets the presence of `vendor_ie_data`. Useful for initializing the Hazzer.
        #[inline]
        pub const fn init_vendor_ie_data(mut self) -> Self {
            self.set_vendor_ie_data();
            self
        }
    }
}
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_SetSoftAPVendorSpecificIE {
    pub r#enable: bool,
    pub r#type: Ctrl_VendorIEType,
    pub r#idx: Ctrl_VendorIEID,
    pub r#vendor_ie_data: CtrlMsg_Req_VendorIEData,
    pub _has: CtrlMsg_Req_SetSoftAPVendorSpecificIE_::_Hazzer,
}
impl ::core::cmp::PartialEq for CtrlMsg_Req_SetSoftAPVendorSpecificIE {
    fn eq(&self, other: &Self) -> bool {
        let mut ret = true;
        ret &= (self.r#enable == other.r#enable);
        ret &= (self.r#type == other.r#type);
        ret &= (self.r#idx == other.r#idx);
        ret &= (self.r#vendor_ie_data() == other.r#vendor_ie_data());
        ret
    }
}
impl CtrlMsg_Req_SetSoftAPVendorSpecificIE {
    ///Return a reference to `enable`
    #[inline]
    pub fn r#enable(&self) -> &bool {
        &self.r#enable
    }
    ///Return a mutable reference to `enable`
    #[inline]
    pub fn mut_enable(&mut self) -> &mut bool {
        &mut self.r#enable
    }
    ///Set the value of `enable`
    #[inline]
    pub fn set_enable(&mut self, value: bool) -> &mut Self {
        self.r#enable = value.into();
        self
    }
    ///Builder method that sets the value of `enable`. Useful for initializing the message.
    #[inline]
    pub fn init_enable(mut self, value: bool) -> Self {
        self.r#enable = value.into();
        self
    }
    ///Return a reference to `type`
    #[inline]
    pub fn r#type(&self) -> &Ctrl_VendorIEType {
        &self.r#type
    }
    ///Return a mutable reference to `type`
    #[inline]
    pub fn mut_type(&mut self) -> &mut Ctrl_VendorIEType {
        &mut self.r#type
    }
    ///Set the value of `type`
    #[inline]
    pub fn set_type(&mut self, value: Ctrl_VendorIEType) -> &mut Self {
        self.r#type = value.into();
        self
    }
    ///Builder method that sets the value of `type`. Useful for initializing the message.
    #[inline]
    pub fn init_type(mut self, value: Ctrl_VendorIEType) -> Self {
        self.r#type = value.into();
        self
    }
    ///Return a reference to `idx`
    #[inline]
    pub fn r#idx(&self) -> &Ctrl_VendorIEID {
        &self.r#idx
    }
    ///Return a mutable reference to `idx`
    #[inline]
    pub fn mut_idx(&mut self) -> &mut Ctrl_VendorIEID {
        &mut self.r#idx
    }
    ///Set the value of `idx`
    #[inline]
    pub fn set_idx(&mut self, value: Ctrl_VendorIEID) -> &mut Self {
        self.r#idx = value.into();
        self
    }
    ///Builder method that sets the value of `idx`. Useful for initializing the message.
    #[inline]
    pub fn init_idx(mut self, value: Ctrl_VendorIEID) -> Self {
        self.r#idx = value.into();
        self
    }
    ///Return a reference to `vendor_ie_data` as an `Option`
    #[inline]
    pub fn r#vendor_ie_data(&self) -> ::core::option::Option<&CtrlMsg_Req_VendorIEData> {
        self._has.r#vendor_ie_data().then_some(&self.r#vendor_ie_data)
    }
    ///Set the value and presence of `vendor_ie_data`
    #[inline]
    pub fn set_vendor_ie_data(&mut self, value: CtrlMsg_Req_VendorIEData) -> &mut Self {
        self._has.set_vendor_ie_data();
        self.r#vendor_ie_data = value.into();
        self
    }
    ///Return a mutable reference to `vendor_ie_data` as an `Option`
    #[inline]
    pub fn mut_vendor_ie_data(&mut self) -> ::core::option::Option<&mut CtrlMsg_Req_VendorIEData> {
        self._has.r#vendor_ie_data().then_some(&mut self.r#vendor_ie_data)
    }
    ///Clear the presence of `vendor_ie_data`
    #[inline]
    pub fn clear_vendor_ie_data(&mut self) -> &mut Self {
        self._has.clear_vendor_ie_data();
        self
    }
    ///Take the value of `vendor_ie_data` and clear its presence
    #[inline]
    pub fn take_vendor_ie_data(&mut self) -> ::core::option::Option<CtrlMsg_Req_VendorIEData> {
        let val = self
            ._has
            .r#vendor_ie_data()
            .then(|| ::core::mem::take(&mut self.r#vendor_ie_data));
        self._has.clear_vendor_ie_data();
        val
    }
    ///Builder method that sets the value of `vendor_ie_data`. Useful for initializing the message.
    #[inline]
    pub fn init_vendor_ie_data(mut self, value: CtrlMsg_Req_VendorIEData) -> Self {
        self.set_vendor_ie_data(value);
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Req_SetSoftAPVendorSpecificIE {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#enable;
                    {
                        let val = decoder.decode_bool()?;
                        let val_ref = &val;
                        if *val_ref {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#type;
                    {
                        let val = decoder.decode_int32().map(|n| Ctrl_VendorIEType(n as _))?;
                        let val_ref = &val;
                        if val_ref.0 != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#idx;
                    {
                        let val = decoder.decode_int32().map(|n| Ctrl_VendorIEID(n as _))?;
                        let val_ref = &val;
                        if val_ref.0 != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                4u32 => {
                    let mut_ref = &mut self.r#vendor_ie_data;
                    {
                        mut_ref.decode_len_delimited(decoder)?;
                    };
                    self._has.set_vendor_ie_data();
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_SetSoftAPVendorSpecificIE {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(1usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(Ctrl_VendorIEType::_MAX_SIZE), |size| size
                + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(Ctrl_VendorIEID::_MAX_SIZE), |size| size
                + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) = ::micropb::const_map!(
            ::micropb::const_map!(
                <CtrlMsg_Req_VendorIEData as ::micropb::MessageEncode>::MAX_SIZE,
                |size| ::micropb::size::sizeof_len_record(size)
            ),
            |size| size + 1usize
        ) {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#enable;
            if *val_ref {
                encoder.encode_varint32(8u32)?;
                encoder.encode_bool(*val_ref)?;
            }
        }
        {
            let val_ref = &self.r#type;
            if val_ref.0 != 0 {
                encoder.encode_varint32(16u32)?;
                encoder.encode_int32(val_ref.0 as _)?;
            }
        }
        {
            let val_ref = &self.r#idx;
            if val_ref.0 != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_int32(val_ref.0 as _)?;
            }
        }
        {
            if let ::core::option::Option::Some(val_ref) = self.r#vendor_ie_data() {
                encoder.encode_varint32(34u32)?;
                val_ref.encode_len_delimited(encoder)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#enable;
            if *val_ref {
                size += 1usize + 1;
            }
        }
        {
            let val_ref = &self.r#type;
            if val_ref.0 != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(val_ref.0 as _);
            }
        }
        {
            let val_ref = &self.r#idx;
            if val_ref.0 != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(val_ref.0 as _);
            }
        }
        {
            if let ::core::option::Option::Some(val_ref) = self.r#vendor_ie_data() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_SetSoftAPVendorSpecificIE {
    pub r#resp: i32,
}
impl CtrlMsg_Resp_SetSoftAPVendorSpecificIE {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_SetSoftAPVendorSpecificIE {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_SetSoftAPVendorSpecificIE {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_SetWifiMaxTxPower {
    pub r#wifi_max_tx_power: i32,
}
impl CtrlMsg_Req_SetWifiMaxTxPower {
    ///Return a reference to `wifi_max_tx_power`
    #[inline]
    pub fn r#wifi_max_tx_power(&self) -> &i32 {
        &self.r#wifi_max_tx_power
    }
    ///Return a mutable reference to `wifi_max_tx_power`
    #[inline]
    pub fn mut_wifi_max_tx_power(&mut self) -> &mut i32 {
        &mut self.r#wifi_max_tx_power
    }
    ///Set the value of `wifi_max_tx_power`
    #[inline]
    pub fn set_wifi_max_tx_power(&mut self, value: i32) -> &mut Self {
        self.r#wifi_max_tx_power = value.into();
        self
    }
    ///Builder method that sets the value of `wifi_max_tx_power`. Useful for initializing the message.
    #[inline]
    pub fn init_wifi_max_tx_power(mut self, value: i32) -> Self {
        self.r#wifi_max_tx_power = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Req_SetWifiMaxTxPower {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#wifi_max_tx_power;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_SetWifiMaxTxPower {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#wifi_max_tx_power;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#wifi_max_tx_power;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_SetWifiMaxTxPower {
    pub r#resp: i32,
}
impl CtrlMsg_Resp_SetWifiMaxTxPower {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_SetWifiMaxTxPower {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_SetWifiMaxTxPower {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_GetWifiCurrTxPower {}
impl CtrlMsg_Req_GetWifiCurrTxPower {}
impl ::micropb::MessageDecode for CtrlMsg_Req_GetWifiCurrTxPower {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_GetWifiCurrTxPower {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_GetWifiCurrTxPower {
    pub r#wifi_curr_tx_power: i32,
    pub r#resp: i32,
}
impl CtrlMsg_Resp_GetWifiCurrTxPower {
    ///Return a reference to `wifi_curr_tx_power`
    #[inline]
    pub fn r#wifi_curr_tx_power(&self) -> &i32 {
        &self.r#wifi_curr_tx_power
    }
    ///Return a mutable reference to `wifi_curr_tx_power`
    #[inline]
    pub fn mut_wifi_curr_tx_power(&mut self) -> &mut i32 {
        &mut self.r#wifi_curr_tx_power
    }
    ///Set the value of `wifi_curr_tx_power`
    #[inline]
    pub fn set_wifi_curr_tx_power(&mut self, value: i32) -> &mut Self {
        self.r#wifi_curr_tx_power = value.into();
        self
    }
    ///Builder method that sets the value of `wifi_curr_tx_power`. Useful for initializing the message.
    #[inline]
    pub fn init_wifi_curr_tx_power(mut self, value: i32) -> Self {
        self.r#wifi_curr_tx_power = value.into();
        self
    }
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_GetWifiCurrTxPower {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#wifi_curr_tx_power;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_GetWifiCurrTxPower {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#wifi_curr_tx_power;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(16u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#wifi_curr_tx_power;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_ConfigHeartbeat {
    pub r#enable: bool,
    pub r#duration: i32,
}
impl CtrlMsg_Req_ConfigHeartbeat {
    ///Return a reference to `enable`
    #[inline]
    pub fn r#enable(&self) -> &bool {
        &self.r#enable
    }
    ///Return a mutable reference to `enable`
    #[inline]
    pub fn mut_enable(&mut self) -> &mut bool {
        &mut self.r#enable
    }
    ///Set the value of `enable`
    #[inline]
    pub fn set_enable(&mut self, value: bool) -> &mut Self {
        self.r#enable = value.into();
        self
    }
    ///Builder method that sets the value of `enable`. Useful for initializing the message.
    #[inline]
    pub fn init_enable(mut self, value: bool) -> Self {
        self.r#enable = value.into();
        self
    }
    ///Return a reference to `duration`
    #[inline]
    pub fn r#duration(&self) -> &i32 {
        &self.r#duration
    }
    ///Return a mutable reference to `duration`
    #[inline]
    pub fn mut_duration(&mut self) -> &mut i32 {
        &mut self.r#duration
    }
    ///Set the value of `duration`
    #[inline]
    pub fn set_duration(&mut self, value: i32) -> &mut Self {
        self.r#duration = value.into();
        self
    }
    ///Builder method that sets the value of `duration`. Useful for initializing the message.
    #[inline]
    pub fn init_duration(mut self, value: i32) -> Self {
        self.r#duration = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Req_ConfigHeartbeat {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#enable;
                    {
                        let val = decoder.decode_bool()?;
                        let val_ref = &val;
                        if *val_ref {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#duration;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_ConfigHeartbeat {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(1usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#enable;
            if *val_ref {
                encoder.encode_varint32(8u32)?;
                encoder.encode_bool(*val_ref)?;
            }
        }
        {
            let val_ref = &self.r#duration;
            if *val_ref != 0 {
                encoder.encode_varint32(16u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#enable;
            if *val_ref {
                size += 1usize + 1;
            }
        }
        {
            let val_ref = &self.r#duration;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_ConfigHeartbeat {
    pub r#resp: i32,
}
impl CtrlMsg_Resp_ConfigHeartbeat {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_ConfigHeartbeat {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_ConfigHeartbeat {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_EnableDisable {
    pub r#feature: u32,
    pub r#enable: bool,
}
impl CtrlMsg_Req_EnableDisable {
    ///Return a reference to `feature`
    #[inline]
    pub fn r#feature(&self) -> &u32 {
        &self.r#feature
    }
    ///Return a mutable reference to `feature`
    #[inline]
    pub fn mut_feature(&mut self) -> &mut u32 {
        &mut self.r#feature
    }
    ///Set the value of `feature`
    #[inline]
    pub fn set_feature(&mut self, value: u32) -> &mut Self {
        self.r#feature = value.into();
        self
    }
    ///Builder method that sets the value of `feature`. Useful for initializing the message.
    #[inline]
    pub fn init_feature(mut self, value: u32) -> Self {
        self.r#feature = value.into();
        self
    }
    ///Return a reference to `enable`
    #[inline]
    pub fn r#enable(&self) -> &bool {
        &self.r#enable
    }
    ///Return a mutable reference to `enable`
    #[inline]
    pub fn mut_enable(&mut self) -> &mut bool {
        &mut self.r#enable
    }
    ///Set the value of `enable`
    #[inline]
    pub fn set_enable(&mut self, value: bool) -> &mut Self {
        self.r#enable = value.into();
        self
    }
    ///Builder method that sets the value of `enable`. Useful for initializing the message.
    #[inline]
    pub fn init_enable(mut self, value: bool) -> Self {
        self.r#enable = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Req_EnableDisable {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#feature;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#enable;
                    {
                        let val = decoder.decode_bool()?;
                        let val_ref = &val;
                        if *val_ref {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_EnableDisable {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(1usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#feature;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#enable;
            if *val_ref {
                encoder.encode_varint32(16u32)?;
                encoder.encode_bool(*val_ref)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#feature;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#enable;
            if *val_ref {
                size += 1usize + 1;
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_EnableDisable {
    pub r#resp: i32,
}
impl CtrlMsg_Resp_EnableDisable {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_EnableDisable {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_EnableDisable {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_GetFwVersion {}
impl CtrlMsg_Req_GetFwVersion {}
impl ::micropb::MessageDecode for CtrlMsg_Req_GetFwVersion {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_GetFwVersion {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_GetFwVersion {
    pub r#resp: i32,
    pub r#name: ::micropb::heapless::String<32>,
    pub r#major1: u32,
    pub r#major2: u32,
    pub r#minor: u32,
    pub r#rev_patch1: u32,
    pub r#rev_patch2: u32,
}
impl CtrlMsg_Resp_GetFwVersion {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
    ///Return a reference to `name`
    #[inline]
    pub fn r#name(&self) -> &::micropb::heapless::String<32> {
        &self.r#name
    }
    ///Return a mutable reference to `name`
    #[inline]
    pub fn mut_name(&mut self) -> &mut ::micropb::heapless::String<32> {
        &mut self.r#name
    }
    ///Set the value of `name`
    #[inline]
    pub fn set_name(&mut self, value: ::micropb::heapless::String<32>) -> &mut Self {
        self.r#name = value.into();
        self
    }
    ///Builder method that sets the value of `name`. Useful for initializing the message.
    #[inline]
    pub fn init_name(mut self, value: ::micropb::heapless::String<32>) -> Self {
        self.r#name = value.into();
        self
    }
    ///Return a reference to `major1`
    #[inline]
    pub fn r#major1(&self) -> &u32 {
        &self.r#major1
    }
    ///Return a mutable reference to `major1`
    #[inline]
    pub fn mut_major1(&mut self) -> &mut u32 {
        &mut self.r#major1
    }
    ///Set the value of `major1`
    #[inline]
    pub fn set_major1(&mut self, value: u32) -> &mut Self {
        self.r#major1 = value.into();
        self
    }
    ///Builder method that sets the value of `major1`. Useful for initializing the message.
    #[inline]
    pub fn init_major1(mut self, value: u32) -> Self {
        self.r#major1 = value.into();
        self
    }
    ///Return a reference to `major2`
    #[inline]
    pub fn r#major2(&self) -> &u32 {
        &self.r#major2
    }
    ///Return a mutable reference to `major2`
    #[inline]
    pub fn mut_major2(&mut self) -> &mut u32 {
        &mut self.r#major2
    }
    ///Set the value of `major2`
    #[inline]
    pub fn set_major2(&mut self, value: u32) -> &mut Self {
        self.r#major2 = value.into();
        self
    }
    ///Builder method that sets the value of `major2`. Useful for initializing the message.
    #[inline]
    pub fn init_major2(mut self, value: u32) -> Self {
        self.r#major2 = value.into();
        self
    }
    ///Return a reference to `minor`
    #[inline]
    pub fn r#minor(&self) -> &u32 {
        &self.r#minor
    }
    ///Return a mutable reference to `minor`
    #[inline]
    pub fn mut_minor(&mut self) -> &mut u32 {
        &mut self.r#minor
    }
    ///Set the value of `minor`
    #[inline]
    pub fn set_minor(&mut self, value: u32) -> &mut Self {
        self.r#minor = value.into();
        self
    }
    ///Builder method that sets the value of `minor`. Useful for initializing the message.
    #[inline]
    pub fn init_minor(mut self, value: u32) -> Self {
        self.r#minor = value.into();
        self
    }
    ///Return a reference to `rev_patch1`
    #[inline]
    pub fn r#rev_patch1(&self) -> &u32 {
        &self.r#rev_patch1
    }
    ///Return a mutable reference to `rev_patch1`
    #[inline]
    pub fn mut_rev_patch1(&mut self) -> &mut u32 {
        &mut self.r#rev_patch1
    }
    ///Set the value of `rev_patch1`
    #[inline]
    pub fn set_rev_patch1(&mut self, value: u32) -> &mut Self {
        self.r#rev_patch1 = value.into();
        self
    }
    ///Builder method that sets the value of `rev_patch1`. Useful for initializing the message.
    #[inline]
    pub fn init_rev_patch1(mut self, value: u32) -> Self {
        self.r#rev_patch1 = value.into();
        self
    }
    ///Return a reference to `rev_patch2`
    #[inline]
    pub fn r#rev_patch2(&self) -> &u32 {
        &self.r#rev_patch2
    }
    ///Return a mutable reference to `rev_patch2`
    #[inline]
    pub fn mut_rev_patch2(&mut self) -> &mut u32 {
        &mut self.r#rev_patch2
    }
    ///Set the value of `rev_patch2`
    #[inline]
    pub fn set_rev_patch2(&mut self, value: u32) -> &mut Self {
        self.r#rev_patch2 = value.into();
        self
    }
    ///Builder method that sets the value of `rev_patch2`. Useful for initializing the message.
    #[inline]
    pub fn init_rev_patch2(mut self, value: u32) -> Self {
        self.r#rev_patch2 = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_GetFwVersion {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#name;
                    {
                        decoder.decode_string(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#major1;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                4u32 => {
                    let mut_ref = &mut self.r#major2;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                5u32 => {
                    let mut_ref = &mut self.r#minor;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                6u32 => {
                    let mut_ref = &mut self.r#rev_patch1;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                7u32 => {
                    let mut_ref = &mut self.r#rev_patch2;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_GetFwVersion {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#name;
            if !val_ref.is_empty() {
                encoder.encode_varint32(18u32)?;
                encoder.encode_string(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#major1;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#major2;
            if *val_ref != 0 {
                encoder.encode_varint32(32u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#minor;
            if *val_ref != 0 {
                encoder.encode_varint32(40u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#rev_patch1;
            if *val_ref != 0 {
                encoder.encode_varint32(48u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#rev_patch2;
            if *val_ref != 0 {
                encoder.encode_varint32(56u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#name;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#major1;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#major2;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#minor;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#rev_patch1;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#rev_patch2;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_SetCountryCode {
    pub r#country: ::micropb::heapless::Vec<u8, 32>,
    pub r#ieee80211d_enabled: bool,
}
impl CtrlMsg_Req_SetCountryCode {
    ///Return a reference to `country`
    #[inline]
    pub fn r#country(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#country
    }
    ///Return a mutable reference to `country`
    #[inline]
    pub fn mut_country(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#country
    }
    ///Set the value of `country`
    #[inline]
    pub fn set_country(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#country = value.into();
        self
    }
    ///Builder method that sets the value of `country`. Useful for initializing the message.
    #[inline]
    pub fn init_country(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#country = value.into();
        self
    }
    ///Return a reference to `ieee80211d_enabled`
    #[inline]
    pub fn r#ieee80211d_enabled(&self) -> &bool {
        &self.r#ieee80211d_enabled
    }
    ///Return a mutable reference to `ieee80211d_enabled`
    #[inline]
    pub fn mut_ieee80211d_enabled(&mut self) -> &mut bool {
        &mut self.r#ieee80211d_enabled
    }
    ///Set the value of `ieee80211d_enabled`
    #[inline]
    pub fn set_ieee80211d_enabled(&mut self, value: bool) -> &mut Self {
        self.r#ieee80211d_enabled = value.into();
        self
    }
    ///Builder method that sets the value of `ieee80211d_enabled`. Useful for initializing the message.
    #[inline]
    pub fn init_ieee80211d_enabled(mut self, value: bool) -> Self {
        self.r#ieee80211d_enabled = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Req_SetCountryCode {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#country;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#ieee80211d_enabled;
                    {
                        let val = decoder.decode_bool()?;
                        let val_ref = &val;
                        if *val_ref {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_SetCountryCode {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(1usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#country;
            if !val_ref.is_empty() {
                encoder.encode_varint32(10u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#ieee80211d_enabled;
            if *val_ref {
                encoder.encode_varint32(16u32)?;
                encoder.encode_bool(*val_ref)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#country;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#ieee80211d_enabled;
            if *val_ref {
                size += 1usize + 1;
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_SetCountryCode {
    pub r#resp: i32,
}
impl CtrlMsg_Resp_SetCountryCode {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_SetCountryCode {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_SetCountryCode {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_GetCountryCode {}
impl CtrlMsg_Req_GetCountryCode {}
impl ::micropb::MessageDecode for CtrlMsg_Req_GetCountryCode {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_GetCountryCode {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_GetCountryCode {
    pub r#resp: i32,
    pub r#country: ::micropb::heapless::Vec<u8, 32>,
}
impl CtrlMsg_Resp_GetCountryCode {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
    ///Return a reference to `country`
    #[inline]
    pub fn r#country(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#country
    }
    ///Return a mutable reference to `country`
    #[inline]
    pub fn mut_country(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#country
    }
    ///Set the value of `country`
    #[inline]
    pub fn set_country(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#country = value.into();
        self
    }
    ///Builder method that sets the value of `country`. Useful for initializing the message.
    #[inline]
    pub fn init_country(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#country = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_GetCountryCode {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#country;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_GetCountryCode {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#country;
            if !val_ref.is_empty() {
                encoder.encode_varint32(18u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#country;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_SetDhcpDnsStatus {
    pub r#iface: i32,
    pub r#net_link_up: i32,
    pub r#dhcp_up: i32,
    pub r#dhcp_ip: ::micropb::heapless::Vec<u8, 32>,
    pub r#dhcp_nm: ::micropb::heapless::Vec<u8, 32>,
    pub r#dhcp_gw: ::micropb::heapless::Vec<u8, 32>,
    pub r#dns_up: i32,
    pub r#dns_ip: ::micropb::heapless::Vec<u8, 32>,
    pub r#dns_type: i32,
}
impl CtrlMsg_Req_SetDhcpDnsStatus {
    ///Return a reference to `iface`
    #[inline]
    pub fn r#iface(&self) -> &i32 {
        &self.r#iface
    }
    ///Return a mutable reference to `iface`
    #[inline]
    pub fn mut_iface(&mut self) -> &mut i32 {
        &mut self.r#iface
    }
    ///Set the value of `iface`
    #[inline]
    pub fn set_iface(&mut self, value: i32) -> &mut Self {
        self.r#iface = value.into();
        self
    }
    ///Builder method that sets the value of `iface`. Useful for initializing the message.
    #[inline]
    pub fn init_iface(mut self, value: i32) -> Self {
        self.r#iface = value.into();
        self
    }
    ///Return a reference to `net_link_up`
    #[inline]
    pub fn r#net_link_up(&self) -> &i32 {
        &self.r#net_link_up
    }
    ///Return a mutable reference to `net_link_up`
    #[inline]
    pub fn mut_net_link_up(&mut self) -> &mut i32 {
        &mut self.r#net_link_up
    }
    ///Set the value of `net_link_up`
    #[inline]
    pub fn set_net_link_up(&mut self, value: i32) -> &mut Self {
        self.r#net_link_up = value.into();
        self
    }
    ///Builder method that sets the value of `net_link_up`. Useful for initializing the message.
    #[inline]
    pub fn init_net_link_up(mut self, value: i32) -> Self {
        self.r#net_link_up = value.into();
        self
    }
    ///Return a reference to `dhcp_up`
    #[inline]
    pub fn r#dhcp_up(&self) -> &i32 {
        &self.r#dhcp_up
    }
    ///Return a mutable reference to `dhcp_up`
    #[inline]
    pub fn mut_dhcp_up(&mut self) -> &mut i32 {
        &mut self.r#dhcp_up
    }
    ///Set the value of `dhcp_up`
    #[inline]
    pub fn set_dhcp_up(&mut self, value: i32) -> &mut Self {
        self.r#dhcp_up = value.into();
        self
    }
    ///Builder method that sets the value of `dhcp_up`. Useful for initializing the message.
    #[inline]
    pub fn init_dhcp_up(mut self, value: i32) -> Self {
        self.r#dhcp_up = value.into();
        self
    }
    ///Return a reference to `dhcp_ip`
    #[inline]
    pub fn r#dhcp_ip(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#dhcp_ip
    }
    ///Return a mutable reference to `dhcp_ip`
    #[inline]
    pub fn mut_dhcp_ip(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#dhcp_ip
    }
    ///Set the value of `dhcp_ip`
    #[inline]
    pub fn set_dhcp_ip(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#dhcp_ip = value.into();
        self
    }
    ///Builder method that sets the value of `dhcp_ip`. Useful for initializing the message.
    #[inline]
    pub fn init_dhcp_ip(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#dhcp_ip = value.into();
        self
    }
    ///Return a reference to `dhcp_nm`
    #[inline]
    pub fn r#dhcp_nm(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#dhcp_nm
    }
    ///Return a mutable reference to `dhcp_nm`
    #[inline]
    pub fn mut_dhcp_nm(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#dhcp_nm
    }
    ///Set the value of `dhcp_nm`
    #[inline]
    pub fn set_dhcp_nm(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#dhcp_nm = value.into();
        self
    }
    ///Builder method that sets the value of `dhcp_nm`. Useful for initializing the message.
    #[inline]
    pub fn init_dhcp_nm(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#dhcp_nm = value.into();
        self
    }
    ///Return a reference to `dhcp_gw`
    #[inline]
    pub fn r#dhcp_gw(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#dhcp_gw
    }
    ///Return a mutable reference to `dhcp_gw`
    #[inline]
    pub fn mut_dhcp_gw(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#dhcp_gw
    }
    ///Set the value of `dhcp_gw`
    #[inline]
    pub fn set_dhcp_gw(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#dhcp_gw = value.into();
        self
    }
    ///Builder method that sets the value of `dhcp_gw`. Useful for initializing the message.
    #[inline]
    pub fn init_dhcp_gw(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#dhcp_gw = value.into();
        self
    }
    ///Return a reference to `dns_up`
    #[inline]
    pub fn r#dns_up(&self) -> &i32 {
        &self.r#dns_up
    }
    ///Return a mutable reference to `dns_up`
    #[inline]
    pub fn mut_dns_up(&mut self) -> &mut i32 {
        &mut self.r#dns_up
    }
    ///Set the value of `dns_up`
    #[inline]
    pub fn set_dns_up(&mut self, value: i32) -> &mut Self {
        self.r#dns_up = value.into();
        self
    }
    ///Builder method that sets the value of `dns_up`. Useful for initializing the message.
    #[inline]
    pub fn init_dns_up(mut self, value: i32) -> Self {
        self.r#dns_up = value.into();
        self
    }
    ///Return a reference to `dns_ip`
    #[inline]
    pub fn r#dns_ip(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#dns_ip
    }
    ///Return a mutable reference to `dns_ip`
    #[inline]
    pub fn mut_dns_ip(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#dns_ip
    }
    ///Set the value of `dns_ip`
    #[inline]
    pub fn set_dns_ip(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#dns_ip = value.into();
        self
    }
    ///Builder method that sets the value of `dns_ip`. Useful for initializing the message.
    #[inline]
    pub fn init_dns_ip(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#dns_ip = value.into();
        self
    }
    ///Return a reference to `dns_type`
    #[inline]
    pub fn r#dns_type(&self) -> &i32 {
        &self.r#dns_type
    }
    ///Return a mutable reference to `dns_type`
    #[inline]
    pub fn mut_dns_type(&mut self) -> &mut i32 {
        &mut self.r#dns_type
    }
    ///Set the value of `dns_type`
    #[inline]
    pub fn set_dns_type(&mut self, value: i32) -> &mut Self {
        self.r#dns_type = value.into();
        self
    }
    ///Builder method that sets the value of `dns_type`. Useful for initializing the message.
    #[inline]
    pub fn init_dns_type(mut self, value: i32) -> Self {
        self.r#dns_type = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Req_SetDhcpDnsStatus {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#iface;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#net_link_up;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#dhcp_up;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                4u32 => {
                    let mut_ref = &mut self.r#dhcp_ip;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                5u32 => {
                    let mut_ref = &mut self.r#dhcp_nm;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                6u32 => {
                    let mut_ref = &mut self.r#dhcp_gw;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                7u32 => {
                    let mut_ref = &mut self.r#dns_up;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                8u32 => {
                    let mut_ref = &mut self.r#dns_ip;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                9u32 => {
                    let mut_ref = &mut self.r#dns_type;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_SetDhcpDnsStatus {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#iface;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#net_link_up;
            if *val_ref != 0 {
                encoder.encode_varint32(16u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#dhcp_up;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#dhcp_ip;
            if !val_ref.is_empty() {
                encoder.encode_varint32(34u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#dhcp_nm;
            if !val_ref.is_empty() {
                encoder.encode_varint32(42u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#dhcp_gw;
            if !val_ref.is_empty() {
                encoder.encode_varint32(50u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#dns_up;
            if *val_ref != 0 {
                encoder.encode_varint32(56u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#dns_ip;
            if !val_ref.is_empty() {
                encoder.encode_varint32(66u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#dns_type;
            if *val_ref != 0 {
                encoder.encode_varint32(72u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#iface;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#net_link_up;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#dhcp_up;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#dhcp_ip;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#dhcp_nm;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#dhcp_gw;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#dns_up;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#dns_ip;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#dns_type;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_SetDhcpDnsStatus {
    pub r#resp: i32,
}
impl CtrlMsg_Resp_SetDhcpDnsStatus {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_SetDhcpDnsStatus {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_SetDhcpDnsStatus {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_GetDhcpDnsStatus {}
impl CtrlMsg_Req_GetDhcpDnsStatus {}
impl ::micropb::MessageDecode for CtrlMsg_Req_GetDhcpDnsStatus {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_GetDhcpDnsStatus {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_GetDhcpDnsStatus {
    pub r#resp: i32,
    pub r#iface: i32,
    pub r#net_link_up: i32,
    pub r#dhcp_up: i32,
    pub r#dhcp_ip: ::micropb::heapless::Vec<u8, 32>,
    pub r#dhcp_nm: ::micropb::heapless::Vec<u8, 32>,
    pub r#dhcp_gw: ::micropb::heapless::Vec<u8, 32>,
    pub r#dns_up: i32,
    pub r#dns_ip: ::micropb::heapless::Vec<u8, 32>,
    pub r#dns_type: i32,
}
impl CtrlMsg_Resp_GetDhcpDnsStatus {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
    ///Return a reference to `iface`
    #[inline]
    pub fn r#iface(&self) -> &i32 {
        &self.r#iface
    }
    ///Return a mutable reference to `iface`
    #[inline]
    pub fn mut_iface(&mut self) -> &mut i32 {
        &mut self.r#iface
    }
    ///Set the value of `iface`
    #[inline]
    pub fn set_iface(&mut self, value: i32) -> &mut Self {
        self.r#iface = value.into();
        self
    }
    ///Builder method that sets the value of `iface`. Useful for initializing the message.
    #[inline]
    pub fn init_iface(mut self, value: i32) -> Self {
        self.r#iface = value.into();
        self
    }
    ///Return a reference to `net_link_up`
    #[inline]
    pub fn r#net_link_up(&self) -> &i32 {
        &self.r#net_link_up
    }
    ///Return a mutable reference to `net_link_up`
    #[inline]
    pub fn mut_net_link_up(&mut self) -> &mut i32 {
        &mut self.r#net_link_up
    }
    ///Set the value of `net_link_up`
    #[inline]
    pub fn set_net_link_up(&mut self, value: i32) -> &mut Self {
        self.r#net_link_up = value.into();
        self
    }
    ///Builder method that sets the value of `net_link_up`. Useful for initializing the message.
    #[inline]
    pub fn init_net_link_up(mut self, value: i32) -> Self {
        self.r#net_link_up = value.into();
        self
    }
    ///Return a reference to `dhcp_up`
    #[inline]
    pub fn r#dhcp_up(&self) -> &i32 {
        &self.r#dhcp_up
    }
    ///Return a mutable reference to `dhcp_up`
    #[inline]
    pub fn mut_dhcp_up(&mut self) -> &mut i32 {
        &mut self.r#dhcp_up
    }
    ///Set the value of `dhcp_up`
    #[inline]
    pub fn set_dhcp_up(&mut self, value: i32) -> &mut Self {
        self.r#dhcp_up = value.into();
        self
    }
    ///Builder method that sets the value of `dhcp_up`. Useful for initializing the message.
    #[inline]
    pub fn init_dhcp_up(mut self, value: i32) -> Self {
        self.r#dhcp_up = value.into();
        self
    }
    ///Return a reference to `dhcp_ip`
    #[inline]
    pub fn r#dhcp_ip(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#dhcp_ip
    }
    ///Return a mutable reference to `dhcp_ip`
    #[inline]
    pub fn mut_dhcp_ip(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#dhcp_ip
    }
    ///Set the value of `dhcp_ip`
    #[inline]
    pub fn set_dhcp_ip(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#dhcp_ip = value.into();
        self
    }
    ///Builder method that sets the value of `dhcp_ip`. Useful for initializing the message.
    #[inline]
    pub fn init_dhcp_ip(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#dhcp_ip = value.into();
        self
    }
    ///Return a reference to `dhcp_nm`
    #[inline]
    pub fn r#dhcp_nm(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#dhcp_nm
    }
    ///Return a mutable reference to `dhcp_nm`
    #[inline]
    pub fn mut_dhcp_nm(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#dhcp_nm
    }
    ///Set the value of `dhcp_nm`
    #[inline]
    pub fn set_dhcp_nm(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#dhcp_nm = value.into();
        self
    }
    ///Builder method that sets the value of `dhcp_nm`. Useful for initializing the message.
    #[inline]
    pub fn init_dhcp_nm(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#dhcp_nm = value.into();
        self
    }
    ///Return a reference to `dhcp_gw`
    #[inline]
    pub fn r#dhcp_gw(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#dhcp_gw
    }
    ///Return a mutable reference to `dhcp_gw`
    #[inline]
    pub fn mut_dhcp_gw(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#dhcp_gw
    }
    ///Set the value of `dhcp_gw`
    #[inline]
    pub fn set_dhcp_gw(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#dhcp_gw = value.into();
        self
    }
    ///Builder method that sets the value of `dhcp_gw`. Useful for initializing the message.
    #[inline]
    pub fn init_dhcp_gw(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#dhcp_gw = value.into();
        self
    }
    ///Return a reference to `dns_up`
    #[inline]
    pub fn r#dns_up(&self) -> &i32 {
        &self.r#dns_up
    }
    ///Return a mutable reference to `dns_up`
    #[inline]
    pub fn mut_dns_up(&mut self) -> &mut i32 {
        &mut self.r#dns_up
    }
    ///Set the value of `dns_up`
    #[inline]
    pub fn set_dns_up(&mut self, value: i32) -> &mut Self {
        self.r#dns_up = value.into();
        self
    }
    ///Builder method that sets the value of `dns_up`. Useful for initializing the message.
    #[inline]
    pub fn init_dns_up(mut self, value: i32) -> Self {
        self.r#dns_up = value.into();
        self
    }
    ///Return a reference to `dns_ip`
    #[inline]
    pub fn r#dns_ip(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#dns_ip
    }
    ///Return a mutable reference to `dns_ip`
    #[inline]
    pub fn mut_dns_ip(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#dns_ip
    }
    ///Set the value of `dns_ip`
    #[inline]
    pub fn set_dns_ip(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#dns_ip = value.into();
        self
    }
    ///Builder method that sets the value of `dns_ip`. Useful for initializing the message.
    #[inline]
    pub fn init_dns_ip(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#dns_ip = value.into();
        self
    }
    ///Return a reference to `dns_type`
    #[inline]
    pub fn r#dns_type(&self) -> &i32 {
        &self.r#dns_type
    }
    ///Return a mutable reference to `dns_type`
    #[inline]
    pub fn mut_dns_type(&mut self) -> &mut i32 {
        &mut self.r#dns_type
    }
    ///Set the value of `dns_type`
    #[inline]
    pub fn set_dns_type(&mut self, value: i32) -> &mut Self {
        self.r#dns_type = value.into();
        self
    }
    ///Builder method that sets the value of `dns_type`. Useful for initializing the message.
    #[inline]
    pub fn init_dns_type(mut self, value: i32) -> Self {
        self.r#dns_type = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_GetDhcpDnsStatus {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#iface;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#net_link_up;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                4u32 => {
                    let mut_ref = &mut self.r#dhcp_up;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                5u32 => {
                    let mut_ref = &mut self.r#dhcp_ip;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                6u32 => {
                    let mut_ref = &mut self.r#dhcp_nm;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                7u32 => {
                    let mut_ref = &mut self.r#dhcp_gw;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                8u32 => {
                    let mut_ref = &mut self.r#dns_up;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                9u32 => {
                    let mut_ref = &mut self.r#dns_ip;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                10u32 => {
                    let mut_ref = &mut self.r#dns_type;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_GetDhcpDnsStatus {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#iface;
            if *val_ref != 0 {
                encoder.encode_varint32(16u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#net_link_up;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#dhcp_up;
            if *val_ref != 0 {
                encoder.encode_varint32(32u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#dhcp_ip;
            if !val_ref.is_empty() {
                encoder.encode_varint32(42u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#dhcp_nm;
            if !val_ref.is_empty() {
                encoder.encode_varint32(50u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#dhcp_gw;
            if !val_ref.is_empty() {
                encoder.encode_varint32(58u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#dns_up;
            if *val_ref != 0 {
                encoder.encode_varint32(64u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#dns_ip;
            if !val_ref.is_empty() {
                encoder.encode_varint32(74u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#dns_type;
            if *val_ref != 0 {
                encoder.encode_varint32(80u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#iface;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#net_link_up;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#dhcp_up;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#dhcp_ip;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#dhcp_nm;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#dhcp_gw;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#dns_up;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#dns_ip;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#dns_type;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Event_ESPInit {
    pub r#init_data: ::micropb::heapless::Vec<u8, 64>,
}
impl CtrlMsg_Event_ESPInit {
    ///Return a reference to `init_data`
    #[inline]
    pub fn r#init_data(&self) -> &::micropb::heapless::Vec<u8, 64> {
        &self.r#init_data
    }
    ///Return a mutable reference to `init_data`
    #[inline]
    pub fn mut_init_data(&mut self) -> &mut ::micropb::heapless::Vec<u8, 64> {
        &mut self.r#init_data
    }
    ///Set the value of `init_data`
    #[inline]
    pub fn set_init_data(&mut self, value: ::micropb::heapless::Vec<u8, 64>) -> &mut Self {
        self.r#init_data = value.into();
        self
    }
    ///Builder method that sets the value of `init_data`. Useful for initializing the message.
    #[inline]
    pub fn init_init_data(mut self, value: ::micropb::heapless::Vec<u8, 64>) -> Self {
        self.r#init_data = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Event_ESPInit {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#init_data;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Event_ESPInit {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(65usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#init_data;
            if !val_ref.is_empty() {
                encoder.encode_varint32(10u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#init_data;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Event_Heartbeat {
    pub r#hb_num: i32,
}
impl CtrlMsg_Event_Heartbeat {
    ///Return a reference to `hb_num`
    #[inline]
    pub fn r#hb_num(&self) -> &i32 {
        &self.r#hb_num
    }
    ///Return a mutable reference to `hb_num`
    #[inline]
    pub fn mut_hb_num(&mut self) -> &mut i32 {
        &mut self.r#hb_num
    }
    ///Set the value of `hb_num`
    #[inline]
    pub fn set_hb_num(&mut self, value: i32) -> &mut Self {
        self.r#hb_num = value.into();
        self
    }
    ///Builder method that sets the value of `hb_num`. Useful for initializing the message.
    #[inline]
    pub fn init_hb_num(mut self, value: i32) -> Self {
        self.r#hb_num = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Event_Heartbeat {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#hb_num;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Event_Heartbeat {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#hb_num;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#hb_num;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Event_StationDisconnectFromAP {
    pub r#resp: i32,
    pub r#ssid: ::micropb::heapless::Vec<u8, 32>,
    pub r#ssid_len: u32,
    pub r#bssid: ::micropb::heapless::Vec<u8, 32>,
    pub r#reason: u32,
    pub r#rssi: i32,
}
impl CtrlMsg_Event_StationDisconnectFromAP {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
    ///Return a reference to `ssid`
    #[inline]
    pub fn r#ssid(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#ssid
    }
    ///Return a mutable reference to `ssid`
    #[inline]
    pub fn mut_ssid(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#ssid
    }
    ///Set the value of `ssid`
    #[inline]
    pub fn set_ssid(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#ssid = value.into();
        self
    }
    ///Builder method that sets the value of `ssid`. Useful for initializing the message.
    #[inline]
    pub fn init_ssid(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#ssid = value.into();
        self
    }
    ///Return a reference to `ssid_len`
    #[inline]
    pub fn r#ssid_len(&self) -> &u32 {
        &self.r#ssid_len
    }
    ///Return a mutable reference to `ssid_len`
    #[inline]
    pub fn mut_ssid_len(&mut self) -> &mut u32 {
        &mut self.r#ssid_len
    }
    ///Set the value of `ssid_len`
    #[inline]
    pub fn set_ssid_len(&mut self, value: u32) -> &mut Self {
        self.r#ssid_len = value.into();
        self
    }
    ///Builder method that sets the value of `ssid_len`. Useful for initializing the message.
    #[inline]
    pub fn init_ssid_len(mut self, value: u32) -> Self {
        self.r#ssid_len = value.into();
        self
    }
    ///Return a reference to `bssid`
    #[inline]
    pub fn r#bssid(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#bssid
    }
    ///Return a mutable reference to `bssid`
    #[inline]
    pub fn mut_bssid(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#bssid
    }
    ///Set the value of `bssid`
    #[inline]
    pub fn set_bssid(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#bssid = value.into();
        self
    }
    ///Builder method that sets the value of `bssid`. Useful for initializing the message.
    #[inline]
    pub fn init_bssid(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#bssid = value.into();
        self
    }
    ///Return a reference to `reason`
    #[inline]
    pub fn r#reason(&self) -> &u32 {
        &self.r#reason
    }
    ///Return a mutable reference to `reason`
    #[inline]
    pub fn mut_reason(&mut self) -> &mut u32 {
        &mut self.r#reason
    }
    ///Set the value of `reason`
    #[inline]
    pub fn set_reason(&mut self, value: u32) -> &mut Self {
        self.r#reason = value.into();
        self
    }
    ///Builder method that sets the value of `reason`. Useful for initializing the message.
    #[inline]
    pub fn init_reason(mut self, value: u32) -> Self {
        self.r#reason = value.into();
        self
    }
    ///Return a reference to `rssi`
    #[inline]
    pub fn r#rssi(&self) -> &i32 {
        &self.r#rssi
    }
    ///Return a mutable reference to `rssi`
    #[inline]
    pub fn mut_rssi(&mut self) -> &mut i32 {
        &mut self.r#rssi
    }
    ///Set the value of `rssi`
    #[inline]
    pub fn set_rssi(&mut self, value: i32) -> &mut Self {
        self.r#rssi = value.into();
        self
    }
    ///Builder method that sets the value of `rssi`. Useful for initializing the message.
    #[inline]
    pub fn init_rssi(mut self, value: i32) -> Self {
        self.r#rssi = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Event_StationDisconnectFromAP {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#ssid;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#ssid_len;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                4u32 => {
                    let mut_ref = &mut self.r#bssid;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                5u32 => {
                    let mut_ref = &mut self.r#reason;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                6u32 => {
                    let mut_ref = &mut self.r#rssi;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Event_StationDisconnectFromAP {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#ssid;
            if !val_ref.is_empty() {
                encoder.encode_varint32(18u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#ssid_len;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#bssid;
            if !val_ref.is_empty() {
                encoder.encode_varint32(34u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#reason;
            if *val_ref != 0 {
                encoder.encode_varint32(40u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#rssi;
            if *val_ref != 0 {
                encoder.encode_varint32(48u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#ssid;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#ssid_len;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#bssid;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#reason;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#rssi;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Event_StationConnectedToAP {
    pub r#resp: i32,
    pub r#ssid: ::micropb::heapless::Vec<u8, 32>,
    pub r#ssid_len: u32,
    pub r#bssid: ::micropb::heapless::Vec<u8, 32>,
    pub r#channel: u32,
    pub r#authmode: i32,
    pub r#aid: i32,
}
impl CtrlMsg_Event_StationConnectedToAP {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
    ///Return a reference to `ssid`
    #[inline]
    pub fn r#ssid(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#ssid
    }
    ///Return a mutable reference to `ssid`
    #[inline]
    pub fn mut_ssid(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#ssid
    }
    ///Set the value of `ssid`
    #[inline]
    pub fn set_ssid(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#ssid = value.into();
        self
    }
    ///Builder method that sets the value of `ssid`. Useful for initializing the message.
    #[inline]
    pub fn init_ssid(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#ssid = value.into();
        self
    }
    ///Return a reference to `ssid_len`
    #[inline]
    pub fn r#ssid_len(&self) -> &u32 {
        &self.r#ssid_len
    }
    ///Return a mutable reference to `ssid_len`
    #[inline]
    pub fn mut_ssid_len(&mut self) -> &mut u32 {
        &mut self.r#ssid_len
    }
    ///Set the value of `ssid_len`
    #[inline]
    pub fn set_ssid_len(&mut self, value: u32) -> &mut Self {
        self.r#ssid_len = value.into();
        self
    }
    ///Builder method that sets the value of `ssid_len`. Useful for initializing the message.
    #[inline]
    pub fn init_ssid_len(mut self, value: u32) -> Self {
        self.r#ssid_len = value.into();
        self
    }
    ///Return a reference to `bssid`
    #[inline]
    pub fn r#bssid(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#bssid
    }
    ///Return a mutable reference to `bssid`
    #[inline]
    pub fn mut_bssid(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#bssid
    }
    ///Set the value of `bssid`
    #[inline]
    pub fn set_bssid(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#bssid = value.into();
        self
    }
    ///Builder method that sets the value of `bssid`. Useful for initializing the message.
    #[inline]
    pub fn init_bssid(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#bssid = value.into();
        self
    }
    ///Return a reference to `channel`
    #[inline]
    pub fn r#channel(&self) -> &u32 {
        &self.r#channel
    }
    ///Return a mutable reference to `channel`
    #[inline]
    pub fn mut_channel(&mut self) -> &mut u32 {
        &mut self.r#channel
    }
    ///Set the value of `channel`
    #[inline]
    pub fn set_channel(&mut self, value: u32) -> &mut Self {
        self.r#channel = value.into();
        self
    }
    ///Builder method that sets the value of `channel`. Useful for initializing the message.
    #[inline]
    pub fn init_channel(mut self, value: u32) -> Self {
        self.r#channel = value.into();
        self
    }
    ///Return a reference to `authmode`
    #[inline]
    pub fn r#authmode(&self) -> &i32 {
        &self.r#authmode
    }
    ///Return a mutable reference to `authmode`
    #[inline]
    pub fn mut_authmode(&mut self) -> &mut i32 {
        &mut self.r#authmode
    }
    ///Set the value of `authmode`
    #[inline]
    pub fn set_authmode(&mut self, value: i32) -> &mut Self {
        self.r#authmode = value.into();
        self
    }
    ///Builder method that sets the value of `authmode`. Useful for initializing the message.
    #[inline]
    pub fn init_authmode(mut self, value: i32) -> Self {
        self.r#authmode = value.into();
        self
    }
    ///Return a reference to `aid`
    #[inline]
    pub fn r#aid(&self) -> &i32 {
        &self.r#aid
    }
    ///Return a mutable reference to `aid`
    #[inline]
    pub fn mut_aid(&mut self) -> &mut i32 {
        &mut self.r#aid
    }
    ///Set the value of `aid`
    #[inline]
    pub fn set_aid(&mut self, value: i32) -> &mut Self {
        self.r#aid = value.into();
        self
    }
    ///Builder method that sets the value of `aid`. Useful for initializing the message.
    #[inline]
    pub fn init_aid(mut self, value: i32) -> Self {
        self.r#aid = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Event_StationConnectedToAP {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#ssid;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#ssid_len;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                4u32 => {
                    let mut_ref = &mut self.r#bssid;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                5u32 => {
                    let mut_ref = &mut self.r#channel;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                6u32 => {
                    let mut_ref = &mut self.r#authmode;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                7u32 => {
                    let mut_ref = &mut self.r#aid;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Event_StationConnectedToAP {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#ssid;
            if !val_ref.is_empty() {
                encoder.encode_varint32(18u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#ssid_len;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#bssid;
            if !val_ref.is_empty() {
                encoder.encode_varint32(34u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#channel;
            if *val_ref != 0 {
                encoder.encode_varint32(40u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#authmode;
            if *val_ref != 0 {
                encoder.encode_varint32(48u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#aid;
            if *val_ref != 0 {
                encoder.encode_varint32(56u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#ssid;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#ssid_len;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#bssid;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#channel;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#authmode;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#aid;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Event_StationDisconnectFromESPSoftAP {
    pub r#resp: i32,
    pub r#mac: ::micropb::heapless::Vec<u8, 32>,
    pub r#aid: u32,
    pub r#is_mesh_child: bool,
    pub r#reason: u32,
}
impl CtrlMsg_Event_StationDisconnectFromESPSoftAP {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
    ///Return a reference to `mac`
    #[inline]
    pub fn r#mac(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#mac
    }
    ///Return a mutable reference to `mac`
    #[inline]
    pub fn mut_mac(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#mac
    }
    ///Set the value of `mac`
    #[inline]
    pub fn set_mac(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#mac = value.into();
        self
    }
    ///Builder method that sets the value of `mac`. Useful for initializing the message.
    #[inline]
    pub fn init_mac(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#mac = value.into();
        self
    }
    ///Return a reference to `aid`
    #[inline]
    pub fn r#aid(&self) -> &u32 {
        &self.r#aid
    }
    ///Return a mutable reference to `aid`
    #[inline]
    pub fn mut_aid(&mut self) -> &mut u32 {
        &mut self.r#aid
    }
    ///Set the value of `aid`
    #[inline]
    pub fn set_aid(&mut self, value: u32) -> &mut Self {
        self.r#aid = value.into();
        self
    }
    ///Builder method that sets the value of `aid`. Useful for initializing the message.
    #[inline]
    pub fn init_aid(mut self, value: u32) -> Self {
        self.r#aid = value.into();
        self
    }
    ///Return a reference to `is_mesh_child`
    #[inline]
    pub fn r#is_mesh_child(&self) -> &bool {
        &self.r#is_mesh_child
    }
    ///Return a mutable reference to `is_mesh_child`
    #[inline]
    pub fn mut_is_mesh_child(&mut self) -> &mut bool {
        &mut self.r#is_mesh_child
    }
    ///Set the value of `is_mesh_child`
    #[inline]
    pub fn set_is_mesh_child(&mut self, value: bool) -> &mut Self {
        self.r#is_mesh_child = value.into();
        self
    }
    ///Builder method that sets the value of `is_mesh_child`. Useful for initializing the message.
    #[inline]
    pub fn init_is_mesh_child(mut self, value: bool) -> Self {
        self.r#is_mesh_child = value.into();
        self
    }
    ///Return a reference to `reason`
    #[inline]
    pub fn r#reason(&self) -> &u32 {
        &self.r#reason
    }
    ///Return a mutable reference to `reason`
    #[inline]
    pub fn mut_reason(&mut self) -> &mut u32 {
        &mut self.r#reason
    }
    ///Set the value of `reason`
    #[inline]
    pub fn set_reason(&mut self, value: u32) -> &mut Self {
        self.r#reason = value.into();
        self
    }
    ///Builder method that sets the value of `reason`. Useful for initializing the message.
    #[inline]
    pub fn init_reason(mut self, value: u32) -> Self {
        self.r#reason = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Event_StationDisconnectFromESPSoftAP {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#mac;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#aid;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                4u32 => {
                    let mut_ref = &mut self.r#is_mesh_child;
                    {
                        let val = decoder.decode_bool()?;
                        let val_ref = &val;
                        if *val_ref {
                            *mut_ref = val as _;
                        }
                    };
                }
                5u32 => {
                    let mut_ref = &mut self.r#reason;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Event_StationDisconnectFromESPSoftAP {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(1usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#mac;
            if !val_ref.is_empty() {
                encoder.encode_varint32(18u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#aid;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#is_mesh_child;
            if *val_ref {
                encoder.encode_varint32(32u32)?;
                encoder.encode_bool(*val_ref)?;
            }
        }
        {
            let val_ref = &self.r#reason;
            if *val_ref != 0 {
                encoder.encode_varint32(40u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#mac;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#aid;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#is_mesh_child;
            if *val_ref {
                size += 1usize + 1;
            }
        }
        {
            let val_ref = &self.r#reason;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Event_StationConnectedToESPSoftAP {
    pub r#resp: i32,
    pub r#mac: ::micropb::heapless::Vec<u8, 32>,
    pub r#aid: u32,
    pub r#is_mesh_child: bool,
}
impl CtrlMsg_Event_StationConnectedToESPSoftAP {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
    ///Return a reference to `mac`
    #[inline]
    pub fn r#mac(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#mac
    }
    ///Return a mutable reference to `mac`
    #[inline]
    pub fn mut_mac(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#mac
    }
    ///Set the value of `mac`
    #[inline]
    pub fn set_mac(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#mac = value.into();
        self
    }
    ///Builder method that sets the value of `mac`. Useful for initializing the message.
    #[inline]
    pub fn init_mac(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#mac = value.into();
        self
    }
    ///Return a reference to `aid`
    #[inline]
    pub fn r#aid(&self) -> &u32 {
        &self.r#aid
    }
    ///Return a mutable reference to `aid`
    #[inline]
    pub fn mut_aid(&mut self) -> &mut u32 {
        &mut self.r#aid
    }
    ///Set the value of `aid`
    #[inline]
    pub fn set_aid(&mut self, value: u32) -> &mut Self {
        self.r#aid = value.into();
        self
    }
    ///Builder method that sets the value of `aid`. Useful for initializing the message.
    #[inline]
    pub fn init_aid(mut self, value: u32) -> Self {
        self.r#aid = value.into();
        self
    }
    ///Return a reference to `is_mesh_child`
    #[inline]
    pub fn r#is_mesh_child(&self) -> &bool {
        &self.r#is_mesh_child
    }
    ///Return a mutable reference to `is_mesh_child`
    #[inline]
    pub fn mut_is_mesh_child(&mut self) -> &mut bool {
        &mut self.r#is_mesh_child
    }
    ///Set the value of `is_mesh_child`
    #[inline]
    pub fn set_is_mesh_child(&mut self, value: bool) -> &mut Self {
        self.r#is_mesh_child = value.into();
        self
    }
    ///Builder method that sets the value of `is_mesh_child`. Useful for initializing the message.
    #[inline]
    pub fn init_is_mesh_child(mut self, value: bool) -> Self {
        self.r#is_mesh_child = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Event_StationConnectedToESPSoftAP {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#mac;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#aid;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                4u32 => {
                    let mut_ref = &mut self.r#is_mesh_child;
                    {
                        let val = decoder.decode_bool()?;
                        let val_ref = &val;
                        if *val_ref {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Event_StationConnectedToESPSoftAP {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(1usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#mac;
            if !val_ref.is_empty() {
                encoder.encode_varint32(18u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#aid;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#is_mesh_child;
            if *val_ref {
                encoder.encode_varint32(32u32)?;
                encoder.encode_bool(*val_ref)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#mac;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#aid;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#is_mesh_child;
            if *val_ref {
                size += 1usize + 1;
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Event_SetDhcpDnsStatus {
    pub r#iface: i32,
    pub r#net_link_up: i32,
    pub r#dhcp_up: i32,
    pub r#dhcp_ip: ::micropb::heapless::Vec<u8, 32>,
    pub r#dhcp_nm: ::micropb::heapless::Vec<u8, 32>,
    pub r#dhcp_gw: ::micropb::heapless::Vec<u8, 32>,
    pub r#dns_up: i32,
    pub r#dns_ip: ::micropb::heapless::Vec<u8, 32>,
    pub r#dns_type: i32,
    pub r#resp: i32,
}
impl CtrlMsg_Event_SetDhcpDnsStatus {
    ///Return a reference to `iface`
    #[inline]
    pub fn r#iface(&self) -> &i32 {
        &self.r#iface
    }
    ///Return a mutable reference to `iface`
    #[inline]
    pub fn mut_iface(&mut self) -> &mut i32 {
        &mut self.r#iface
    }
    ///Set the value of `iface`
    #[inline]
    pub fn set_iface(&mut self, value: i32) -> &mut Self {
        self.r#iface = value.into();
        self
    }
    ///Builder method that sets the value of `iface`. Useful for initializing the message.
    #[inline]
    pub fn init_iface(mut self, value: i32) -> Self {
        self.r#iface = value.into();
        self
    }
    ///Return a reference to `net_link_up`
    #[inline]
    pub fn r#net_link_up(&self) -> &i32 {
        &self.r#net_link_up
    }
    ///Return a mutable reference to `net_link_up`
    #[inline]
    pub fn mut_net_link_up(&mut self) -> &mut i32 {
        &mut self.r#net_link_up
    }
    ///Set the value of `net_link_up`
    #[inline]
    pub fn set_net_link_up(&mut self, value: i32) -> &mut Self {
        self.r#net_link_up = value.into();
        self
    }
    ///Builder method that sets the value of `net_link_up`. Useful for initializing the message.
    #[inline]
    pub fn init_net_link_up(mut self, value: i32) -> Self {
        self.r#net_link_up = value.into();
        self
    }
    ///Return a reference to `dhcp_up`
    #[inline]
    pub fn r#dhcp_up(&self) -> &i32 {
        &self.r#dhcp_up
    }
    ///Return a mutable reference to `dhcp_up`
    #[inline]
    pub fn mut_dhcp_up(&mut self) -> &mut i32 {
        &mut self.r#dhcp_up
    }
    ///Set the value of `dhcp_up`
    #[inline]
    pub fn set_dhcp_up(&mut self, value: i32) -> &mut Self {
        self.r#dhcp_up = value.into();
        self
    }
    ///Builder method that sets the value of `dhcp_up`. Useful for initializing the message.
    #[inline]
    pub fn init_dhcp_up(mut self, value: i32) -> Self {
        self.r#dhcp_up = value.into();
        self
    }
    ///Return a reference to `dhcp_ip`
    #[inline]
    pub fn r#dhcp_ip(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#dhcp_ip
    }
    ///Return a mutable reference to `dhcp_ip`
    #[inline]
    pub fn mut_dhcp_ip(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#dhcp_ip
    }
    ///Set the value of `dhcp_ip`
    #[inline]
    pub fn set_dhcp_ip(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#dhcp_ip = value.into();
        self
    }
    ///Builder method that sets the value of `dhcp_ip`. Useful for initializing the message.
    #[inline]
    pub fn init_dhcp_ip(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#dhcp_ip = value.into();
        self
    }
    ///Return a reference to `dhcp_nm`
    #[inline]
    pub fn r#dhcp_nm(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#dhcp_nm
    }
    ///Return a mutable reference to `dhcp_nm`
    #[inline]
    pub fn mut_dhcp_nm(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#dhcp_nm
    }
    ///Set the value of `dhcp_nm`
    #[inline]
    pub fn set_dhcp_nm(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#dhcp_nm = value.into();
        self
    }
    ///Builder method that sets the value of `dhcp_nm`. Useful for initializing the message.
    #[inline]
    pub fn init_dhcp_nm(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#dhcp_nm = value.into();
        self
    }
    ///Return a reference to `dhcp_gw`
    #[inline]
    pub fn r#dhcp_gw(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#dhcp_gw
    }
    ///Return a mutable reference to `dhcp_gw`
    #[inline]
    pub fn mut_dhcp_gw(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#dhcp_gw
    }
    ///Set the value of `dhcp_gw`
    #[inline]
    pub fn set_dhcp_gw(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#dhcp_gw = value.into();
        self
    }
    ///Builder method that sets the value of `dhcp_gw`. Useful for initializing the message.
    #[inline]
    pub fn init_dhcp_gw(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#dhcp_gw = value.into();
        self
    }
    ///Return a reference to `dns_up`
    #[inline]
    pub fn r#dns_up(&self) -> &i32 {
        &self.r#dns_up
    }
    ///Return a mutable reference to `dns_up`
    #[inline]
    pub fn mut_dns_up(&mut self) -> &mut i32 {
        &mut self.r#dns_up
    }
    ///Set the value of `dns_up`
    #[inline]
    pub fn set_dns_up(&mut self, value: i32) -> &mut Self {
        self.r#dns_up = value.into();
        self
    }
    ///Builder method that sets the value of `dns_up`. Useful for initializing the message.
    #[inline]
    pub fn init_dns_up(mut self, value: i32) -> Self {
        self.r#dns_up = value.into();
        self
    }
    ///Return a reference to `dns_ip`
    #[inline]
    pub fn r#dns_ip(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#dns_ip
    }
    ///Return a mutable reference to `dns_ip`
    #[inline]
    pub fn mut_dns_ip(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#dns_ip
    }
    ///Set the value of `dns_ip`
    #[inline]
    pub fn set_dns_ip(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#dns_ip = value.into();
        self
    }
    ///Builder method that sets the value of `dns_ip`. Useful for initializing the message.
    #[inline]
    pub fn init_dns_ip(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#dns_ip = value.into();
        self
    }
    ///Return a reference to `dns_type`
    #[inline]
    pub fn r#dns_type(&self) -> &i32 {
        &self.r#dns_type
    }
    ///Return a mutable reference to `dns_type`
    #[inline]
    pub fn mut_dns_type(&mut self) -> &mut i32 {
        &mut self.r#dns_type
    }
    ///Set the value of `dns_type`
    #[inline]
    pub fn set_dns_type(&mut self, value: i32) -> &mut Self {
        self.r#dns_type = value.into();
        self
    }
    ///Builder method that sets the value of `dns_type`. Useful for initializing the message.
    #[inline]
    pub fn init_dns_type(mut self, value: i32) -> Self {
        self.r#dns_type = value.into();
        self
    }
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Event_SetDhcpDnsStatus {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#iface;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#net_link_up;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#dhcp_up;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                4u32 => {
                    let mut_ref = &mut self.r#dhcp_ip;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                5u32 => {
                    let mut_ref = &mut self.r#dhcp_nm;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                6u32 => {
                    let mut_ref = &mut self.r#dhcp_gw;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                7u32 => {
                    let mut_ref = &mut self.r#dns_up;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                8u32 => {
                    let mut_ref = &mut self.r#dns_ip;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                9u32 => {
                    let mut_ref = &mut self.r#dns_type;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                10u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Event_SetDhcpDnsStatus {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#iface;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#net_link_up;
            if *val_ref != 0 {
                encoder.encode_varint32(16u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#dhcp_up;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#dhcp_ip;
            if !val_ref.is_empty() {
                encoder.encode_varint32(34u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#dhcp_nm;
            if !val_ref.is_empty() {
                encoder.encode_varint32(42u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#dhcp_gw;
            if !val_ref.is_empty() {
                encoder.encode_varint32(50u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#dns_up;
            if *val_ref != 0 {
                encoder.encode_varint32(56u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#dns_ip;
            if !val_ref.is_empty() {
                encoder.encode_varint32(66u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        {
            let val_ref = &self.r#dns_type;
            if *val_ref != 0 {
                encoder.encode_varint32(72u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(80u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#iface;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#net_link_up;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#dhcp_up;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#dhcp_ip;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#dhcp_nm;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#dhcp_gw;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#dns_up;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#dns_ip;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        {
            let val_ref = &self.r#dns_type;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Req_CustomRpcUnserialisedMsg {
    pub r#custom_msg_id: u32,
    pub r#data: ::micropb::heapless::Vec<u8, 32>,
}
impl CtrlMsg_Req_CustomRpcUnserialisedMsg {
    ///Return a reference to `custom_msg_id`
    #[inline]
    pub fn r#custom_msg_id(&self) -> &u32 {
        &self.r#custom_msg_id
    }
    ///Return a mutable reference to `custom_msg_id`
    #[inline]
    pub fn mut_custom_msg_id(&mut self) -> &mut u32 {
        &mut self.r#custom_msg_id
    }
    ///Set the value of `custom_msg_id`
    #[inline]
    pub fn set_custom_msg_id(&mut self, value: u32) -> &mut Self {
        self.r#custom_msg_id = value.into();
        self
    }
    ///Builder method that sets the value of `custom_msg_id`. Useful for initializing the message.
    #[inline]
    pub fn init_custom_msg_id(mut self, value: u32) -> Self {
        self.r#custom_msg_id = value.into();
        self
    }
    ///Return a reference to `data`
    #[inline]
    pub fn r#data(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#data
    }
    ///Return a mutable reference to `data`
    #[inline]
    pub fn mut_data(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#data
    }
    ///Set the value of `data`
    #[inline]
    pub fn set_data(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#data = value.into();
        self
    }
    ///Builder method that sets the value of `data`. Useful for initializing the message.
    #[inline]
    pub fn init_data(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#data = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Req_CustomRpcUnserialisedMsg {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#custom_msg_id;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#data;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Req_CustomRpcUnserialisedMsg {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#custom_msg_id;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#data;
            if !val_ref.is_empty() {
                encoder.encode_varint32(18u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#custom_msg_id;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#data;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Resp_CustomRpcUnserialisedMsg {
    pub r#resp: i32,
    pub r#custom_msg_id: u32,
    pub r#data: ::micropb::heapless::Vec<u8, 32>,
}
impl CtrlMsg_Resp_CustomRpcUnserialisedMsg {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
    ///Return a reference to `custom_msg_id`
    #[inline]
    pub fn r#custom_msg_id(&self) -> &u32 {
        &self.r#custom_msg_id
    }
    ///Return a mutable reference to `custom_msg_id`
    #[inline]
    pub fn mut_custom_msg_id(&mut self) -> &mut u32 {
        &mut self.r#custom_msg_id
    }
    ///Set the value of `custom_msg_id`
    #[inline]
    pub fn set_custom_msg_id(&mut self, value: u32) -> &mut Self {
        self.r#custom_msg_id = value.into();
        self
    }
    ///Builder method that sets the value of `custom_msg_id`. Useful for initializing the message.
    #[inline]
    pub fn init_custom_msg_id(mut self, value: u32) -> Self {
        self.r#custom_msg_id = value.into();
        self
    }
    ///Return a reference to `data`
    #[inline]
    pub fn r#data(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#data
    }
    ///Return a mutable reference to `data`
    #[inline]
    pub fn mut_data(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#data
    }
    ///Set the value of `data`
    #[inline]
    pub fn set_data(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#data = value.into();
        self
    }
    ///Builder method that sets the value of `data`. Useful for initializing the message.
    #[inline]
    pub fn init_data(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#data = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Resp_CustomRpcUnserialisedMsg {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#custom_msg_id;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#data;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Resp_CustomRpcUnserialisedMsg {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#custom_msg_id;
            if *val_ref != 0 {
                encoder.encode_varint32(16u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#data;
            if !val_ref.is_empty() {
                encoder.encode_varint32(26u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#custom_msg_id;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#data;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        size
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg_Event_CustomRpcUnserialisedMsg {
    pub r#resp: i32,
    pub r#custom_evt_id: u32,
    pub r#data: ::micropb::heapless::Vec<u8, 32>,
}
impl CtrlMsg_Event_CustomRpcUnserialisedMsg {
    ///Return a reference to `resp`
    #[inline]
    pub fn r#resp(&self) -> &i32 {
        &self.r#resp
    }
    ///Return a mutable reference to `resp`
    #[inline]
    pub fn mut_resp(&mut self) -> &mut i32 {
        &mut self.r#resp
    }
    ///Set the value of `resp`
    #[inline]
    pub fn set_resp(&mut self, value: i32) -> &mut Self {
        self.r#resp = value.into();
        self
    }
    ///Builder method that sets the value of `resp`. Useful for initializing the message.
    #[inline]
    pub fn init_resp(mut self, value: i32) -> Self {
        self.r#resp = value.into();
        self
    }
    ///Return a reference to `custom_evt_id`
    #[inline]
    pub fn r#custom_evt_id(&self) -> &u32 {
        &self.r#custom_evt_id
    }
    ///Return a mutable reference to `custom_evt_id`
    #[inline]
    pub fn mut_custom_evt_id(&mut self) -> &mut u32 {
        &mut self.r#custom_evt_id
    }
    ///Set the value of `custom_evt_id`
    #[inline]
    pub fn set_custom_evt_id(&mut self, value: u32) -> &mut Self {
        self.r#custom_evt_id = value.into();
        self
    }
    ///Builder method that sets the value of `custom_evt_id`. Useful for initializing the message.
    #[inline]
    pub fn init_custom_evt_id(mut self, value: u32) -> Self {
        self.r#custom_evt_id = value.into();
        self
    }
    ///Return a reference to `data`
    #[inline]
    pub fn r#data(&self) -> &::micropb::heapless::Vec<u8, 32> {
        &self.r#data
    }
    ///Return a mutable reference to `data`
    #[inline]
    pub fn mut_data(&mut self) -> &mut ::micropb::heapless::Vec<u8, 32> {
        &mut self.r#data
    }
    ///Set the value of `data`
    #[inline]
    pub fn set_data(&mut self, value: ::micropb::heapless::Vec<u8, 32>) -> &mut Self {
        self.r#data = value.into();
        self
    }
    ///Builder method that sets the value of `data`. Useful for initializing the message.
    #[inline]
    pub fn init_data(mut self, value: ::micropb::heapless::Vec<u8, 32>) -> Self {
        self.r#data = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg_Event_CustomRpcUnserialisedMsg {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#resp;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#custom_evt_id;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#data;
                    {
                        decoder.decode_bytes(mut_ref, ::micropb::Presence::Implicit)?;
                    };
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg_Event_CustomRpcUnserialisedMsg {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(33usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#custom_evt_id;
            if *val_ref != 0 {
                encoder.encode_varint32(16u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#data;
            if !val_ref.is_empty() {
                encoder.encode_varint32(26u32)?;
                encoder.encode_bytes(val_ref)?;
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#resp;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#custom_evt_id;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#data;
            if !val_ref.is_empty() {
                size += 1usize + ::micropb::size::sizeof_len_record(val_ref.len());
            }
        }
        size
    }
}
pub mod CtrlMsg_ {
    #[derive(Debug, PartialEq, Clone)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum Payload {
        ReqGetMacAddress(super::CtrlMsg_Req_GetMacAddress),
        ReqSetMacAddress(super::CtrlMsg_Req_SetMacAddress),
        ReqGetWifiMode(super::CtrlMsg_Req_GetMode),
        ReqSetWifiMode(super::CtrlMsg_Req_SetMode),
        ReqScanApList(super::CtrlMsg_Req_ScanResult),
        ReqGetApConfig(super::CtrlMsg_Req_GetAPConfig),
        ReqConnectAp(super::CtrlMsg_Req_ConnectAP),
        ReqDisconnectAp(super::CtrlMsg_Req_GetStatus),
        ReqGetSoftapConfig(super::CtrlMsg_Req_GetSoftAPConfig),
        ReqSetSoftapVendorSpecificIe(super::CtrlMsg_Req_SetSoftAPVendorSpecificIE),
        ReqStartSoftap(super::CtrlMsg_Req_StartSoftAP),
        ReqSoftapConnectedStasList(super::CtrlMsg_Req_SoftAPConnectedSTA),
        ReqStopSoftap(super::CtrlMsg_Req_GetStatus),
        ReqSetPowerSaveMode(super::CtrlMsg_Req_SetMode),
        ReqGetPowerSaveMode(super::CtrlMsg_Req_GetMode),
        ReqOtaBegin(super::CtrlMsg_Req_OTABegin),
        ReqOtaWrite(super::CtrlMsg_Req_OTAWrite),
        ReqOtaEnd(super::CtrlMsg_Req_OTAEnd),
        ReqSetWifiMaxTxPower(super::CtrlMsg_Req_SetWifiMaxTxPower),
        ReqGetWifiCurrTxPower(super::CtrlMsg_Req_GetWifiCurrTxPower),
        ReqConfigHeartbeat(super::CtrlMsg_Req_ConfigHeartbeat),
        ReqEnableDisableFeat(super::CtrlMsg_Req_EnableDisable),
        ReqGetFwVersion(super::CtrlMsg_Req_GetFwVersion),
        ReqSetCountryCode(super::CtrlMsg_Req_SetCountryCode),
        ReqGetCountryCode(super::CtrlMsg_Req_GetCountryCode),
        ReqSetDhcpDnsStatus(super::CtrlMsg_Req_SetDhcpDnsStatus),
        ReqGetDhcpDnsStatus(super::CtrlMsg_Req_GetDhcpDnsStatus),
        ReqCustomRpcUnserialisedMsg(super::CtrlMsg_Req_CustomRpcUnserialisedMsg),
        RespGetMacAddress(super::CtrlMsg_Resp_GetMacAddress),
        RespSetMacAddress(super::CtrlMsg_Resp_SetMacAddress),
        RespGetWifiMode(super::CtrlMsg_Resp_GetMode),
        RespSetWifiMode(super::CtrlMsg_Resp_SetMode),
        RespScanApList(super::CtrlMsg_Resp_ScanResult),
        RespGetApConfig(super::CtrlMsg_Resp_GetAPConfig),
        RespConnectAp(super::CtrlMsg_Resp_ConnectAP),
        RespDisconnectAp(super::CtrlMsg_Resp_GetStatus),
        RespGetSoftapConfig(super::CtrlMsg_Resp_GetSoftAPConfig),
        RespSetSoftapVendorSpecificIe(super::CtrlMsg_Resp_SetSoftAPVendorSpecificIE),
        RespStartSoftap(super::CtrlMsg_Resp_StartSoftAP),
        RespSoftapConnectedStasList(super::CtrlMsg_Resp_SoftAPConnectedSTA),
        RespStopSoftap(super::CtrlMsg_Resp_GetStatus),
        RespSetPowerSaveMode(super::CtrlMsg_Resp_SetMode),
        RespGetPowerSaveMode(super::CtrlMsg_Resp_GetMode),
        RespOtaBegin(super::CtrlMsg_Resp_OTABegin),
        RespOtaWrite(super::CtrlMsg_Resp_OTAWrite),
        RespOtaEnd(super::CtrlMsg_Resp_OTAEnd),
        RespSetWifiMaxTxPower(super::CtrlMsg_Resp_SetWifiMaxTxPower),
        RespGetWifiCurrTxPower(super::CtrlMsg_Resp_GetWifiCurrTxPower),
        RespConfigHeartbeat(super::CtrlMsg_Resp_ConfigHeartbeat),
        RespEnableDisableFeat(super::CtrlMsg_Resp_EnableDisable),
        RespGetFwVersion(super::CtrlMsg_Resp_GetFwVersion),
        RespSetCountryCode(super::CtrlMsg_Resp_SetCountryCode),
        RespGetCountryCode(super::CtrlMsg_Resp_GetCountryCode),
        RespSetDhcpDnsStatus(super::CtrlMsg_Resp_SetDhcpDnsStatus),
        RespGetDhcpDnsStatus(super::CtrlMsg_Resp_GetDhcpDnsStatus),
        RespCustomRpcUnserialisedMsg(super::CtrlMsg_Resp_CustomRpcUnserialisedMsg),
        EventEspInit(super::CtrlMsg_Event_ESPInit),
        EventHeartbeat(super::CtrlMsg_Event_Heartbeat),
        EventStationDisconnectFromAp(super::CtrlMsg_Event_StationDisconnectFromAP),
        EventStationDisconnectFromEspSoftAp(super::CtrlMsg_Event_StationDisconnectFromESPSoftAP),
        EventStationConnectedToAp(super::CtrlMsg_Event_StationConnectedToAP),
        EventStationConnectedToEspSoftAp(super::CtrlMsg_Event_StationConnectedToESPSoftAP),
        EventSetDhcpDnsStatus(super::CtrlMsg_Event_SetDhcpDnsStatus),
        EventCustomRpcUnserialisedMsg(super::CtrlMsg_Event_CustomRpcUnserialisedMsg),
    }
}
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsg {
    pub r#msg_type: CtrlMsgType,
    pub r#msg_id: CtrlMsgId,
    pub r#uid: i32,
    pub r#req_resp_type: u32,
    pub r#payload: ::core::option::Option<CtrlMsg_::Payload>,
}
impl CtrlMsg {
    ///Return a reference to `msg_type`
    #[inline]
    pub fn r#msg_type(&self) -> &CtrlMsgType {
        &self.r#msg_type
    }
    ///Return a mutable reference to `msg_type`
    #[inline]
    pub fn mut_msg_type(&mut self) -> &mut CtrlMsgType {
        &mut self.r#msg_type
    }
    ///Set the value of `msg_type`
    #[inline]
    pub fn set_msg_type(&mut self, value: CtrlMsgType) -> &mut Self {
        self.r#msg_type = value.into();
        self
    }
    ///Builder method that sets the value of `msg_type`. Useful for initializing the message.
    #[inline]
    pub fn init_msg_type(mut self, value: CtrlMsgType) -> Self {
        self.r#msg_type = value.into();
        self
    }
    ///Return a reference to `msg_id`
    #[inline]
    pub fn r#msg_id(&self) -> &CtrlMsgId {
        &self.r#msg_id
    }
    ///Return a mutable reference to `msg_id`
    #[inline]
    pub fn mut_msg_id(&mut self) -> &mut CtrlMsgId {
        &mut self.r#msg_id
    }
    ///Set the value of `msg_id`
    #[inline]
    pub fn set_msg_id(&mut self, value: CtrlMsgId) -> &mut Self {
        self.r#msg_id = value.into();
        self
    }
    ///Builder method that sets the value of `msg_id`. Useful for initializing the message.
    #[inline]
    pub fn init_msg_id(mut self, value: CtrlMsgId) -> Self {
        self.r#msg_id = value.into();
        self
    }
    ///Return a reference to `uid`
    #[inline]
    pub fn r#uid(&self) -> &i32 {
        &self.r#uid
    }
    ///Return a mutable reference to `uid`
    #[inline]
    pub fn mut_uid(&mut self) -> &mut i32 {
        &mut self.r#uid
    }
    ///Set the value of `uid`
    #[inline]
    pub fn set_uid(&mut self, value: i32) -> &mut Self {
        self.r#uid = value.into();
        self
    }
    ///Builder method that sets the value of `uid`. Useful for initializing the message.
    #[inline]
    pub fn init_uid(mut self, value: i32) -> Self {
        self.r#uid = value.into();
        self
    }
    ///Return a reference to `req_resp_type`
    #[inline]
    pub fn r#req_resp_type(&self) -> &u32 {
        &self.r#req_resp_type
    }
    ///Return a mutable reference to `req_resp_type`
    #[inline]
    pub fn mut_req_resp_type(&mut self) -> &mut u32 {
        &mut self.r#req_resp_type
    }
    ///Set the value of `req_resp_type`
    #[inline]
    pub fn set_req_resp_type(&mut self, value: u32) -> &mut Self {
        self.r#req_resp_type = value.into();
        self
    }
    ///Builder method that sets the value of `req_resp_type`. Useful for initializing the message.
    #[inline]
    pub fn init_req_resp_type(mut self, value: u32) -> Self {
        self.r#req_resp_type = value.into();
        self
    }
}
impl ::micropb::MessageDecode for CtrlMsg {
    fn decode<IMPL_MICROPB_READ: ::micropb::PbRead>(
        &mut self,
        decoder: &mut ::micropb::PbDecoder<IMPL_MICROPB_READ>,
        len: usize,
    ) -> Result<(), ::micropb::DecodeError<IMPL_MICROPB_READ::Error>> {
        use ::micropb::{FieldDecode, PbBytes, PbMap, PbString, PbVec};
        let before = decoder.bytes_read();
        while decoder.bytes_read() - before < len {
            let tag = decoder.decode_tag()?;
            match tag.field_num() {
                0 => return Err(::micropb::DecodeError::ZeroField),
                1u32 => {
                    let mut_ref = &mut self.r#msg_type;
                    {
                        let val = decoder.decode_int32().map(|n| CtrlMsgType(n as _))?;
                        let val_ref = &val;
                        if val_ref.0 != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                2u32 => {
                    let mut_ref = &mut self.r#msg_id;
                    {
                        let val = decoder.decode_int32().map(|n| CtrlMsgId(n as _))?;
                        let val_ref = &val;
                        if val_ref.0 != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                3u32 => {
                    let mut_ref = &mut self.r#uid;
                    {
                        let val = decoder.decode_int32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                4u32 => {
                    let mut_ref = &mut self.r#req_resp_type;
                    {
                        let val = decoder.decode_varint32()?;
                        let val_ref = &val;
                        if *val_ref != 0 {
                            *mut_ref = val as _;
                        }
                    };
                }
                101u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqGetMacAddress(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqGetMacAddress(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                102u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqSetMacAddress(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqSetMacAddress(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                103u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqGetWifiMode(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqGetWifiMode(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                104u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqSetWifiMode(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqSetWifiMode(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                105u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqScanApList(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqScanApList(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                106u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqGetApConfig(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqGetApConfig(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                107u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqConnectAp(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqConnectAp(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                108u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqDisconnectAp(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqDisconnectAp(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                109u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqGetSoftapConfig(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqGetSoftapConfig(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                110u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqSetSoftapVendorSpecificIe(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqSetSoftapVendorSpecificIe(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                111u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqStartSoftap(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqStartSoftap(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                112u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqSoftapConnectedStasList(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqSoftapConnectedStasList(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                113u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqStopSoftap(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqStopSoftap(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                114u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqSetPowerSaveMode(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqSetPowerSaveMode(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                115u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqGetPowerSaveMode(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqGetPowerSaveMode(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                116u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqOtaBegin(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqOtaBegin(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                117u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqOtaWrite(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqOtaWrite(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                118u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqOtaEnd(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqOtaEnd(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                119u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqSetWifiMaxTxPower(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqSetWifiMaxTxPower(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                120u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqGetWifiCurrTxPower(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqGetWifiCurrTxPower(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                121u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqConfigHeartbeat(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqConfigHeartbeat(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                122u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqEnableDisableFeat(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqEnableDisableFeat(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                123u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqGetFwVersion(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqGetFwVersion(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                124u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqSetCountryCode(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqSetCountryCode(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                125u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqGetCountryCode(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqGetCountryCode(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                126u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqSetDhcpDnsStatus(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqSetDhcpDnsStatus(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                127u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqGetDhcpDnsStatus(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqGetDhcpDnsStatus(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                128u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::ReqCustomRpcUnserialisedMsg(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::ReqCustomRpcUnserialisedMsg(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                201u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespGetMacAddress(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespGetMacAddress(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                202u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespSetMacAddress(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespSetMacAddress(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                203u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespGetWifiMode(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespGetWifiMode(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                204u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespSetWifiMode(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespSetWifiMode(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                205u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespScanApList(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespScanApList(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                206u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespGetApConfig(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespGetApConfig(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                207u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespConnectAp(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespConnectAp(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                208u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespDisconnectAp(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespDisconnectAp(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                209u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespGetSoftapConfig(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespGetSoftapConfig(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                210u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespSetSoftapVendorSpecificIe(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(
                            CtrlMsg_::Payload::RespSetSoftapVendorSpecificIe(::core::default::Default::default()),
                        );
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                211u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespStartSoftap(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespStartSoftap(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                212u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespSoftapConnectedStasList(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespSoftapConnectedStasList(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                213u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespStopSoftap(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespStopSoftap(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                214u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespSetPowerSaveMode(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespSetPowerSaveMode(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                215u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespGetPowerSaveMode(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespGetPowerSaveMode(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                216u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespOtaBegin(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespOtaBegin(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                217u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespOtaWrite(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespOtaWrite(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                218u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespOtaEnd(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespOtaEnd(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                219u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespSetWifiMaxTxPower(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespSetWifiMaxTxPower(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                220u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespGetWifiCurrTxPower(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespGetWifiCurrTxPower(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                221u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespConfigHeartbeat(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespConfigHeartbeat(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                222u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespEnableDisableFeat(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespEnableDisableFeat(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                223u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespGetFwVersion(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespGetFwVersion(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                224u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespSetCountryCode(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespSetCountryCode(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                225u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespGetCountryCode(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespGetCountryCode(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                226u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespSetDhcpDnsStatus(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespSetDhcpDnsStatus(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                227u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespGetDhcpDnsStatus(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespGetDhcpDnsStatus(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                228u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::RespCustomRpcUnserialisedMsg(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::RespCustomRpcUnserialisedMsg(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                301u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::EventEspInit(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::EventEspInit(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                302u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::EventHeartbeat(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::EventHeartbeat(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                303u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::EventStationDisconnectFromAp(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::EventStationDisconnectFromAp(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                304u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::EventStationDisconnectFromEspSoftAp(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(
                            CtrlMsg_::Payload::EventStationDisconnectFromEspSoftAp(::core::default::Default::default()),
                        );
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                305u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::EventStationConnectedToAp(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::EventStationConnectedToAp(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                306u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::EventStationConnectedToEspSoftAp(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(
                            CtrlMsg_::Payload::EventStationConnectedToEspSoftAp(::core::default::Default::default()),
                        );
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                307u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::EventSetDhcpDnsStatus(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(CtrlMsg_::Payload::EventSetDhcpDnsStatus(
                            ::core::default::Default::default(),
                        ));
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                308u32 => {
                    let mut_ref = loop {
                        if let ::core::option::Option::Some(variant) = &mut self.r#payload {
                            if let CtrlMsg_::Payload::EventCustomRpcUnserialisedMsg(variant) = &mut *variant {
                                break &mut *variant;
                            }
                        }
                        self.r#payload = ::core::option::Option::Some(
                            CtrlMsg_::Payload::EventCustomRpcUnserialisedMsg(::core::default::Default::default()),
                        );
                    };
                    mut_ref.decode_len_delimited(decoder)?;
                }
                _ => {
                    decoder.skip_wire_value(tag.wire_type())?;
                }
            }
        }
        Ok(())
    }
}
impl ::micropb::MessageEncode for CtrlMsg {
    const MAX_SIZE: ::core::option::Option<usize> = 'msg: {
        let mut max_size = 0;
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(CtrlMsgType::_MAX_SIZE), |size| size
                + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(CtrlMsgId::_MAX_SIZE), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(10usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) =
            ::micropb::const_map!(::core::option::Option::Some(5usize), |size| size + 1usize)
        {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        if let ::core::option::Option::Some(size) = 'oneof: {
            let mut max_size = 0;
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_GetMacAddress as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_SetMacAddress as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Req_GetMode as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Req_SetMode as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Req_ScanResult as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_GetAPConfig as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Req_ConnectAP as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Req_GetStatus as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_GetSoftAPConfig as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_SetSoftAPVendorSpecificIE as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_StartSoftAP as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_SoftAPConnectedSTA as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Req_GetStatus as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Req_SetMode as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Req_GetMode as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Req_OTABegin as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Req_OTAWrite as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Req_OTAEnd as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_SetWifiMaxTxPower as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_GetWifiCurrTxPower as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_ConfigHeartbeat as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_EnableDisable as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_GetFwVersion as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_SetCountryCode as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_GetCountryCode as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_SetDhcpDnsStatus as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_GetDhcpDnsStatus as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Req_CustomRpcUnserialisedMsg as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_GetMacAddress as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_SetMacAddress as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Resp_GetMode as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Resp_SetMode as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_ScanResult as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_GetAPConfig as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Resp_ConnectAP as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Resp_GetStatus as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_GetSoftAPConfig as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_SetSoftAPVendorSpecificIE as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_StartSoftAP as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_SoftAPConnectedSTA as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Resp_GetStatus as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Resp_SetMode as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Resp_GetMode as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Resp_OTABegin as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Resp_OTAWrite as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Resp_OTAEnd as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_SetWifiMaxTxPower as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_GetWifiCurrTxPower as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_ConfigHeartbeat as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_EnableDisable as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_GetFwVersion as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_SetCountryCode as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_GetCountryCode as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_SetDhcpDnsStatus as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_GetDhcpDnsStatus as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Resp_CustomRpcUnserialisedMsg as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(<CtrlMsg_Event_ESPInit as ::micropb::MessageEncode>::MAX_SIZE, |size| {
                    ::micropb::size::sizeof_len_record(size)
                }),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Event_Heartbeat as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Event_StationDisconnectFromAP as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Event_StationDisconnectFromESPSoftAP as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Event_StationConnectedToAP as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Event_StationConnectedToESPSoftAP as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Event_SetDhcpDnsStatus as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            if let ::core::option::Option::Some(size) = ::micropb::const_map!(
                ::micropb::const_map!(
                    <CtrlMsg_Event_CustomRpcUnserialisedMsg as ::micropb::MessageEncode>::MAX_SIZE,
                    |size| ::micropb::size::sizeof_len_record(size)
                ),
                |size| size + 2usize
            ) {
                if size > max_size {
                    max_size = size;
                }
            } else {
                break 'oneof (::core::option::Option::<usize>::None);
            }
            ::core::option::Option::Some(max_size)
        } {
            max_size += size;
        } else {
            break 'msg (::core::option::Option::<usize>::None);
        };
        ::core::option::Option::Some(max_size)
    };
    fn encode<IMPL_MICROPB_WRITE: ::micropb::PbWrite>(
        &self,
        encoder: &mut ::micropb::PbEncoder<IMPL_MICROPB_WRITE>,
    ) -> Result<(), IMPL_MICROPB_WRITE::Error> {
        use ::micropb::{FieldEncode, PbMap};
        {
            let val_ref = &self.r#msg_type;
            if val_ref.0 != 0 {
                encoder.encode_varint32(8u32)?;
                encoder.encode_int32(val_ref.0 as _)?;
            }
        }
        {
            let val_ref = &self.r#msg_id;
            if val_ref.0 != 0 {
                encoder.encode_varint32(16u32)?;
                encoder.encode_int32(val_ref.0 as _)?;
            }
        }
        {
            let val_ref = &self.r#uid;
            if *val_ref != 0 {
                encoder.encode_varint32(24u32)?;
                encoder.encode_int32(*val_ref as _)?;
            }
        }
        {
            let val_ref = &self.r#req_resp_type;
            if *val_ref != 0 {
                encoder.encode_varint32(32u32)?;
                encoder.encode_varint32(*val_ref as _)?;
            }
        }
        if let Some(oneof) = &self.r#payload {
            match &*oneof {
                CtrlMsg_::Payload::ReqGetMacAddress(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(810u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqSetMacAddress(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(818u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqGetWifiMode(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(826u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqSetWifiMode(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(834u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqScanApList(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(842u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqGetApConfig(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(850u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqConnectAp(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(858u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqDisconnectAp(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(866u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqGetSoftapConfig(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(874u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqSetSoftapVendorSpecificIe(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(882u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqStartSoftap(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(890u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqSoftapConnectedStasList(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(898u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqStopSoftap(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(906u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqSetPowerSaveMode(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(914u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqGetPowerSaveMode(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(922u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqOtaBegin(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(930u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqOtaWrite(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(938u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqOtaEnd(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(946u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqSetWifiMaxTxPower(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(954u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqGetWifiCurrTxPower(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(962u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqConfigHeartbeat(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(970u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqEnableDisableFeat(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(978u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqGetFwVersion(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(986u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqSetCountryCode(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(994u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqGetCountryCode(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1002u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqSetDhcpDnsStatus(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1010u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqGetDhcpDnsStatus(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1018u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::ReqCustomRpcUnserialisedMsg(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1026u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespGetMacAddress(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1610u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespSetMacAddress(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1618u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespGetWifiMode(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1626u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespSetWifiMode(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1634u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespScanApList(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1642u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespGetApConfig(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1650u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespConnectAp(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1658u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespDisconnectAp(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1666u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespGetSoftapConfig(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1674u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespSetSoftapVendorSpecificIe(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1682u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespStartSoftap(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1690u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespSoftapConnectedStasList(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1698u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespStopSoftap(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1706u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespSetPowerSaveMode(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1714u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespGetPowerSaveMode(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1722u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespOtaBegin(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1730u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespOtaWrite(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1738u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespOtaEnd(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1746u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespSetWifiMaxTxPower(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1754u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespGetWifiCurrTxPower(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1762u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespConfigHeartbeat(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1770u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespEnableDisableFeat(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1778u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespGetFwVersion(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1786u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespSetCountryCode(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1794u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespGetCountryCode(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1802u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespSetDhcpDnsStatus(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1810u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespGetDhcpDnsStatus(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1818u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::RespCustomRpcUnserialisedMsg(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(1826u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::EventEspInit(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(2410u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::EventHeartbeat(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(2418u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::EventStationDisconnectFromAp(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(2426u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::EventStationDisconnectFromEspSoftAp(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(2434u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::EventStationConnectedToAp(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(2442u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::EventStationConnectedToEspSoftAp(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(2450u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::EventSetDhcpDnsStatus(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(2458u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
                CtrlMsg_::Payload::EventCustomRpcUnserialisedMsg(val_ref) => {
                    let val_ref = &*val_ref;
                    encoder.encode_varint32(2466u32)?;
                    val_ref.encode_len_delimited(encoder)?;
                }
            }
        }
        Ok(())
    }
    fn compute_size(&self) -> usize {
        use ::micropb::{FieldEncode, PbMap};
        let mut size = 0;
        {
            let val_ref = &self.r#msg_type;
            if val_ref.0 != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(val_ref.0 as _);
            }
        }
        {
            let val_ref = &self.r#msg_id;
            if val_ref.0 != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(val_ref.0 as _);
            }
        }
        {
            let val_ref = &self.r#uid;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_int32(*val_ref as _);
            }
        }
        {
            let val_ref = &self.r#req_resp_type;
            if *val_ref != 0 {
                size += 1usize + ::micropb::size::sizeof_varint32(*val_ref as _);
            }
        }
        if let Some(oneof) = &self.r#payload {
            match &*oneof {
                CtrlMsg_::Payload::ReqGetMacAddress(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqSetMacAddress(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqGetWifiMode(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqSetWifiMode(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqScanApList(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqGetApConfig(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqConnectAp(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqDisconnectAp(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqGetSoftapConfig(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqSetSoftapVendorSpecificIe(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqStartSoftap(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqSoftapConnectedStasList(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqStopSoftap(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqSetPowerSaveMode(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqGetPowerSaveMode(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqOtaBegin(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqOtaWrite(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqOtaEnd(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqSetWifiMaxTxPower(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqGetWifiCurrTxPower(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqConfigHeartbeat(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqEnableDisableFeat(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqGetFwVersion(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqSetCountryCode(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqGetCountryCode(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqSetDhcpDnsStatus(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqGetDhcpDnsStatus(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::ReqCustomRpcUnserialisedMsg(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespGetMacAddress(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespSetMacAddress(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespGetWifiMode(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespSetWifiMode(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespScanApList(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespGetApConfig(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespConnectAp(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespDisconnectAp(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespGetSoftapConfig(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespSetSoftapVendorSpecificIe(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespStartSoftap(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespSoftapConnectedStasList(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespStopSoftap(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespSetPowerSaveMode(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespGetPowerSaveMode(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespOtaBegin(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespOtaWrite(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespOtaEnd(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespSetWifiMaxTxPower(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespGetWifiCurrTxPower(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespConfigHeartbeat(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespEnableDisableFeat(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespGetFwVersion(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespSetCountryCode(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespGetCountryCode(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespSetDhcpDnsStatus(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespGetDhcpDnsStatus(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::RespCustomRpcUnserialisedMsg(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::EventEspInit(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::EventHeartbeat(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::EventStationDisconnectFromAp(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::EventStationDisconnectFromEspSoftAp(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::EventStationConnectedToAp(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::EventStationConnectedToEspSoftAp(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::EventSetDhcpDnsStatus(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
                CtrlMsg_::Payload::EventCustomRpcUnserialisedMsg(val_ref) => {
                    let val_ref = &*val_ref;
                    size += 2usize + ::micropb::size::sizeof_len_record(val_ref.compute_size());
                }
            }
        }
        size
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ctrl_VendorIEType(pub i32);
impl Ctrl_VendorIEType {
    pub const _MAX_SIZE: usize = 10usize;
    pub const Beacon: Self = Self(0);
    pub const ProbeReq: Self = Self(1);
    pub const ProbeResp: Self = Self(2);
    pub const AssocReq: Self = Self(3);
    pub const AssocResp: Self = Self(4);
}
impl core::default::Default for Ctrl_VendorIEType {
    fn default() -> Self {
        Self(0)
    }
}
impl core::convert::From<i32> for Ctrl_VendorIEType {
    fn from(val: i32) -> Self {
        Self(val)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ctrl_VendorIEID(pub i32);
impl Ctrl_VendorIEID {
    pub const _MAX_SIZE: usize = 10usize;
    pub const Id0: Self = Self(0);
    pub const Id1: Self = Self(1);
}
impl core::default::Default for Ctrl_VendorIEID {
    fn default() -> Self {
        Self(0)
    }
}
impl core::convert::From<i32> for Ctrl_VendorIEID {
    fn from(val: i32) -> Self {
        Self(val)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ctrl_WifiMode(pub i32);
impl Ctrl_WifiMode {
    pub const _MAX_SIZE: usize = 10usize;
    pub const None: Self = Self(0);
    pub const Sta: Self = Self(1);
    pub const Ap: Self = Self(2);
    pub const Apsta: Self = Self(3);
}
impl core::default::Default for Ctrl_WifiMode {
    fn default() -> Self {
        Self(0)
    }
}
impl core::convert::From<i32> for Ctrl_WifiMode {
    fn from(val: i32) -> Self {
        Self(val)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ctrl_WifiBw(pub i32);
impl Ctrl_WifiBw {
    pub const _MAX_SIZE: usize = 10usize;
    pub const BwInvalid: Self = Self(0);
    pub const Ht20: Self = Self(1);
    pub const Ht40: Self = Self(2);
}
impl core::default::Default for Ctrl_WifiBw {
    fn default() -> Self {
        Self(0)
    }
}
impl core::convert::From<i32> for Ctrl_WifiBw {
    fn from(val: i32) -> Self {
        Self(val)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ctrl_WifiPowerSave(pub i32);
impl Ctrl_WifiPowerSave {
    pub const _MAX_SIZE: usize = 10usize;
    pub const NoPs: Self = Self(0);
    pub const MinModem: Self = Self(1);
    pub const MaxModem: Self = Self(2);
    pub const PsInvalid: Self = Self(3);
}
impl core::default::Default for Ctrl_WifiPowerSave {
    fn default() -> Self {
        Self(0)
    }
}
impl core::convert::From<i32> for Ctrl_WifiPowerSave {
    fn from(val: i32) -> Self {
        Self(val)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ctrl_WifiSecProt(pub i32);
impl Ctrl_WifiSecProt {
    pub const _MAX_SIZE: usize = 10usize;
    pub const Open: Self = Self(0);
    pub const Wep: Self = Self(1);
    pub const WpaPsk: Self = Self(2);
    pub const Wpa2Psk: Self = Self(3);
    pub const WpaWpa2Psk: Self = Self(4);
    pub const Wpa2Enterprise: Self = Self(5);
    pub const Wpa3Psk: Self = Self(6);
    pub const Wpa2Wpa3Psk: Self = Self(7);
}
impl core::default::Default for Ctrl_WifiSecProt {
    fn default() -> Self {
        Self(0)
    }
}
impl core::convert::From<i32> for Ctrl_WifiSecProt {
    fn from(val: i32) -> Self {
        Self(val)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Ctrl_Status(pub i32);
impl Ctrl_Status {
    pub const _MAX_SIZE: usize = 10usize;
    pub const Connected: Self = Self(0);
    pub const NotConnected: Self = Self(1);
    pub const NoApFound: Self = Self(2);
    pub const ConnectionFail: Self = Self(3);
    pub const InvalidArgument: Self = Self(4);
    pub const OutOfRange: Self = Self(5);
}
impl core::default::Default for Ctrl_Status {
    fn default() -> Self {
        Self(0)
    }
}
impl core::convert::From<i32> for Ctrl_Status {
    fn from(val: i32) -> Self {
        Self(val)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgType(pub i32);
impl CtrlMsgType {
    pub const _MAX_SIZE: usize = 10usize;
    pub const MsgTypeInvalid: Self = Self(0);
    pub const Req: Self = Self(1);
    pub const Resp: Self = Self(2);
    pub const Event: Self = Self(3);
    pub const MsgTypeMax: Self = Self(4);
}
impl core::default::Default for CtrlMsgType {
    fn default() -> Self {
        Self(0)
    }
}
impl core::convert::From<i32> for CtrlMsgType {
    fn from(val: i32) -> Self {
        Self(val)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CtrlMsgId(pub i32);
impl CtrlMsgId {
    pub const _MAX_SIZE: usize = 10usize;
    pub const MsgIdInvalid: Self = Self(0);
    pub const ReqBase: Self = Self(100);
    pub const ReqGetMacAddress: Self = Self(101);
    pub const ReqSetMacAddress: Self = Self(102);
    pub const ReqGetWifiMode: Self = Self(103);
    pub const ReqSetWifiMode: Self = Self(104);
    pub const ReqGetApScanList: Self = Self(105);
    pub const ReqGetApConfig: Self = Self(106);
    pub const ReqConnectAp: Self = Self(107);
    pub const ReqDisconnectAp: Self = Self(108);
    pub const ReqGetSoftApConfig: Self = Self(109);
    pub const ReqSetSoftApVendorSpecificIe: Self = Self(110);
    pub const ReqStartSoftAp: Self = Self(111);
    pub const ReqGetSoftApConnectedStaList: Self = Self(112);
    pub const ReqStopSoftAp: Self = Self(113);
    pub const ReqSetPowerSaveMode: Self = Self(114);
    pub const ReqGetPowerSaveMode: Self = Self(115);
    pub const ReqOtaBegin: Self = Self(116);
    pub const ReqOtaWrite: Self = Self(117);
    pub const ReqOtaEnd: Self = Self(118);
    pub const ReqSetWifiMaxTxPower: Self = Self(119);
    pub const ReqGetWifiCurrTxPower: Self = Self(120);
    pub const ReqConfigHeartbeat: Self = Self(121);
    pub const ReqEnableDisable: Self = Self(122);
    pub const ReqGetFwVersion: Self = Self(123);
    pub const ReqSetCountryCode: Self = Self(124);
    pub const ReqGetCountryCode: Self = Self(125);
    pub const ReqSetDhcpDnsStatus: Self = Self(126);
    pub const ReqGetDhcpDnsStatus: Self = Self(127);
    pub const ReqCustomRpcUnserialisedMsg: Self = Self(128);
    pub const ReqMax: Self = Self(129);
    pub const RespBase: Self = Self(200);
    pub const RespGetMacAddress: Self = Self(201);
    pub const RespSetMacAddress: Self = Self(202);
    pub const RespGetWifiMode: Self = Self(203);
    pub const RespSetWifiMode: Self = Self(204);
    pub const RespGetApScanList: Self = Self(205);
    pub const RespGetApConfig: Self = Self(206);
    pub const RespConnectAp: Self = Self(207);
    pub const RespDisconnectAp: Self = Self(208);
    pub const RespGetSoftApConfig: Self = Self(209);
    pub const RespSetSoftApVendorSpecificIe: Self = Self(210);
    pub const RespStartSoftAp: Self = Self(211);
    pub const RespGetSoftApConnectedStaList: Self = Self(212);
    pub const RespStopSoftAp: Self = Self(213);
    pub const RespSetPowerSaveMode: Self = Self(214);
    pub const RespGetPowerSaveMode: Self = Self(215);
    pub const RespOtaBegin: Self = Self(216);
    pub const RespOtaWrite: Self = Self(217);
    pub const RespOtaEnd: Self = Self(218);
    pub const RespSetWifiMaxTxPower: Self = Self(219);
    pub const RespGetWifiCurrTxPower: Self = Self(220);
    pub const RespConfigHeartbeat: Self = Self(221);
    pub const RespEnableDisable: Self = Self(222);
    pub const RespGetFwVersion: Self = Self(223);
    pub const RespSetCountryCode: Self = Self(224);
    pub const RespGetCountryCode: Self = Self(225);
    pub const RespSetDhcpDnsStatus: Self = Self(226);
    pub const RespGetDhcpDnsStatus: Self = Self(227);
    pub const RespCustomRpcUnserialisedMsg: Self = Self(228);
    pub const RespMax: Self = Self(229);
    pub const EventBase: Self = Self(300);
    pub const EventEspInit: Self = Self(301);
    pub const EventHeartbeat: Self = Self(302);
    pub const EventStationDisconnectFromAp: Self = Self(303);
    pub const EventStationDisconnectFromEspSoftAp: Self = Self(304);
    pub const EventStationConnectedToAp: Self = Self(305);
    pub const EventStationConnectedToEspSoftAp: Self = Self(306);
    pub const EventSetDhcpDnsStatus: Self = Self(307);
    pub const EventCustomRpcUnserialisedMsg: Self = Self(308);
    pub const EventMax: Self = Self(309);
}
impl core::default::Default for CtrlMsgId {
    fn default() -> Self {
        Self(0)
    }
}
impl core::convert::From<i32> for CtrlMsgId {
    fn from(val: i32) -> Self {
        Self(val)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HostedFeature(pub i32);
impl HostedFeature {
    pub const _MAX_SIZE: usize = 10usize;
    pub const HostedInvalidFeature: Self = Self(0);
    pub const HostedWifi: Self = Self(1);
    pub const HostedBluetooth: Self = Self(2);
    pub const HostedIsNetworkSplitOn: Self = Self(3);
}
impl core::default::Default for HostedFeature {
    fn default() -> Self {
        Self(0)
    }
}
impl core::convert::From<i32> for HostedFeature {
    fn from(val: i32) -> Self {
        Self(val)
    }
}
