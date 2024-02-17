// Note: This file is copied and modified from fdcan crate by Richard Meadows
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

use super::enums::{
    BitRateSwitching, ErrorStateIndicator, FilterElementConfig, FilterType, FrameFormat, IdType,
    RemoteTransmissionRequest,
};
use super::generic;

#[doc = "Reader of field `ID`"]
pub type ID_R = generic::R<u32, u32>;

#[doc = "Reader of field `RTR`"]
pub type RTR_R = generic::R<bool, RemoteTransmissionRequest>;
impl RTR_R {
    pub fn rtr(&self) -> RemoteTransmissionRequest {
        match self.bits {
            false => RemoteTransmissionRequest::TransmitDataFrame,
            true => RemoteTransmissionRequest::TransmitRemoteFrame,
        }
    }
    pub fn is_transmit_remote_frame(&self) -> bool {
        *self == RemoteTransmissionRequest::TransmitRemoteFrame
    }
    pub fn is_transmit_data_frame(&self) -> bool {
        *self == RemoteTransmissionRequest::TransmitDataFrame
    }
}

#[doc = "Reader of field `XTD`"]
pub type XTD_R = generic::R<bool, IdType>;
impl XTD_R {
    pub fn id_type(&self) -> IdType {
        match self.bits() {
            false => IdType::StandardId,
            true => IdType::ExtendedId,
        }
    }
    pub fn is_standard_id(&self) -> bool {
        *self == IdType::StandardId
    }
    pub fn is_exteded_id(&self) -> bool {
        *self == IdType::ExtendedId
    }
}

#[doc = "Reader of field `ESI`"]
pub type ESI_R = generic::R<bool, ErrorStateIndicator>;
impl ESI_R {
    pub fn error_state(&self) -> ErrorStateIndicator {
        match self.bits() {
            false => ErrorStateIndicator::ErrorActive,
            true => ErrorStateIndicator::ErrorPassive,
        }
    }
    pub fn is_error_active(&self) -> bool {
        *self == ErrorStateIndicator::ErrorActive
    }
    pub fn is_error_passive(&self) -> bool {
        *self == ErrorStateIndicator::ErrorPassive
    }
}

#[doc = "Reader of field `DLC`"]
pub type DLC_R = generic::R<u8, u8>;

#[doc = "Reader of field `BRS`"]
pub type BRS_R = generic::R<bool, BitRateSwitching>;
impl BRS_R {
    pub fn bit_rate_switching(&self) -> BitRateSwitching {
        match self.bits() {
            true => BitRateSwitching::WithBRS,
            false => BitRateSwitching::WithoutBRS,
        }
    }
    pub fn is_with_brs(&self) -> bool {
        *self == BitRateSwitching::WithBRS
    }
    pub fn is_without_brs(&self) -> bool {
        *self == BitRateSwitching::WithoutBRS
    }
}

#[doc = "Reader of field `FDF`"]
pub type FDF_R = generic::R<bool, FrameFormat>;
impl FDF_R {
    pub fn frame_format(&self) -> FrameFormat {
        match self.bits() {
            false => FrameFormat::Classic,
            true => FrameFormat::Fdcan,
        }
    }
    pub fn is_classic_format(&self) -> bool {
        *self == FrameFormat::Classic
    }
    pub fn is_fdcan_format(&self) -> bool {
        *self == FrameFormat::Fdcan
    }
}

#[doc = "Reader of field `(X|S)FT`"]
pub type ESFT_R = generic::R<u8, FilterType>;
impl ESFT_R {
    #[doc = r"Gets the Filtertype"]
    #[inline(always)]
    pub fn to_filter_type(&self) -> FilterType {
        match self.bits() {
            0b00 => FilterType::RangeFilter,
            0b01 => FilterType::DualIdFilter,
            0b10 => FilterType::ClassicFilter,
            0b11 => FilterType::FilterDisabled,
            _ => unreachable!(),
        }
    }
}

#[doc = "Reader of field `(E|S)FEC`"]
pub type ESFEC_R = generic::R<u8, FilterElementConfig>;
impl ESFEC_R {
    pub fn to_filter_element_config(&self) -> FilterElementConfig {
        match self.bits() {
            0b000 => FilterElementConfig::DisableFilterElement,
            0b001 => FilterElementConfig::StoreInFifo0,
            0b010 => FilterElementConfig::StoreInFifo1,
            0b011 => FilterElementConfig::Reject,
            0b100 => FilterElementConfig::SetPriority,
            0b101 => FilterElementConfig::SetPriorityAndStoreInFifo0,
            0b110 => FilterElementConfig::SetPriorityAndStoreInFifo1,
            _ => unimplemented!(),
        }
    }
}
