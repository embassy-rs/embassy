//! Trait-based API for the chips to implement

use embedded_io::{ErrorKind, ErrorType};

/// Adapter trait for serializing HCI types to embedded-io implementations.
pub trait WriteHci {
    /// The number of bytes this value will write
    fn size(&self) -> usize;

    /// Write this value to the provided writer.
    fn write_hci<W: embedded_io::Write>(&self, writer: W) -> Result<(), W::Error>;
}

pub mod mlme {
    use crate::net::indications::{
        AssociateIndication, BeaconNotifyIndication, CommStatusIndication, DisassociateIndication, DpsIndication,
        GtsIndication, OrphanIndication, PollIndication, SyncLossIndication,
    };
    use crate::net::responses::{
        AssociateConfirm, CalibrateConfirm, DisassociateConfirm, DpsConfirm, GetConfirm, GtsConfirm, PollConfirm,
        ResetConfirm, RxEnableConfirm, ScanConfirm, SetConfirm, SoundingConfirm, StartConfirm,
    };

    #[derive(Clone, Copy, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum IndicationPacket<'a> {
        Associate(&'a AssociateIndication),
        Disassociate(&'a DisassociateIndication),
        BeaconNotify(&'a BeaconNotifyIndication),
        CommStatus(&'a CommStatusIndication),
        Gts(&'a GtsIndication),
        Orphan(&'a OrphanIndication),
        SyncLoss(&'a SyncLossIndication),
        Dps(&'a DpsIndication),
        Poll(&'a PollIndication),
    }

    #[derive(Clone, Copy, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum ConfirmPacket<'a> {
        Associate(&'a AssociateConfirm),
        Disassociate(&'a DisassociateConfirm),
        Get(&'a GetConfirm),
        Gts(&'a GtsConfirm),
        Reset(&'a ResetConfirm),
        RxEnable(&'a RxEnableConfirm),
        Scan(&'a ScanConfirm),
        Set(&'a SetConfirm),
        Start(&'a StartConfirm),
        Poll(&'a PollConfirm),
        Dps(&'a DpsConfirm),
        Sounding(&'a SoundingConfirm),
        Calibrate(&'a CalibrateConfirm),
    }

    /// Type representing valid deserialized HCI packets.
    #[derive(Clone, Copy, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum Packet<'a> {
        /// ACL packet.
        Indication(IndicationPacket<'a>),
        /// Sync packet.
        Confirm(ConfirmPacket<'a>),
    }

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
    use crate::net::indications::DataIndication;
    use crate::net::responses::{DataConfirm, PurgeConfirm};

    #[derive(Clone, Copy, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum IndicationPacket<'a> {
        Data(&'a DataIndication),
    }

    #[derive(Clone, Copy, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum ConfirmPacket<'a> {
        Data(&'a DataConfirm),
        Purge(&'a PurgeConfirm),
    }

    /// Type representing valid deserialized HCI packets.
    #[derive(Clone, Copy, Debug)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum Packet<'a> {
        /// ACL packet.
        Indication(IndicationPacket<'a>),
        /// Sync packet.
        Confirm(ConfirmPacket<'a>),
    }

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

    fn kind(&self) -> PacketKind {
        Self::KIND
    }
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

impl Into<ErrorKind> for FromHciBytesError {
    fn into(self) -> ErrorKind {
        match self {
            Self::InvalidSize => ErrorKind::InvalidInput,
            Self::InvalidValue => ErrorKind::InvalidData,
        }
    }
}

/// A fixed size HCI type that can be deserialized from bytes.
pub trait FromHciBytes<'de>: Sized {
    /// Deserialize bytes into a HCI type
    fn from_hci_bytes(data: &'de [u8]) -> Result<&'de Self, FromHciBytesError>;
}

/// Type representing valid deserialized HCI packets.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ControllerToHostPacket<'a> {
    /// ACL packet.
    Mlme(mlme::Packet<'a>),
    /// Sync packet.
    Mcps(mcps::Packet<'a>),
}

pub trait ControllerToHostPacketBox {
    fn packet<'b>(&'b self) -> ControllerToHostPacket<'b>;
}

impl<'d> ControllerToHostPacketBox for ControllerToHostPacket<'d> {
    fn packet<'b>(&'b self) -> ControllerToHostPacket<'b> {
        *self
    }
}

/// Trait representing a HCI controller which supports async operations.
pub trait Controller: ErrorType {
    type Packet: ControllerToHostPacketBox;

    /// Write a packet to the controller.
    fn write(&self, packet: &impl HostToControllerPacket) -> impl Future<Output = Result<(), Self::Error>>;

    /// Read a valid packet from the controller.
    fn read<'a>(&self) -> impl Future<Output = Result<Self::Packet, Self::Error>>;
}
