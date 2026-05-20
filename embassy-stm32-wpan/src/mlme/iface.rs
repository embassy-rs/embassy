//! Trait-based API for the chips to implement

use embedded_io::ErrorType;

use crate::mlme::indications::{
    AssociateIndication, BeaconNotifyIndication, CommStatusIndication, DataIndication, DisassociateIndication,
    DpsIndication, GtsIndication, OrphanIndication, PollIndication, SyncLossIndication,
};
use crate::mlme::responses::{
    AssociateConfirm, CalibrateConfirm, DataConfirm, DisassociateConfirm, DpsConfirm, GetConfirm, GtsConfirm,
    PollConfirm, PurgeConfirm, ResetConfirm, RxEnableConfirm, ScanConfirm, SetConfirm, SoundingConfirm, StartConfirm,
};

/// Adapter trait for serializing HCI types to embedded-io implementations.
pub trait WriteHci {
    /// The number of bytes this value will write
    fn size(&self) -> usize;

    /// Write this value to the provided writer.
    fn write_hci<W: embedded_io::Write>(&self, writer: W) -> Result<(), W::Error>;
}

pub mod mlme {

    #[derive(Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum RequestPacketKind {
        Associate,
        Dissassociate,
        Get,
        Gts,
        Reset,
        RxEnable,
        Scan,
        Set,
        Start,
        Sync,
        Poll,
        Dps,
        Sounding,
        Calibrate,
    }

    #[derive(Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum ResponsePacketKind {
        Associate,
        Orphan,
    }

    #[derive(Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum PacketKind {
        Request(RequestPacketKind),
        Response(ResponsePacketKind),
    }
}

pub mod mcps {
    #[derive(Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum PacketKind {
        Data,
        Purge,
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PacketKind {
    Mlme(mlme::PacketKind),
    Mcps(mcps::PacketKind),
}

/// Trait representing a HCI packet.
pub trait HostToControllerPacket: WriteHci {
    /// Packet kind associated with this HCI packet.
    const KIND: PacketKind;
}

/// Errors from parsing HCI data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FromHciBytesError {
    /// Size of input did not match valid size.
    InvalidSize,
    /// Value of input did not match valid values.
    InvalidValue,
}

/// A fixed size HCI type that can be deserialized from bytes.
pub trait FromHciBytes<'de>: Sized {
    /// Deserialize bytes into a HCI type
    fn from_hci_bytes(data: &'de [u8]) -> Result<Self, FromHciBytesError>;
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum IndicationPacket<'a> {
    MlmeAssociate(&'a AssociateIndication),
    MlmeDisassociate(&'a DisassociateIndication),
    MlmeBeaconNotify(&'a BeaconNotifyIndication),
    MlmeCommStatus(&'a CommStatusIndication),
    MlmeGts(&'a GtsIndication),
    MlmeOrphan(&'a OrphanIndication),
    MlmeSyncLoss(&'a SyncLossIndication),
    MlmeDps(&'a DpsIndication),
    McpsData(&'a DataIndication),
    MlmePoll(&'a PollIndication),
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConfirmPacket<'a> {
    MlmeAssociate(&'a AssociateConfirm),
    MlmeDisassociate(&'a DisassociateConfirm),
    MlmeGet(&'a GetConfirm),
    MlmeGts(&'a GtsConfirm),
    MlmeReset(&'a ResetConfirm),
    MlmeRxEnable(&'a RxEnableConfirm),
    MlmeScan(&'a ScanConfirm),
    MlmeSet(&'a SetConfirm),
    MlmeStart(&'a StartConfirm),
    MlmePoll(&'a PollConfirm),
    MlmeDps(&'a DpsConfirm),
    MlmeSounding(&'a SoundingConfirm),
    MlmeCalibrate(&'a CalibrateConfirm),
    McpsData(&'a DataConfirm),
    McpsPurge(&'a PurgeConfirm),
}

/// Type representing valid deserialized HCI packets.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ControllerToHostPacket<'a> {
    /// ACL packet.
    Indication(IndicationPacket<'a>),
    /// Sync packet.
    Confirm(ConfirmPacket<'a>),
}

pub trait ControllerToHostPacketBox<'a> {
    fn packet<'b>(&'b self) -> ControllerToHostPacket<'b>
    where
        'a: 'b;
}

impl<'a> ControllerToHostPacketBox<'a> for ControllerToHostPacket<'a> {
    fn packet<'b>(&'b self) -> ControllerToHostPacket<'b>
    where
        'a: 'b,
    {
        *self
    }
}

/// Trait representing a HCI controller which supports async operations.
pub trait Controller: ErrorType {
    /// Write a packet to the controller.
    fn write(&self, packet: &impl HostToControllerPacket) -> impl Future<Output = Result<(), Self::Error>>;

    /// Read a valid packet from the controller.
    fn read<'a>(
        &self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<impl ControllerToHostPacketBox<'a>, Self::Error>>;
}
