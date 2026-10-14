//! Bulk-Only Transport command cycles and recovery.

use core::marker::PhantomData;

use embassy_usb_driver::Direction as UsbDirection;
use embassy_usb_driver::host::{PipeError, UsbHostAllocator, UsbPipe, pipe};

use super::super::bot::{CSW_LEN, Cbw, Csw, CswStatus};
use super::super::scsi::*;
use super::super::{BOT_REQ_GET_MAX_LUN, BOT_REQ_RESET};
use super::{CommandOutcome, DataDir, MscError, SenseData};
use crate::control::Request;
use crate::host::control::{ControlType, Recipient, RequestType, SetupPacket};

/// Shared, mutex-protected BBB transport state.
pub(super) struct Transport<'d, A>
where
    A: UsbHostAllocator<'d>,
{
    pub(super) ctrl: A::Pipe<pipe::Control, pipe::InOut>,
    pub(super) bulk_in: A::Pipe<pipe::Bulk, pipe::In>,
    pub(super) bulk_out: A::Pipe<pipe::Bulk, pipe::Out>,
    pub(super) interface: u8,
    pub(super) bulk_in_ep: u8,
    pub(super) bulk_out_ep: u8,
    pub(super) next_tag: u32,
    /// Set when a command was cancelled mid-transfer; the next command
    /// resets the transport before running.
    pub(super) dirty: bool,
    pub(super) _phantom: PhantomData<&'d ()>,
}

