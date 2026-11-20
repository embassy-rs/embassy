use core::sync::atomic::{Ordering, compiler_fence};

use embassy_net_driver_channel::StateRunner;
use embassy_net_driver_channel::driver::{HardwareAddress, LinkState};

use crate::net::ZeroCopyPubSub;
use crate::net::commands::*;
use crate::net::iface::{Controller, ControllerToHostPacket, ControllerToHostPacketBox, HostToControllerPacket, mlme};
use crate::net::typedefs::*;

pub struct Control<'a, C: Controller> {
    controller: &'a C,

    state_ch: StateRunner<'a>,
    events: &'a ZeroCopyPubSub<C::Packet>,
}

impl<'a, C: Controller> Control<'a, C> {
    pub(crate) const fn new(
        controller: &'a C,
        state_ch: StateRunner<'a>,
        events: &'a ZeroCopyPubSub<C::Packet>,
    ) -> Self {
        Self {
            controller,
            state_ch,
            events,
        }
    }

    pub async fn send_command_and_get_response(
        &self,
        cmd: &impl HostToControllerPacket,
    ) -> Result<C::Packet, MacError> {
        let subscriber = self.events.subscribe();

        compiler_fence(Ordering::Release);

        self.controller.write(cmd).await.map_err(|_e| MacError::Error)?;

        let response = subscriber.wait().await;

        Ok(response)
    }

    pub async fn scan(&mut self) -> Result<(), ()> {
        todo!()
    }

    pub async fn join(&mut self) -> Result<(), ()> {
        todo!()
    }

    pub async fn leave(&mut self) -> Result<(), ()> {
        todo!()
    }

    /// Send a command, wait for the radio's answer, and fail if the answer
    /// is not a success.
    async fn request(&self, cmd: &impl HostToControllerPacket) -> Result<(), MacError> {
        let resp = self.send_command_and_get_response(cmd).await?;

        match resp.packet() {
            ControllerToHostPacket::Mlme(mlme::Packet::Confirm(confirm)) => confirm.result(),
            other => {
                warn!("expected a confirm, got {:?}", other);
                Err(MacError::UnexpectedResponse)
            }
        }
    }

    pub async fn start_ap(&mut self, pan_id: [u8; 2], short_addr: [u8; 2]) -> Result<(), MacError> {
        debug!("resetting");
        self.request(&ResetRequest {
            set_default_pib: true,
            ..Default::default()
        })
        .await?;

        debug!("setting extended address");
        let extended_addr = match self.state_ch.get_hardware_address() {
            HardwareAddress::Ieee802154(addr) => addr,
            _ => unreachable!(),
        };
        self.request(&SetRequest {
            pib_attribute_ptr: &extended_addr as *const _ as *const u8,
            pib_attribute: PibId::ExtendedAddress,
        })
        .await?;

        debug!("setting short address");
        self.request(&SetRequest {
            pib_attribute_ptr: &short_addr as *const _ as *const u8,
            pib_attribute: PibId::ShortAddress,
        })
        .await?;

        debug!("setting association permit");
        let association_permit: bool = true;
        self.request(&SetRequest {
            pib_attribute_ptr: &association_permit as *const _ as *const u8,
            pib_attribute: PibId::AssociationPermit,
        })
        .await?;

        debug!("setting TX power");
        let transmit_power: i8 = 2;
        self.request(&SetRequest {
            pib_attribute_ptr: &transmit_power as *const _ as *const u8,
            pib_attribute: PibId::TransmitPower,
        })
        .await?;

        debug!("starting FFD device");
        self.request(&StartRequest {
            pan_id: PanId(pan_id),
            channel_number: MacChannel::Channel16,
            beacon_order: 0x0F,
            superframe_order: 0x0F,
            pan_coordinator: true,
            battery_life_extension: false,
            ..Default::default()
        })
        .await?;

        debug!("setting RX on when idle");
        let rx_on_while_idle: bool = true;
        self.request(&SetRequest {
            pib_attribute_ptr: &rx_on_while_idle as *const _ as *const u8,
            pib_attribute: PibId::RxOnWhenIdle,
        })
        .await?;

        self.state_ch.set_link_state(LinkState::Up);
        Ok(())
    }

    pub async fn close_ap(&mut self) -> Result<(), ()> {
        todo!()
    }
}
