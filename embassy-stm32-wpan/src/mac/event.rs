use core::cell::RefCell;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::pubsub::{PubSubChannel, Subscriber};

use super::helpers::to_u16;
use super::indications::{
    AssociateIndication, BeaconNotifyIndication, CommStatusIndication, DataIndication, DisassociateIndication,
    DpsIndication, GtsIndication, OrphanIndication, PollIndication, SyncLossIndication,
};
use super::responses::{
    AssociateConfirm, CalibrateConfirm, DataConfirm, DisassociateConfirm, DpsConfirm, GetConfirm, GtsConfirm,
    PollConfirm, PurgeConfirm, ResetConfirm, RxEnableConfirm, ScanConfirm, SetConfirm, SoundingConfirm, StartConfirm,
};
use crate::mac::opcodes::OpcodeM0ToM4;

pub trait ParseableMacEvent {
    const SIZE: usize;

    fn validate(buf: &[u8]) -> Result<(), ()> {
        if buf.len() < Self::SIZE {
            return Err(());
        }

        Ok(())
    }

    fn try_parse(buf: &[u8]) -> Result<Self, ()>
    where
        Self: Sized;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MacEvent {
    MlmeAssociateCnf(AssociateConfirm),
    MlmeDisassociateCnf(DisassociateConfirm),
    MlmeGetCnf(GetConfirm),
    MlmeGtsCnf(GtsConfirm),
    MlmeResetCnf(ResetConfirm),
    MlmeRxEnableCnf(RxEnableConfirm),
    MlmeScanCnf(ScanConfirm),
    MlmeSetCnf(SetConfirm),
    MlmeStartCnf(StartConfirm),
    MlmePollCnf(PollConfirm),
    MlmeDpsCnf(DpsConfirm),
    MlmeSoundingCnf(SoundingConfirm),
    MlmeCalibrateCnf(CalibrateConfirm),
    McpsDataCnf(DataConfirm),
    McpsPurgeCnf(PurgeConfirm),
    MlmeAssociateInd(AssociateIndication),
    MlmeDisassociateInd(DisassociateIndication),
    MlmeBeaconNotifyInd(BeaconNotifyIndication),
    MlmeCommStatusInd(CommStatusIndication),
    MlmeGtsInd(GtsIndication),
    MlmeOrphanInd(OrphanIndication),
    MlmeSyncLossInd(SyncLossIndication),
    MlmeDpsInd(DpsIndication),
    McpsDataInd(DataIndication),
    MlmePollInd(PollIndication),
}

impl TryFrom<&[u8]> for MacEvent {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let opcode = to_u16(&value[0..2]);
        let opcode = OpcodeM0ToM4::try_from(opcode)?;

        let buf = &value[2..];

