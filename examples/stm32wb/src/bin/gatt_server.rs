#![no_std]
#![no_main]

use bt_hci::cmd::controller_baseband::Reset;
use bt_hci::cmd::le::LeSetScanResponseData;
use bt_hci::controller::{Controller, ControllerCmdSync};
use bt_hci::event::{Event, EventKind, EventPacket};
use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::ipcc::{Config, ReceiveInterruptHandler, TransmitInterruptHandler};
use embassy_stm32::rcc::Config as RccConfig;
use embassy_stm32_wpan::TlMbox;
use embassy_stm32_wpan::lhci::LhciC1DeviceInformationCcrp;
use embassy_stm32_wpan::sub::ble::ControllerAdapter;
use embassy_stm32_wpan::sub::mm;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Sender};
use panic_probe as _;
use static_cell::StaticCell;
use stm32wb_hci::aci::AciEvent;
use stm32wb_hci::aci::durations::{AdvInterval, PreferredConnInterval};
use stm32wb_hci::aci::flags::{CharProperties, GattEventMask, Role, SecurityPermissions};
use stm32wb_hci::aci::gap::{GapInit, GapSetAuthenticationRequirement, GapSetDiscoverable, GapSetIoCapability};
use stm32wb_hci::aci::gatt::{
    GattAddChar, GattAddService, GattInit, GattPermitRead, GattPermitWrite, GattUpdateCharValue,
};
use stm32wb_hci::aci::hal::{HalSetTxPowerLevel, HalWriteConfigData};
use stm32wb_hci::aci::ranges::{AttAppError, EncKeySize, PaLevel, Passkey};
use stm32wb_hci::aci::values::{
    AddressType, AdvertisingType, ConfigDataOffset, IoCapability, OwnAddressType, PermitStatus, Privacy, ScSupport,
    ServiceType, UseFixedPin,
};
use stm32wb_hci::adv_data::{AdvData, local_name_structure};
use stm32wb_hci::event::BleEvent;
use stm32wb_hci::wire::Uuid;

bind_interrupts!(struct Irqs{
    IPCC_C1_RX => ReceiveInterruptHandler;
    IPCC_C1_TX => TransmitInterruptHandler;
});

