use core::{mem, ptr};

use super::indications::{
    AssociateIndication, BeaconNotifyIndication, CommStatusIndication, DataIndication, DisassociateIndication,
    DpsIndication, GtsIndication, OrphanIndication, PollIndication, SyncLossIndication,
};
use super::responses::{
    AssociateConfirm, CalibrateConfirm, DataConfirm, DisassociateConfirm, DpsConfirm, GetConfirm, GtsConfirm,
    PollConfirm, PurgeConfirm, ResetConfirm, RxEnableConfirm, ScanConfirm, SetConfirm, SoundingConfirm, StartConfirm,
};
use crate::evt::{EvtBox, MemoryManager};
use crate::mac::opcodes::OpcodeM0ToM4;
use crate::sub::mac::{self, Mac};

pub(crate) trait ParseableMacEvent: Sized {
    fn from_buffer<'a>(buf: &'a [u8]) -> Result<&'a Self, ()> {
        if buf.len() < mem::size_of::<Self>() {
            Err(())
        } else {
            Ok(unsafe { &*(buf as *const _ as *const Self) })
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug)]
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

impl<'a> MacEvent<'a> {
    pub(crate) fn new(event_box: EvtBox<Mac>) -> Result<Self, ()> {
        let payload = event_box.payload();
        let opcode = u16::from_le_bytes(payload[0..2].try_into().unwrap());

        let opcode = OpcodeM0ToM4::try_from(opcode)?;
        let buf = &payload[2..];

        // To avoid re-parsing the opcode, we store the result of the parse
        // this requires use of unsafe because rust cannot assume that a reference will become
        // invalid when the underlying result is moved. However, because we refer to a "heap"
        // allocation, the underlying reference will not move until the struct is dropped.

        let mac_event = match opcode {
            OpcodeM0ToM4::MlmeAssociateCnf => {
                MacEvent::MlmeAssociateCnf(unsafe { &*(AssociateConfirm::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmeDisassociateCnf => {
                MacEvent::MlmeDisassociateCnf(unsafe { &*(DisassociateConfirm::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmeGetCnf => MacEvent::MlmeGetCnf(unsafe { &*(GetConfirm::from_buffer(buf)? as *const _) }),
            OpcodeM0ToM4::MlmeGtsCnf => MacEvent::MlmeGtsCnf(unsafe { &*(GtsConfirm::from_buffer(buf)? as *const _) }),
            OpcodeM0ToM4::MlmeResetCnf => {
                MacEvent::MlmeResetCnf(unsafe { &*(ResetConfirm::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmeRxEnableCnf => {
                MacEvent::MlmeRxEnableCnf(unsafe { &*(RxEnableConfirm::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmeScanCnf => {
                MacEvent::MlmeScanCnf(unsafe { &*(ScanConfirm::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmeSetCnf => MacEvent::MlmeSetCnf(unsafe { &*(SetConfirm::from_buffer(buf)? as *const _) }),
            OpcodeM0ToM4::MlmeStartCnf => {
                MacEvent::MlmeStartCnf(unsafe { &*(StartConfirm::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmePollCnf => {
                MacEvent::MlmePollCnf(unsafe { &*(PollConfirm::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmeDpsCnf => MacEvent::MlmeDpsCnf(unsafe { &*(DpsConfirm::from_buffer(buf)? as *const _) }),
            OpcodeM0ToM4::MlmeSoundingCnf => {
                MacEvent::MlmeSoundingCnf(unsafe { &*(SoundingConfirm::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmeCalibrateCnf => {
                MacEvent::MlmeCalibrateCnf(unsafe { &*(CalibrateConfirm::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::McpsDataCnf => {
                MacEvent::McpsDataCnf(unsafe { &*(DataConfirm::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::McpsPurgeCnf => {
                MacEvent::McpsPurgeCnf(unsafe { &*(PurgeConfirm::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmeAssociateInd => {
                MacEvent::MlmeAssociateInd(unsafe { &*(AssociateIndication::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmeDisassociateInd => {
                MacEvent::MlmeDisassociateInd(unsafe { &*(DisassociateIndication::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmeBeaconNotifyInd => {
                MacEvent::MlmeBeaconNotifyInd(unsafe { &*(BeaconNotifyIndication::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmeCommStatusInd => {
                MacEvent::MlmeCommStatusInd(unsafe { &*(CommStatusIndication::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmeGtsInd => {
                MacEvent::MlmeGtsInd(unsafe { &*(GtsIndication::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmeOrphanInd => {
                MacEvent::MlmeOrphanInd(unsafe { &*(OrphanIndication::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmeSyncLossInd => {
                MacEvent::MlmeSyncLossInd(unsafe { &*(SyncLossIndication::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmeDpsInd => {
                MacEvent::MlmeDpsInd(unsafe { &*(DpsIndication::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::McpsDataInd => {
                MacEvent::McpsDataInd(unsafe { &*(DataIndication::from_buffer(buf)? as *const _) })
            }
            OpcodeM0ToM4::MlmePollInd => {
                MacEvent::MlmePollInd(unsafe { &*(PollIndication::from_buffer(buf)? as *const _) })
            }
        };

        // Forget the event box so that drop isn't called
        // We want to handle the lifetime ourselves

        mem::forget(event_box);

        Ok(mac_event)
    }
}

unsafe impl<'a> Send for MacEvent<'a> {}

impl<'a> Drop for MacEvent<'a> {
    fn drop(&mut self) {
        unsafe { mac::Mac::drop_event_packet(ptr::null_mut()) };
    }
}
