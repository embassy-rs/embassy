use core::mem;

use super::indications::{
    AssociateIndication, BeaconNotifyIndication, CommStatusIndication, DataIndication, DisassociateIndication,
    DpsIndication, GtsIndication, OrphanIndication, PollIndication, SyncLossIndication,
};
use super::responses::{
    AssociateConfirm, CalibrateConfirm, DataConfirm, DisassociateConfirm, DpsConfirm, GetConfirm, GtsConfirm,
    PollConfirm, PurgeConfirm, ResetConfirm, RxEnableConfirm, ScanConfirm, SetConfirm, SoundingConfirm, StartConfirm,
};
use crate::evt::EvtBox;
use crate::mac::opcodes::OpcodeM0ToM4;
use crate::sub::mac::Mac;

pub(crate) trait ParseableMacEvent: Sized {
    fn from_buffer<'a>(buf: &'a [u8]) -> Result<&'a Self, ()> {
        if buf.len() < mem::size_of::<Self>() {
            Err(())
        } else {
            Ok(unsafe { &*(buf as *const _ as *const Self) })
        }
    }
}

pub struct Event {
    event_box: EvtBox<Mac>,
}

impl Event {
    pub(crate) fn new(event_box: EvtBox<Mac>) -> Self {
        Self { event_box }
    }

    pub fn mac_event<'a>(&'a self) -> Result<MacEvent<'a>, ()> {
        let payload = self.event_box.payload();
        let opcode = u16::from_le_bytes(payload[0..2].try_into().unwrap());

        let opcode = OpcodeM0ToM4::try_from(opcode)?;

        match opcode {
            OpcodeM0ToM4::MlmeAssociateCnf => Ok(MacEvent::MlmeAssociateCnf(AssociateConfirm::from_buffer(
                &payload[2..],
            )?)),
            OpcodeM0ToM4::MlmeDisassociateCnf => Ok(MacEvent::MlmeDisassociateCnf(DisassociateConfirm::from_buffer(
                &payload[2..],
            )?)),
            OpcodeM0ToM4::MlmeGetCnf => Ok(MacEvent::MlmeGetCnf(GetConfirm::from_buffer(&payload[2..])?)),
            OpcodeM0ToM4::MlmeGtsCnf => Ok(MacEvent::MlmeGtsCnf(GtsConfirm::from_buffer(&payload[2..])?)),
            OpcodeM0ToM4::MlmeResetCnf => Ok(MacEvent::MlmeResetCnf(ResetConfirm::from_buffer(&payload[2..])?)),
            OpcodeM0ToM4::MlmeRxEnableCnf => {
                Ok(MacEvent::MlmeRxEnableCnf(RxEnableConfirm::from_buffer(&payload[2..])?))
            }
            OpcodeM0ToM4::MlmeScanCnf => Ok(MacEvent::MlmeScanCnf(ScanConfirm::from_buffer(&payload[2..])?)),
            OpcodeM0ToM4::MlmeSetCnf => Ok(MacEvent::MlmeSetCnf(SetConfirm::from_buffer(&payload[2..])?)),
            OpcodeM0ToM4::MlmeStartCnf => Ok(MacEvent::MlmeStartCnf(StartConfirm::from_buffer(&payload[2..])?)),
            OpcodeM0ToM4::MlmePollCnf => Ok(MacEvent::MlmePollCnf(PollConfirm::from_buffer(&payload[2..])?)),
            OpcodeM0ToM4::MlmeDpsCnf => Ok(MacEvent::MlmeDpsCnf(DpsConfirm::from_buffer(&payload[2..])?)),
            OpcodeM0ToM4::MlmeSoundingCnf => {
                Ok(MacEvent::MlmeSoundingCnf(SoundingConfirm::from_buffer(&payload[2..])?))
            }
            OpcodeM0ToM4::MlmeCalibrateCnf => Ok(MacEvent::MlmeCalibrateCnf(CalibrateConfirm::from_buffer(
                &payload[2..],
            )?)),
            OpcodeM0ToM4::McpsDataCnf => Ok(MacEvent::McpsDataCnf(DataConfirm::from_buffer(&payload[2..])?)),
            OpcodeM0ToM4::McpsPurgeCnf => Ok(MacEvent::McpsPurgeCnf(PurgeConfirm::from_buffer(&payload[2..])?)),
            OpcodeM0ToM4::MlmeAssociateInd => Ok(MacEvent::MlmeAssociateInd(AssociateIndication::from_buffer(
                &payload[2..],
            )?)),
            OpcodeM0ToM4::MlmeDisassociateInd => Ok(MacEvent::MlmeDisassociateInd(
                DisassociateIndication::from_buffer(&payload[2..])?,
            )),
            OpcodeM0ToM4::MlmeBeaconNotifyInd => Ok(MacEvent::MlmeBeaconNotifyInd(
                BeaconNotifyIndication::from_buffer(&payload[2..])?,
            )),
            OpcodeM0ToM4::MlmeCommStatusInd => Ok(MacEvent::MlmeCommStatusInd(CommStatusIndication::from_buffer(
                &payload[2..],
            )?)),
            OpcodeM0ToM4::MlmeGtsInd => Ok(MacEvent::MlmeGtsInd(GtsIndication::from_buffer(&payload[2..])?)),
            OpcodeM0ToM4::MlmeOrphanInd => Ok(MacEvent::MlmeOrphanInd(OrphanIndication::from_buffer(&payload[2..])?)),
            OpcodeM0ToM4::MlmeSyncLossInd => Ok(MacEvent::MlmeSyncLossInd(SyncLossIndication::from_buffer(
                &payload[2..],
            )?)),
            OpcodeM0ToM4::MlmeDpsInd => Ok(MacEvent::MlmeDpsInd(DpsIndication::from_buffer(&payload[2..])?)),
            OpcodeM0ToM4::McpsDataInd => Ok(MacEvent::McpsDataInd(DataIndication::from_buffer(&payload[2..])?)),
            OpcodeM0ToM4::MlmePollInd => Ok(MacEvent::MlmePollInd(PollIndication::from_buffer(&payload[2..])?)),
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MacEvent<'a> {
    MlmeAssociateCnf(&'a AssociateConfirm),
    MlmeDisassociateCnf(&'a DisassociateConfirm),
    MlmeGetCnf(&'a GetConfirm),
    MlmeGtsCnf(&'a GtsConfirm),
    MlmeResetCnf(&'a ResetConfirm),
    MlmeRxEnableCnf(&'a RxEnableConfirm),
    MlmeScanCnf(&'a ScanConfirm),
    MlmeSetCnf(&'a SetConfirm),
    MlmeStartCnf(&'a StartConfirm),
    MlmePollCnf(&'a PollConfirm),
    MlmeDpsCnf(&'a DpsConfirm),
    MlmeSoundingCnf(&'a SoundingConfirm),
    MlmeCalibrateCnf(&'a CalibrateConfirm),
    McpsDataCnf(&'a DataConfirm),
    McpsPurgeCnf(&'a PurgeConfirm),
    MlmeAssociateInd(&'a AssociateIndication),
    MlmeDisassociateInd(&'a DisassociateIndication),
    MlmeBeaconNotifyInd(&'a BeaconNotifyIndication),
    MlmeCommStatusInd(&'a CommStatusIndication),
    MlmeGtsInd(&'a GtsIndication),
    MlmeOrphanInd(&'a OrphanIndication),
    MlmeSyncLossInd(&'a SyncLossIndication),
    MlmeDpsInd(&'a DpsIndication),
    McpsDataInd(&'a DataIndication),
    MlmePollInd(&'a PollIndication),
}