const BLE_GAP_DEVICE_NAME_LENGTH: u8 = 7;

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
    /*
        How to make this work:

        - Obtain a NUCLEO-STM32WB55 from your preferred supplier.
        - Run the `fus_update` example: it installs the FUS and the wireless stack on its own,
          no external tool needed.
        - Run this example.

        Note: extended stack versions are not supported at this time. Do not attempt to install a stack with "extended" in the name.
    */

    static EVENT_CHANNEL: StaticCell<Channel<CriticalSectionRawMutex, OwnedEvent, 3>> = StaticCell::new();
    static CONTROLLER: StaticCell<ControllerAdapter<'static>> = StaticCell::new();

    let mut config = embassy_stm32::Config::default();
    config.rcc = RccConfig::new_wpan();
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let config = Config::default();
    let (ble, mm) = TlMbox::wait_ready(p.IPCC, Irqs, config)
        .await
        .unwrap()
        .init_ble(Default::default())
        .await
        .unwrap();

    let event_channel = EVENT_CHANNEL.init(Channel::new());
    let ble = CONTROLLER.init(ControllerAdapter::new(ble));
    let event_sender = event_channel.sender();
    let event_receiver = event_channel.receiver();

    spawner.spawn(run_mm_queue(mm).unwrap());
    // Events must be read for commands to complete.
    spawner.spawn(receive_events(ble, event_sender).unwrap());

    info!("resetting BLE...");
    let response = ble.exec(&Reset::new()).await;
    defmt::debug!("{}", response);

    info!("config public address...");
    let address = get_bd_addr();
    let command = unwrap!(HalWriteConfigData::entry(ConfigDataOffset::PublicAddress, &address));
    let response = ble.exec(&command).await;
    defmt::debug!("{}", response);

    info!("config random address...");
    let address = get_random_addr();
    let command = unwrap!(HalWriteConfigData::entry(
        ConfigDataOffset::StaticRandomAddress,
        &address
    ));
    let response = ble.exec(&command).await;
    defmt::debug!("{}", response);

    info!("config identity root...");
    let command = unwrap!(HalWriteConfigData::entry(ConfigDataOffset::IdentityRoot, &BLE_CFG_IRK));
    let response = ble.exec(&command).await;
    defmt::debug!("{}", response);

    info!("config encryption root...");
    let command = unwrap!(HalWriteConfigData::entry(
        ConfigDataOffset::EncryptionRoot,
        &BLE_CFG_ERK
    ));
    let response = ble.exec(&command).await;
    defmt::debug!("{}", response);

    info!("config tx power level...");
    // PA level 0x19 is 0 dBm.
    let response = ble
        .exec(&HalSetTxPowerLevel::new(false, unwrap!(PaLevel::new(0x19))))
        .await;
    defmt::debug!("{}", response);

    info!("GATT init...");
    let response = ble.exec(&GattInit::new()).await;
    defmt::debug!("{}", response);

    info!("GAP init...");
    let response = ble
        .exec(&GapInit::new(
            Role::PERIPHERAL,
            Privacy::Disabled,
            BLE_GAP_DEVICE_NAME_LENGTH,
        ))
        .await;
    defmt::debug!("{}", response);

    info!("set IO capabilities...");
    let response = ble.exec(&GapSetIoCapability::new(IoCapability::DisplayYesNo)).await;
    defmt::debug!("{}", response);

    info!("set authentication requirements...");
    let response = ble
        .exec(&GapSetAuthenticationRequirement::new(
            false, // bonding
            false, // MITM protection
            ScSupport::Optional,
            false, // keypress notifications
            8,
            16,
            UseFixedPin::No,
            Passkey::MIN,
            AddressType::Public,
        ))
        .await;
    defmt::debug!("{}", response);

    info!("set scan response data...");
    let scan_rsp = unwrap!(AdvData::<31>::new().complete_local_name(b"TXTX"));
    let mut data = [0; 31];
    data[..scan_rsp.as_bytes().len()].copy_from_slice(scan_rsp.as_bytes());
    let response = ble
        .exec(&LeSetScanResponseData::new(scan_rsp.as_bytes().len() as u8, data))
        .await;
    defmt::debug!("{}", response);

    defmt::info!("initializing services and characteristics...");
    let ble_context = init_gatt_services(ble).await;
    defmt::info!("{}", ble_context);

    let mut ble_context = ble_context.unwrap();

    let local_name = unwrap!(local_name_structure::<8>(b"TXTX", true));
    let interval = unwrap!(AdvInterval::from_millis(100));
    let set_discoverable = unwrap!(GapSetDiscoverable::try_new(
        AdvertisingType::ConnectableUndirected,
        interval,
        interval,
        OwnAddressType::Public,
        0, // no filter accept list
        local_name.as_bytes(),
        &[],
        PreferredConnInterval::OMITTED,
        PreferredConnInterval::OMITTED,
    ));

    info!("set discoverable...");
    let response = ble.exec(&set_discoverable).await;
    defmt::debug!("{}", response);

    loop {
        let event = event_receiver.receive().await;
        let event = match event.decode() {
            Ok(event) => event,
            Err(e) => {
                defmt::warn!("undecodable event: {}", e);
                continue;
            }
        };
        defmt::debug!("{}", event);

        match event {
            BleEvent::Core(Event::Le(bt_hci::event::le::LeEvent::LeConnectionComplete(_))) => {
                defmt::info!("connected");
            }
            BleEvent::Core(Event::DisconnectionComplete(_)) => {
                defmt::info!("disconnected");
                ble_context.is_subscribed = false;
                ble.exec(&set_discoverable).await.unwrap();
            }
            BleEvent::Vendor(AciEvent::GattReadPermitReq(read_req)) => {
                defmt::info!("read request received {}, allowing", read_req);
                ble.exec(&GattPermitRead::new(
                    read_req.connection_handle,
                    PermitStatus::Allowed,
                    AttAppError::MIN,
                    read_req.attribute_handle,
                ))
                .await
                .unwrap();
            }
            BleEvent::Vendor(AciEvent::GattWritePermitReq(write_req)) => {
                defmt::info!("write request received {}, allowing", write_req);
                let response = unwrap!(GattPermitWrite::try_new(
                    write_req.connection_handle,
                    write_req.attribute_handle,
                    PermitStatus::Allowed,
                    0,
                    write_req.data,
                ));
                ble.exec(&response).await.unwrap()
            }
            BleEvent::Vendor(AciEvent::GattAttributeModified(attribute)) => {
                defmt::info!("{}", ble_context);
                if attribute.attr_handle == ble_context.chars.notify + 2 {
                    if attribute.attr_data[0] == 0x01 {
                        defmt::info!("subscribed");
                        ble_context.is_subscribed = true;
                    } else {
                        defmt::info!("unsubscribed");
                        ble_context.is_subscribed = false;
                    }
                }
            }
            _ => {}
        }
    }
}

/// An HCI event, copied out of the controller's buffer to cross the channel.
pub struct OwnedEvent {
    kind: EventKind,
    data: heapless::Vec<u8, 255>,
}

impl OwnedEvent {
    fn decode(&self) -> Result<BleEvent<'_>, bt_hci::FromHciBytesError> {
        BleEvent::from_packet(EventPacket {
            kind: self.kind,
            data: &self.data,
        })
    }
}