impl<'d, A> Transport<'d, A>
where
    A: UsbHostAllocator<'d>,
{
    pub(super) async fn clear_halt_in(&mut self) -> Result<(), MscError> {
        clear_endpoint_halt(&mut self.ctrl, self.bulk_in_ep).await?;
        self.bulk_in.reset_data_toggle();
        Ok(())
    }

    pub(super) async fn clear_halt_out(&mut self) -> Result<(), MscError> {
        clear_endpoint_halt(&mut self.ctrl, self.bulk_out_ep).await?;
        self.bulk_out.reset_data_toggle();
        Ok(())
    }

    pub(super) async fn mass_storage_reset(&mut self) -> Result<(), MscError> {
        let setup = SetupPacket::class_interface_out(BOT_REQ_RESET, 0, self.interface as u16, 0);
        self.ctrl.control_out(&setup.to_bytes(), &[]).await?;
        self.clear_halt_in().await?;
        self.clear_halt_out().await?;
        Ok(())
    }

    /// Run one CBW → data → CSW cycle.
    pub(super) async fn run_cycle(
        &mut self,
        lun: u8,
        cdb: &[u8],
        data: DataDir<'_>,
    ) -> Result<CommandOutcome, MscError> {
        let tag = self.next_tag;
        self.next_tag = self.next_tag.wrapping_add(1);

        let mut cb = [0u8; 16];
        cb[..cdb.len()].copy_from_slice(cdb);
        let cbw = Cbw {
            tag,
            data_transfer_length: data.len() as u32,
            flags: data.cbw_flags(),
            lun,
            cb_length: cdb.len() as u8,
            cb,
        };

        trace!(
            "MSC: CBW tag={:#010x} lun={} op={:#04x} data_len={} dir={}",
            tag,
            lun,
            cdb[0],
            data.len(),
            match data {
                DataDir::None => "none",
                DataDir::In(_) => "in",
                DataDir::Out(_) => "out",
            },
        );

        // CBW phase.
        if let Err(e) = self.bulk_out.request_out(&cbw.encode(), false).await {
            if matches!(e, PipeError::Stall) {
                self.clear_halt_out().await?;
            }
            return Err(e.into());
        }

        // Data phase. A stall here is recoverable: clear the halt, then
        // read the CSW.
        match data {
            DataDir::None => {}
            DataDir::In(buf) => match self.bulk_in.request_in(buf).await {
                Ok(_) => {}
                Err(PipeError::Stall) => self.clear_halt_in().await?,
                Err(e) => return Err(e.into()),
            },
            DataDir::Out(buf) => match self.bulk_out.request_out(buf, false).await {
                Ok(()) => {}
                Err(PipeError::Stall) => self.clear_halt_out().await?,
                Err(e) => return Err(e.into()),
            },
        }

        // CSW phase. A stall here gets one retry after clearing.
        let raw = match self.read_csw().await {
            Ok(b) => b,
            Err(MscError::Transfer(PipeError::Stall)) => {
                self.clear_halt_in().await?;
                self.read_csw().await?
            }
            Err(e) => return Err(e),
        };

        let csw = match Csw::parse(&raw) {
            Ok(csw) if csw.tag == tag => csw,
            parsed => {
                warn!(
                    "MSC: invalid CSW (expected tag={:#010x}, got {:?}, raw={:?})",
                    tag, parsed, raw
                );
                self.mass_storage_reset().await.ok();
                return Err(MscError::Protocol);
            }
        };

        match csw.status {
            CswStatus::Passed => Ok(CommandOutcome::Ok { residue: csw.residue }),
            CswStatus::Failed => Ok(CommandOutcome::Failed { residue: csw.residue }),
            CswStatus::PhaseError => {
                self.mass_storage_reset().await?;
                Err(MscError::PhaseError)
            }
        }
    }

    pub(super) async fn read_csw(&mut self) -> Result<[u8; CSW_LEN], MscError> {
        let mut buf = [0u8; CSW_LEN];
        let n = self.bulk_in.request_in(&mut buf).await?;
        if n == CSW_LEN {
            trace!("MSC: CSW raw={:?}", buf);
            Ok(buf)
        } else {
            warn!(
                "MSC: short CSW ({} bytes, expected {}), data={:?}",
                n,
                CSW_LEN,
                &buf[..n]
            );
            Err(MscError::Protocol)
        }
    }

    /// Run one command: dirty-check, recover if needed, run the cycle,
    /// and clear dirty on success.
    pub(super) async fn command(&mut self, lun: u8, cdb: &[u8], data: DataDir<'_>) -> Result<CommandOutcome, MscError> {
        if self.dirty {
            // A previous command was cancelled or wedged the transport;
            // re-sync before starting.
            self.mass_storage_reset().await?;
            self.dirty = false;
        }
        self.dirty = true;
        let result = self.run_cycle(lun, cdb, data).await;
        if result.is_ok() {
            self.dirty = false;
        }
        result
    }

    /// Issue `REQUEST SENSE`.
    pub(super) async fn request_sense(&mut self, lun: u8) -> Result<SenseData, MscError> {
        let cdb = [SCSI_REQUEST_SENSE, 0, 0, 0, SENSE_DATA_LEN as u8, 0];
        let mut buf = [0u8; SENSE_DATA_LEN];
        // Accept Failed here too — a second failure would loop.
        let _ = self.command(lun, &cdb, DataDir::In(&mut buf)).await?;
        Ok(SenseData::parse(&buf))
    }

    /// Run a command, auto-issuing `REQUEST SENSE` on CSW Failed so the
    /// sense data reports the command we just ran (the caller holds the
    /// lock, so no other task can interleave).
    pub(super) async fn run_with_sense(&mut self, lun: u8, cdb: &[u8], data: DataDir<'_>) -> Result<u32, MscError> {
        match self.command(lun, cdb, data).await? {
            CommandOutcome::Ok { residue } => Ok(residue),
            CommandOutcome::Failed { .. } => Err(MscError::Scsi(self.request_sense(lun).await?)),
        }
    }
}

async fn clear_endpoint_halt<P>(ctrl: &mut P, ep_addr: u8) -> Result<(), MscError>
where
    P: UsbPipe<pipe::Control, pipe::InOut>,
{
    let setup = SetupPacket {
        request_type: RequestType {
            direction: UsbDirection::Out,
            control_type: ControlType::Standard,
            recipient: Recipient::Endpoint,
        },
        request: Request::CLEAR_FEATURE,
        value: Request::FEATURE_ENDPOINT_HALT,
        index: ep_addr as u16,
        length: 0,
    };
    ctrl.control_out(&setup.to_bytes(), &[]).await?;
    Ok(())
}

pub(super) async fn get_max_lun<P>(ctrl: &mut P, interface: u8) -> Result<u8, MscError>
where
    P: UsbPipe<pipe::Control, pipe::InOut>,
{
    let setup = SetupPacket::class_interface_in(BOT_REQ_GET_MAX_LUN, 0, interface as u16, 1);
    let mut buf = [0u8; 1];
    // Many devices stall GET_MAX_LUN — treat that as "single LUN".
    match ctrl.control_in(&setup.to_bytes(), &mut buf).await {
        Ok(1) if buf[0] <= 15 => Ok(buf[0]),
        Ok(_) => Err(MscError::InvalidResponse),
        Err(PipeError::Stall) => Ok(0),
        Err(e) => Err(e.into()),
    }
}
