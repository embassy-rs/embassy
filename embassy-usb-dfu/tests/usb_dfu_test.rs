use core::cell::RefCell;
use core::convert::Infallible;

use dfu_core::DfuIo;
use embassy_boot::{BlockingFirmwareUpdater, FirmwareUpdaterConfig};
use embassy_usb::Handler;
use embassy_usb::class::dfu::dfu_mode::Handler as DfuModeHandler;
use embassy_usb::control::{InResponse, OutResponse, Recipient, Request as ControlRequest, RequestType};
use embassy_usb::driver::Direction;
use embassy_usb_dfu::consts::DfuAttributes;
use embassy_usb_dfu::{Reset, UsbDfuState, new_state};
use embedded_storage::nor_flash::{ErrorType, NorFlash, ReadNorFlash};

const READ_WRITE_SIZE: usize = 8;

struct InMemoryFlashPartition<'a, const SIZE: usize> {
    buffer: &'a RefCell<[u8; SIZE]>,
}

impl<'a, const SIZE: usize> ReadNorFlash for InMemoryFlashPartition<'a, SIZE> {
    const READ_SIZE: usize = READ_WRITE_SIZE;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let start = offset as usize;
        bytes.copy_from_slice(&self.buffer.borrow()[start..start + bytes.len()]);
        Ok(())
    }

    fn capacity(&self) -> usize {
        self.buffer.borrow().len()
    }
}

impl<'a, const SIZE: usize> ErrorType for InMemoryFlashPartition<'a, SIZE> {
    type Error = Infallible;
}

impl<'a, const SIZE: usize> NorFlash for InMemoryFlashPartition<'a, SIZE> {
    const WRITE_SIZE: usize = READ_WRITE_SIZE;

    const ERASE_SIZE: usize = READ_WRITE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        let (from, to) = (from as usize, to as usize);
        self.buffer.borrow_mut()[from..to].fill(0);
        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        let start = offset as usize;
        self.buffer.borrow_mut()[start..start + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }
}

struct NoopReset {}

impl Reset for NoopReset {
    fn sys_reset(&self) {
        // noop
    }
}

struct InMemoryDfu<H: DfuModeHandler> {
    functional_descriptor: dfu_core::functional_descriptor::FunctionalDescriptor,
    dfu_state: RefCell<UsbDfuState<H>>,
}

impl<H: DfuModeHandler> DfuIo for InMemoryDfu<H> {
    type Read = usize;

    type Write = usize;

    type Reset = ();

    type Error = anyhow::Error;

    type MemoryLayout = dfu_core::memory_layout::MemoryLayout;

    fn read_control(
        &self,
        _request_type: u8,
        request: u8,
        value: u16,
        buffer: &mut [u8],
    ) -> Result<Self::Read, Self::Error> {
        let req = ControlRequest {
            direction: Direction::In,
            request_type: RequestType::Class,
            recipient: Recipient::Interface,
            request,
            value,
            index: 0,
            length: buffer.len() as u16,
        };
        let mut state = self.dfu_state.borrow_mut();
        match state.control_in(req, buffer) {
            Some(InResponse::Accepted(data)) => Ok(data.len()),
            Some(InResponse::Rejected) => Err(anyhow::anyhow!("control in rejected")),
            None => Err(anyhow::anyhow!("control in returned None")),
        }
    }

    fn write_control(
        &self,
        _request_type: u8,
        request: u8,
        value: u16,
        buffer: &[u8],
    ) -> Result<Self::Write, Self::Error> {
        let req = ControlRequest {
            direction: Direction::Out,
            request_type: RequestType::Class,
            recipient: Recipient::Interface,
            request,
            value,
            index: 0,
            length: buffer.len() as u16,
        };
        let mut state = self.dfu_state.borrow_mut();
        match state.control_out(req, buffer) {
            Some(OutResponse::Accepted) => Ok(buffer.len()),
            Some(OutResponse::Rejected) => Err(anyhow::anyhow!("control out rejected")),
            None => Err(anyhow::anyhow!("control out returned None")),
        }
    }

    fn usb_reset(&mut self) -> Result<Self::Reset, Self::Error> {
        Ok(())
    }

    fn protocol(&self) -> &dfu_core::DfuProtocol<Self::MemoryLayout> {
        &dfu_core::DfuProtocol::Dfu
    }

    fn functional_descriptor(&self) -> &dfu_core::functional_descriptor::FunctionalDescriptor {
        &self.functional_descriptor
    }
}

fn usb_dfu(dfu_attributes: DfuAttributes) {
    let mut aligned_buffer = [0; READ_WRITE_SIZE];

    const BLOCK_SIZE: usize = 128;

    let dfu_buffer = RefCell::new([0; { BLOCK_SIZE * 2 }]);
    let dfu_partition = InMemoryFlashPartition { buffer: &dfu_buffer };
    let state_buffer = RefCell::new([0; { READ_WRITE_SIZE * 2 }]);
    let state_partition = InMemoryFlashPartition { buffer: &state_buffer };
    let fw_config = FirmwareUpdaterConfig {
        dfu: dfu_partition,
        state: state_partition,
    };
    let updater = BlockingFirmwareUpdater::new(fw_config, &mut aligned_buffer);

    let functional_descriptor = dfu_core::functional_descriptor::FunctionalDescriptor {
        can_download: true,
        can_upload: false,
        manifestation_tolerant: dfu_attributes.contains(DfuAttributes::MANIFESTATION_TOLERANT),
        will_detach: dfu_attributes.contains(DfuAttributes::WILL_DETACH),
        detach_timeout: 10,
        transfer_size: READ_WRITE_SIZE as u16,
        dfu_version: (1, 1),
    };
    let dfu_state = new_state::<_, _, _, BLOCK_SIZE>(updater, dfu_attributes, NoopReset {});
    let mut dfu = dfu_core::sync::DfuSync::new(InMemoryDfu {
        functional_descriptor,
        dfu_state: RefCell::new(dfu_state),
    });

    let firmware = [42; BLOCK_SIZE];
    let err = dfu.download_from_slice(&firmware);
    println!("{:?}", err);
    assert_eq!(&dfu_buffer.borrow()[..firmware.len()], firmware);
    assert!(err.is_ok());
}

#[test]
fn test_usb_dfu_manifestation_tolerant_will_detach() {
    usb_dfu(DfuAttributes::CAN_DOWNLOAD | DfuAttributes::MANIFESTATION_TOLERANT | DfuAttributes::WILL_DETACH);
}

#[test]
fn test_usb_dfu_manifestation_tolerant() {
    usb_dfu(DfuAttributes::CAN_DOWNLOAD | DfuAttributes::MANIFESTATION_TOLERANT);
}

#[test]
fn test_usb_dfu_will_detach() {
    usb_dfu(DfuAttributes::CAN_DOWNLOAD | DfuAttributes::WILL_DETACH);
}

#[test]
fn test_usb_dfu() {
    usb_dfu(DfuAttributes::CAN_DOWNLOAD);
}