        match opcode {
            OpcodeM0ToM4::MlmeAssociateCnf => Ok(Self::MlmeAssociateCnf(AssociateConfirm::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeDisassociateCnf => Ok(Self::MlmeDisassociateCnf(DisassociateConfirm::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeGetCnf => Ok(Self::MlmeGetCnf(GetConfirm::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeGtsCnf => Ok(Self::MlmeGtsCnf(GtsConfirm::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeResetCnf => Ok(Self::MlmeResetCnf(ResetConfirm::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeRxEnableCnf => Ok(Self::MlmeRxEnableCnf(RxEnableConfirm::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeScanCnf => Ok(Self::MlmeScanCnf(ScanConfirm::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeSetCnf => Ok(Self::MlmeSetCnf(SetConfirm::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeStartCnf => Ok(Self::MlmeStartCnf(StartConfirm::try_parse(buf)?)),
            OpcodeM0ToM4::MlmePollCnf => Ok(Self::MlmePollCnf(PollConfirm::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeDpsCnf => Ok(Self::MlmeDpsCnf(DpsConfirm::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeSoundingCnf => Ok(Self::MlmeSoundingCnf(SoundingConfirm::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeCalibrateCnf => Ok(Self::MlmeCalibrateCnf(CalibrateConfirm::try_parse(buf)?)),
            OpcodeM0ToM4::McpsDataCnf => Ok(Self::McpsDataCnf(DataConfirm::try_parse(buf)?)),
            OpcodeM0ToM4::McpsPurgeCnf => Ok(Self::McpsPurgeCnf(PurgeConfirm::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeAssociateInd => Ok(Self::MlmeAssociateInd(AssociateIndication::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeDisassociateInd => Ok(Self::MlmeDisassociateInd(DisassociateIndication::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeBeaconNotifyInd => Ok(Self::MlmeBeaconNotifyInd(BeaconNotifyIndication::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeCommStatusInd => Ok(Self::MlmeCommStatusInd(CommStatusIndication::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeGtsInd => Ok(Self::MlmeGtsInd(GtsIndication::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeOrphanInd => Ok(Self::MlmeOrphanInd(OrphanIndication::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeSyncLossInd => Ok(Self::MlmeSyncLossInd(SyncLossIndication::try_parse(buf)?)),
            OpcodeM0ToM4::MlmeDpsInd => Ok(Self::MlmeDpsInd(DpsIndication::try_parse(buf)?)),
            OpcodeM0ToM4::McpsDataInd => Ok(Self::McpsDataInd(DataIndication::try_parse(buf)?)),
            OpcodeM0ToM4::MlmePollInd => Ok(Self::MlmePollInd(PollIndication::try_parse(buf)?)),
        }
    }
}

// TODO this PubSub can probably be replaced with shared memory to make it a bit more efficient.
pub type EventQueue = PubSubChannel<NoopRawMutex, Message, 2, 1, 1>;
pub type EventSubscriber<'a> = Subscriber<'a, NoopRawMutex, Message, 2, 1, 1>;

pub struct Events {
    pub queue: EventQueue,
    pub mask: SharedEventMask,
}

impl Events {
    pub fn new() -> Self {
        Self {
            queue: EventQueue::new(),
            mask: SharedEventMask::default(),
        }
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Status {
    pub event_type: MacEvent,
    pub status: u32,
}

#[derive(Clone, Copy)]
pub enum Payload {
    None,
    //    BssInfo(BssInfo),
}

#[derive(Clone)]

pub struct Message {
    pub header: Status,
    pub payload: Payload,
}

impl Message {
    pub fn new(status: Status, payload: Payload) -> Self {
        Self {
            header: status,
            payload,
        }
    }
}

#[derive(Default)]
struct EventMask {
    mask: [u32; Self::WORD_COUNT],
}

impl EventMask {
    const WORD_COUNT: usize = ((Event::LAST as u32 + (u32::BITS - 1)) / u32::BITS) as usize;

    fn enable(&mut self, event: MacEvent) {
        let n = event as u32;
        let word = n / u32::BITS;
        let bit = n % u32::BITS;

        self.mask[word as usize] |= 1 << bit;
    }

    fn disable(&mut self, event: MacEvent) {
        let n = event as u32;
        let word = n / u32::BITS;
        let bit = n % u32::BITS;

        self.mask[word as usize] &= !(1 << bit);
    }

    fn is_enabled(&self, event: MacEvent) -> bool {
        let n = event as u32;
        let word = n / u32::BITS;
        let bit = n % u32::BITS;

        self.mask[word as usize] & (1 << bit) > 0
    }
}

#[derive(Default)]

pub struct SharedEventMask {
    mask: RefCell<EventMask>,
}

impl SharedEventMask {
    pub fn enable(&self, events: &[MacEvent]) {
        let mut mask = self.mask.borrow_mut();
        for event in events {
            mask.enable(*event);
        }
    }

    #[allow(dead_code)]
    pub fn disable(&self, events: &[MacEvent]) {
        let mut mask = self.mask.borrow_mut();
        for event in events {
            mask.disable(*event);
        }
    }

    pub fn disable_all(&self) {
        let mut mask = self.mask.borrow_mut();
        mask.mask = Default::default();
    }

    pub fn is_enabled(&self, event: MacEvent) -> bool {
        let mask = self.mask.borrow();
        mask.is_enabled(event)
    }
}
