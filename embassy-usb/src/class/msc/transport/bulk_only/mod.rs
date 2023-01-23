pub mod cbw;
pub mod csw;

use core::mem::{size_of, MaybeUninit};

use embassy_usb_driver::{Direction, Endpoint, EndpointError, EndpointIn, EndpointOut};

use self::cbw::CommandBlockWrapper;
use self::csw::{CommandStatus, CommandStatusWrapper};
use super::{CommandError, CommandSetHandler, DataPipeError, DataPipeIn, DataPipeOut};
use crate::class::msc::{MscProtocol, MscSubclass, USB_CLASS_MSC};
use crate::control::{ControlHandler, InResponse, Request, RequestType};
use crate::driver::Driver;
use crate::Builder;

const REQ_GET_MAX_LUN: u8 = 0xFE;
const REQ_BULK_ONLY_RESET: u8 = 0xFF;

pub struct State {
    control: MaybeUninit<Control>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            control: MaybeUninit::uninit(),
        }
    }
}

pub struct Control {
    max_lun: u8,
}

impl ControlHandler for Control {
    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> InResponse<'a> {
        match (req.request_type, req.request) {
            (RequestType::Class, REQ_GET_MAX_LUN) => {
                debug!("REQ_GET_MAX_LUN");
                buf[0] = self.max_lun;
                InResponse::Accepted(&buf[..1])
            }
            (RequestType::Class, REQ_BULK_ONLY_RESET) => {
                debug!("REQ_BULK_ONLY_RESET");
                InResponse::Accepted(&[])
            }
            _ => InResponse::Rejected,
        }
    }
}

pub struct BulkOnlyTransport<'d, D: Driver<'d>, C: CommandSetHandler> {
    read_ep: D::EndpointOut,
    write_ep: D::EndpointIn,
    max_packet_size: u16,
    handler: C,
}

impl<'d, D: Driver<'d>, C: CommandSetHandler> BulkOnlyTransport<'d, D, C> {
    pub fn new(
        builder: &mut Builder<'d, D>,
        state: &'d mut State,
        subclass: MscSubclass,
        max_packet_size: u16,
        max_lun: u8,
        handler: C,
    ) -> Self {
        assert!(max_lun < 16, "BulkOnlyTransport supports maximum 16 LUNs");

        let control = state.control.write(Control { max_lun });

        let mut func = builder.function(USB_CLASS_MSC, subclass as _, MscProtocol::BulkOnlyTransport as _);

        // Control interface
        let mut iface = func.interface();
        iface.handler(control);

        let mut alt = iface.alt_setting(USB_CLASS_MSC, subclass as _, MscProtocol::BulkOnlyTransport as _);

        let read_ep = alt.endpoint_bulk_out(max_packet_size);
        let write_ep = alt.endpoint_bulk_in(max_packet_size);

        Self {
            read_ep,
            write_ep,
            max_packet_size,
            handler,
        }
    }

    async fn receive_control_block_wrapper(&mut self) -> CommandBlockWrapper {
        let mut cbw_buf = [0u8; size_of::<CommandBlockWrapper>()];

        loop {
            // CBW is always sent at a packet boundary and is a short packet of 31 bytes
            match self.read_ep.read(&mut cbw_buf).await {
                Ok(len) => {
                    if len != cbw_buf.len() {
                        error!("Invalid CBW length");
                    }

                    match CommandBlockWrapper::from_bytes(&cbw_buf) {
                        Ok(cbw) => return cbw,
                        Err(e) => {
                            error!("Invalid CBW: {:?}", e);
                        }
                    }
                }
                Err(e) => match e {
                    EndpointError::BufferOverflow => {
                        error!("Host sent too long CBW");
                    }
                    EndpointError::Disabled => self.read_ep.wait_enabled().await,
                },
            };
        }
    }

    async fn send_csw(&mut self, csw: CommandStatusWrapper) {
        let mut csw_buf = [0u8; size_of::<CommandStatusWrapper>()];
        match self.write_ep.write(csw.to_bytes(&mut csw_buf)).await {
            Ok(_) => {}
            Err(e) => error!("error sending CSW: {:?}", e),
        }
    }

    async fn handle_command_out(&mut self, cbw: CommandBlockWrapper) -> CommandStatusWrapper {
        let mut pipe_out = BulkOnlyTransportDataPipeOut {
            ep: &mut self.read_ep,
            data_residue: cbw.data_transfer_length as _,
            max_packet_size: self.max_packet_size,
            last_packet_full: true,
        };

        let status = match self.handler.command_out(cbw.lun, cbw.data(), &mut pipe_out).await {
            Ok(_) => CommandStatus::CommandOk,
            Err(e) => match e {
                CommandError::PipeError(e) => {
                    error!("data pipe error: {:?}", e);
                    CommandStatus::PhaseError
                }
                CommandError::CommandError => CommandStatus::CommandError,
            },
        };

        CommandStatusWrapper::new(cbw.tag, pipe_out.data_residue, status)
    }

    async fn handle_command_in(&mut self, cbw: CommandBlockWrapper) -> CommandStatusWrapper {
        let mut pipe_in = BulkOnlyTransportDataPipeIn {
            ep: &mut self.write_ep,
            data_residue: cbw.data_transfer_length as _,
            max_packet_size: self.max_packet_size,
            last_packet_full: true,
        };

        let status = match self.handler.command_in(cbw.lun, cbw.data(), &mut pipe_in).await {
            Ok(_) => match pipe_in.finalize().await {
                Ok(_) => CommandStatus::CommandOk,
                Err(e) => {
                    error!("Error finalizing data pipe: {:?}", e);
                    CommandStatus::PhaseError
                }
            },
            Err(e) => match e {
                CommandError::PipeError(e) => {
                    error!("data pipe error: {:?}", e);
                    CommandStatus::PhaseError
                }
                CommandError::CommandError => CommandStatus::CommandError,
            },
        };

        CommandStatusWrapper::new(cbw.tag, pipe_in.data_residue, status)
    }

    pub async fn run(&mut self) {
        loop {
            let cbw = self.receive_control_block_wrapper().await;
            trace!("received CBW");

            let csw = match cbw.dir() {
                Direction::Out => {
                    trace!("handle_command_out");
                    self.handle_command_out(cbw).await
                }
                Direction::In => {
                    trace!("handle_command_in");
                    self.handle_command_in(cbw).await
                }
            };

            trace!("sending CSW");
            self.send_csw(csw).await;
        }
    }
}