#[embassy_executor::task]
async fn receive_events(
    controller: &'static ControllerAdapter<'static>,
    event_sender: Sender<'static, CriticalSectionRawMutex, OwnedEvent, 3>,
) {
    loop {
        let mut buf = unwrap!(controller.alloc_buf());
        match controller.read(&mut buf).await {
            Ok(bt_hci::ControllerToHostPacket::Event(packet)) => {
                let mut data = heapless::Vec::new();
                unwrap!(data.extend_from_slice(packet.data));
                event_sender
                    .send(OwnedEvent {
                        kind: packet.kind,
                        data,
                    })
                    .await;
            }
            Ok(packet) => defmt::debug!("{}", packet),
            Err(e) => defmt::warn!("read failed: {}", e),
        }
    }
}

#[embassy_executor::task]
async fn run_mm_queue(mut memory_manager: mm::MemoryManager<'static>) {
    memory_manager.run_queue().await;
}

fn get_bd_addr() -> [u8; 6] {
    let mut bytes = [0u8; 6];

    let lhci_info = LhciC1DeviceInformationCcrp::new();
    bytes[0] = (lhci_info.uid64 & 0xff) as u8;
    bytes[1] = ((lhci_info.uid64 >> 8) & 0xff) as u8;
    bytes[2] = ((lhci_info.uid64 >> 16) & 0xff) as u8;
    bytes[3] = lhci_info.device_type_id;
    bytes[4] = (lhci_info.st_company_id & 0xff) as u8;
    bytes[5] = (lhci_info.st_company_id >> 8 & 0xff) as u8;

    bytes
}

fn get_random_addr() -> [u8; 6] {
    let mut bytes = [0u8; 6];

    let lhci_info = LhciC1DeviceInformationCcrp::new();
    bytes[0] = (lhci_info.uid64 & 0xff) as u8;
    bytes[1] = ((lhci_info.uid64 >> 8) & 0xff) as u8;
    bytes[2] = ((lhci_info.uid64 >> 16) & 0xff) as u8;
    bytes[3] = 0;
    bytes[4] = 0x6E;
    bytes[5] = 0xED;

    bytes
}

const BLE_CFG_IRK: [u8; 16] = [
    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0,
];
const BLE_CFG_ERK: [u8; 16] = [
    0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65, 0x43, 0x21, 0xfe, 0xdc, 0xba, 0x09, 0x87, 0x65, 0x43, 0x21,
];

#[derive(defmt::Format)]
pub struct BleContext {
    pub service_handle: u16,
    pub chars: CharHandles,
    pub is_subscribed: bool,
}

#[derive(defmt::Format)]
pub struct CharHandles {
    pub read: u16,
    pub write: u16,
    pub notify: u16,
}

pub async fn init_gatt_services<'a>(controller: &ControllerAdapter<'a>) -> Result<BleContext, ()> {
    let service_handle = gatt_add_service(controller, Uuid::Uuid16(0x500)).await?;

    let read = gatt_add_char(
        controller,
        service_handle,
        Uuid::Uuid16(0x501),
        CharProperties::READ,
        Some(b"Hello from embassy!"),
    )
    .await?;

    let write = gatt_add_char(
        controller,
        service_handle,
        Uuid::Uuid16(0x502),
        CharProperties::WRITE_WITHOUT_RESPONSE | CharProperties::WRITE | CharProperties::READ,
        None,
    )
    .await?;

    let notify = gatt_add_char(
        controller,
        service_handle,
        Uuid::Uuid16(0x503),
        CharProperties::NOTIFY | CharProperties::READ,
        None,
    )
    .await?;

    Ok(BleContext {
        service_handle,
        is_subscribed: false,
        chars: CharHandles { read, write, notify },
    })
}

async fn gatt_add_service<'a>(controller: &ControllerAdapter<'a>, uuid: Uuid) -> Result<u16, ()> {
    let response = controller
        .exec(&GattAddService::new(uuid, ServiceType::Primary, 8))
        .await;
    defmt::debug!("{}", response);

    response.map(|r| r.service_handle).map_err(|_| ())
}

async fn gatt_add_char<'a>(
    controller: &ControllerAdapter<'a>,
    service_handle: u16,
    characteristic_uuid: Uuid,
    characteristic_properties: CharProperties,
    default_value: Option<&[u8]>,
) -> Result<u16, ()> {
    let response = controller
        .exec(&GattAddChar::new(
            service_handle,
            characteristic_uuid,
            32,
            characteristic_properties,
            SecurityPermissions::empty(),
            GattEventMask::all(),
            unwrap!(EncKeySize::new(7)),
            true,
        ))
        .await;
    defmt::debug!("{}", response);

    let characteristic_handle = response.map_err(|_| ())?.char_handle;
    if let Some(value) = default_value {
        let command = unwrap!(GattUpdateCharValue::try_new(
            service_handle,
            characteristic_handle,
            0,
            value
        ));
        let response = controller.exec(&command).await;
        defmt::debug!("{}", response);
    }
    Ok(characteristic_handle)
}
