//! USB Mass Storage Class (MSC) implementation.
//!
//! This implements the USB Bulk-Only Transport (BOT) protocol with a SCSI transparent
//! command set suitable for a simple block device.

use core::cell::Cell;
use core::cmp::min;

use embassy_sync::blocking_mutex::CriticalSectionMutex;

use super::bot::{CBW_LEN, Cbw, Csw, CswStatus};
use super::scsi::*;
use super::{SenseData, USB_CLASS_MSC, USB_PROTOCOL_BULK_ONLY, USB_SUBCLASS_SCSI_TRANSPARENT};
use crate::Builder;
use crate::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use crate::types::InterfaceNumber;

mod block_device;
mod commands;
mod config;
mod control;
mod transfer;

pub use block_device::*;
use commands::*;
pub use config::Config;
use control::Control;
pub use control::State;

/// Why [`MscClass::run`] returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum StopReason {
    /// The host ejected the medium with START STOP UNIT (LoEj = 1, Start = 0).
    ///
    /// Pending writes were flushed before the command completed.
    Ejected,
}

/// USB Mass Storage Class (MSC) implementation.
pub struct MscClass<'d, D: Driver<'d>> {
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,
    _interface: InterfaceNumber,
    reset_requested: &'d CriticalSectionMutex<Cell<bool>>,
    config: Config,
    sense: SenseData,
}