pub struct BulkOnlyTransportDataPipeIn<'d, E: EndpointIn> {
    ep: &'d mut E,
    // requested transfer size minus already transfered bytes
    data_residue: u32,
    max_packet_size: u16,
    last_packet_full: bool,
}

impl<'d, E: EndpointIn> DataPipeIn for BulkOnlyTransportDataPipeIn<'d, E> {
    async fn write(&mut self, buf: &[u8]) -> Result<(), DataPipeError> {
        if !self.last_packet_full {
            return Err(DataPipeError::TransferFinalized);
        }

        for chunk in buf.chunks(self.max_packet_size.into()) {
            if self.data_residue < chunk.len() as _ {
                return Err(DataPipeError::TransferSizeExceeded);
            }

            self.ep.write(chunk).await?;
            self.data_residue -= chunk.len() as u32;
            self.last_packet_full = chunk.len() == self.max_packet_size.into();
        }

        Ok(())
    }
}

impl<'d, E: EndpointIn> BulkOnlyTransportDataPipeIn<'d, E> {
    async fn finalize(&mut self) -> Result<(), DataPipeError> {
        // Send ZLP only if last packet was full and transfer size was not exhausted
        if self.last_packet_full && self.data_residue != 0 {
            self.ep.write(&[]).await?;
        }

        Ok(())
    }
}

pub struct BulkOnlyTransportDataPipeOut<'d, E: EndpointOut> {
    ep: &'d mut E,
    // requested transfer size minus already transfered bytes
    data_residue: u32,
    max_packet_size: u16,
    last_packet_full: bool,
}

impl<'d, E: EndpointOut> DataPipeOut for BulkOnlyTransportDataPipeOut<'d, E> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<(), DataPipeError> {
        if !self.last_packet_full {
            return Err(DataPipeError::TransferFinalized);
        }

        for chunk in buf.chunks_mut(self.max_packet_size.into()) {
            if self.data_residue < chunk.len() as _ {
                return Err(DataPipeError::TransferSizeExceeded);
            }

            self.ep.read(chunk).await?;
            self.data_residue -= chunk.len() as u32;
            self.last_packet_full = chunk.len() == self.max_packet_size.into();
        }

        Ok(())
    }
}