impl<'d, D: Driver<'d>> MscClass<'d, D> {
    /// Creates a new MSC class with configuration.
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State<'d>, config: Config) -> Self {
        let mut func = builder.function(USB_CLASS_MSC, USB_SUBCLASS_SCSI_TRANSPARENT, USB_PROTOCOL_BULK_ONLY);

        let mut iface = func.interface();
        let interface = iface.interface_number();
        let mut alt = iface.alt_setting(
            USB_CLASS_MSC,
            USB_SUBCLASS_SCSI_TRANSPARENT,
            USB_PROTOCOL_BULK_ONLY,
            None,
        );

        let (read_ep, write_ep) = if config.double_buffered {
            (
                alt.endpoint_bulk_out_double_buffered(None, config.max_packet_size),
                alt.endpoint_bulk_in_double_buffered(None, config.max_packet_size),
            )
        } else {
            (
                alt.endpoint_bulk_out(None, config.max_packet_size),
                alt.endpoint_bulk_in(None, config.max_packet_size),
            )
        };

        drop(func);

        let control = state.control.write(Control {
            interface,
            reset_requested: &state.reset_requested,
        });
        builder.handler(control);

        Self {
            read_ep,
            write_ep,
            _interface: interface,
            reset_requested: &state.reset_requested,
            config,
            sense: SenseData::NO_SENSE,
        }
    }

    /// Gets the endpoint max packet size in bytes.
    pub fn max_packet_size(&self) -> u16 {
        self.read_ep.info().max_packet_size
    }

    /// Waits for the USB host to enable this interface.
    pub async fn wait_connection(&mut self) {
        self.read_ep.wait_enabled().await;
    }

    /// Runs the MSC BOT state machine until the host ejects the medium.
    ///
    /// Disconnects and reconnects are handled internally. When this returns, the
    /// host has received the status of the eject command but the device is still
    /// attached: disable the USB device, or call `run` again to keep serving commands.
    ///
    /// `block_buf` is a temporary buffer used for block transfers and must be at
    /// least `block_device.block_size()` and `max_packet_size` bytes long.
    ///
    /// A buffer of two or more blocks is split in two halves, so the USB transfer of
    /// one half overlaps with `block_device` reading or writing the other. Each call
    /// to `block_device` moves up to half the buffer, so a buffer of several blocks
    /// speeds up large transfers.
    pub async fn run<B: AsyncBlockDevice>(&mut self, block_device: &mut B, block_buf: &mut [u8]) -> StopReason {
        assert!(
            block_buf.len() >= block_device.block_size() as usize
                && block_buf.len() >= self.config.max_packet_size as usize,
            "block_buf must be at least block_size and max_packet_size"
        );
        loop {
            self.wait_connection().await;
            info!("msc: connected");

            if let Ok(reason) = self.run_connected(block_device, block_buf).await {
                info!("msc: stopped: {:?}", reason);
                return reason;
            }

            info!("msc: disconnected");
        }
    }

    async fn run_connected<B: AsyncBlockDevice>(
        &mut self,
        block_device: &mut B,
        block_buf: &mut [u8],
    ) -> Result<StopReason, EndpointError> {
        loop {
            let Some(cbw) = self.read_cbw(block_buf).await? else {
                continue;
            };

            let reset_requested = self.reset_requested.lock(|flag| {
                let was_requested = flag.get();
                flag.set(false);
                was_requested
            });
            if reset_requested {
                self.sense = SenseData::NO_SENSE;
            }

            let result = self.process_cbw(block_device, block_buf, &cbw).await?;

            self.write_csw(cbw.tag, result.residue, result.status).await?;

            if let Some(reason) = result.stop {
                return Ok(reason);
            }
        }
    }

    async fn read_cbw(&mut self, buf: &mut [u8]) -> Result<Option<Cbw>, EndpointError> {
        let n = self
            .read_ep
            .read(&mut buf[..self.config.max_packet_size as usize])
            .await?;

        if n != CBW_LEN {
            warn!("msc: invalid CBW size {}", n);
            return Ok(None);
        }

        let cbw = match Cbw::parse(&buf[..CBW_LEN]) {
            Ok(cbw) => cbw,
            Err(_) => {
                warn!("msc: invalid CBW payload");
                return Ok(None);
            }
        };

        Ok(Some(cbw))
    }

    async fn write_csw(&mut self, tag: u32, residue: u32, status: CswStatus) -> Result<(), EndpointError> {
        let csw = Csw { tag, residue, status };
        self.write_ep.write(&csw.encode()).await
    }

    async fn process_cbw<B: AsyncBlockDevice>(
        &mut self,
        block_device: &mut B,
        block_buf: &mut [u8],
        cbw: &Cbw,
    ) -> Result<CommandResult, EndpointError> {
        let command = Command::parse(&cbw.cb, cbw.cb_length as usize);
        let is_write = matches!(command, Command::Write(_));

        // Drain unexpected OUT data for any non-WRITE command upfront, or any command to an unsupported LUN (BOT Cases 9 & 10)
        if !cbw.direction_in() && cbw.data_transfer_length > 0 && (!is_write || cbw.lun != 0) {
            self.discard_out_data(cbw.data_transfer_length, block_buf).await?;
        }

        // Only LUN 0 is supported.
        if cbw.lun != 0 {
            return match command {
                Command::Inquiry(Inquiry { allocation_length, .. }) => {
                    if let Some(result) = self.check_phase(cbw, DataPhase::DataIn) {
                        return Ok(result);
                    }
                    // SPC-2: an unsupported LUN reports that no device is connected. Leaves the sense data alone.
                    let mut data = [0u8; INQUIRY_DATA_LEN];
                    data[0] = PERIPHERAL_NO_LUN;
                    let len = min(data.len(), allocation_length);
                    self.send_in_data(cbw, &data[..len]).await
                }
                // SPC-4 processes REPORT LUNS sent to an incorrect logical unit normally.
                Command::ReportLuns { .. } => self.execute(cbw, command, block_device, block_buf).await,
                _ => {
                    self.sense = SenseData::LOGICAL_UNIT_NOT_SUPPORTED;
                    Ok(CommandResult::failed(cbw.data_transfer_length))
                }
            };
        }

        let result = self.execute(cbw, command, block_device, block_buf).await?;

        // If an IN command failed and no data was transferred, terminate host's Data-In
        // phase with a ZLP before the CSW is sent (prevents host reading CSW as data).
        if result.status != CswStatus::Passed
            && cbw.direction_in()
            && result.residue == cbw.data_transfer_length
            && cbw.data_transfer_length > 0
        {
            let _ = self.write_ep.write(&[]).await;
        }

        Ok(result)
    }

    /// Runs `command` against `block_device`, including its data phase.
    async fn execute<B: AsyncBlockDevice>(
        &mut self,
        cbw: &Cbw,
        command: Command,
        block_device: &mut B,
        block_buf: &mut [u8],
    ) -> Result<CommandResult, EndpointError> {
        if let Some(phase) = command.data_phase()
            && let Some(result) = self.check_phase(cbw, phase)
        {
            return Ok(result);
        }

        let mut response = [0u8; RESPONSE_LEN];
        let reply = match command {
            // READ and WRITE stream their data and handle sense themselves.
            Command::Read(rw) => return self.read(cbw, rw, block_device, block_buf).await,
            Command::Write(rw) => return self.write(cbw, rw, block_device, block_buf).await,
            Command::TestUnitReady | Command::PreventAllowMediumRemoval => Ok(Reply::Done),
            Command::RequestSense { allocation_length } => request_sense(&self.sense, allocation_length, &mut response),
            Command::Inquiry(cmd) => inquiry(&self.config, cmd, &mut response),
            Command::ModeSense6 { allocation_length } => mode_sense_6(allocation_length, block_device, &mut response),
            Command::ModeSense10 { allocation_length } => mode_sense_10(allocation_length, block_device, &mut response),
            Command::ReadFormatCapacities { allocation_length } => {
                read_format_capacities(allocation_length, block_device, &mut response)
            }
            Command::ReadCapacity10 => read_capacity_10(block_device, &mut response),
            Command::ReadCapacity16 { allocation_length } => {
                read_capacity_16(allocation_length, block_device, &mut response)
            }
            Command::StartStopUnit { eject } => start_stop_unit(eject, block_device).await,
            Command::SynchronizeCache10 => synchronize_cache_10(block_device).await,
            Command::ReportLuns {
                select_report,
                allocation_length,
            } => report_luns(select_report, allocation_length, &mut response),
            Command::Unsupported { opcode } => {
                debug!("msc: unsupported opcode 0x{:02x}", opcode);
                Err(SenseData::INVALID_COMMAND_OPERATION_CODE)
            }
        };

        match reply {
            Ok(reply) => {
                self.sense = SenseData::NO_SENSE;
                match reply {
                    Reply::Done => Ok(CommandResult::passed(0)),
                    Reply::Data(len) => self.send_in_data(cbw, &response[..len]).await,
                    Reply::Stop(reason) => Ok(CommandResult {
                        stop: Some(reason),
                        ..CommandResult::passed(0)
                    }),
                }
            }
            Err(sense) => {
                self.sense = sense;
                Ok(CommandResult::failed(cbw.data_transfer_length))
            }
        }
    }

    /// Checks the data phase the host announced in `cbw` against the one the command expects.
    ///
    /// Returns the result to report if they disagree.
    fn check_phase(&mut self, cbw: &Cbw, phase: DataPhase) -> Option<CommandResult> {
        let len = cbw.data_transfer_length;
        match phase {
            DataPhase::NoData if len != 0 => {
                self.sense = SenseData::INVALID_FIELD_IN_CDB;
                Some(CommandResult::failed(len))
            }
            DataPhase::DataIn if len > 0 && !cbw.direction_in() => Some(CommandResult::phase_error(len)),
            DataPhase::DataOut if len > 0 && cbw.direction_in() => Some(CommandResult::phase_error(len)),
            _ => None,
        }
    }

    async fn send_in_data(&mut self, cbw: &Cbw, data: &[u8]) -> Result<CommandResult, EndpointError> {
        let transfer_len = min(data.len(), cbw.data_transfer_length as usize);
        self.write_ep.write_transfer(&data[..transfer_len], false).await?;

        // If transfer completed with fewer bytes than expected by the host, and the
        // length transferred is a multiple of max_packet_size (including 0), send a ZLP
        // to terminate the USB transfer.
        if (transfer_len as u32) < cbw.data_transfer_length
            && transfer_len.is_multiple_of(self.config.max_packet_size as usize)
        {
            self.write_ep.write(&[]).await?;
        }

        let residue = cbw.data_transfer_length.saturating_sub(transfer_len as u32);
        Ok(CommandResult::passed(residue))
    }

    async fn discard_out_data(&mut self, mut len: u32, scratch: &mut [u8]) -> Result<(), EndpointError> {
        if scratch.is_empty() {
            return Ok(());
        }

        let max_pkt = self.config.max_packet_size as usize;
        while len > 0 {
            // Read at least max_packet_size into scratch if possible to avoid BufferOverflow.
            let chunk_len = min(scratch.len(), core::cmp::max(len as usize, max_pkt));
            let n = self.read_ep.read(&mut scratch[..chunk_len]).await?;
            len = len.saturating_sub(n as u32);
            if n < max_pkt {
                break;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
struct CommandResult {
    residue: u32,
    status: CswStatus,
    stop: Option<StopReason>,
}

impl CommandResult {
    const fn passed(residue: u32) -> Self {
        Self {
            residue,
            status: CswStatus::Passed,
            stop: None,
        }
    }

    const fn failed(residue: u32) -> Self {
        Self {
            residue,
            status: CswStatus::Failed,
            stop: None,
        }
    }

    const fn phase_error(residue: u32) -> Self {
        Self {
            residue,
            status: CswStatus::PhaseError,
            stop: None,
        }
    }
}
