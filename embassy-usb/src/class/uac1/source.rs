use core::marker::PhantomData;
use core::usize;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::{PubSubChannel, Publisher, Subscriber};
use embassy_usb_driver::{EndpointAddress, EndpointIn, EndpointOut};
use heapless::Vec;

use super::class_codes::*;
use super::terminal_type::TerminalType;
use super::{ChannelConfig, SampleWidth};
use crate::builder::InterfaceAltBuilder;
use crate::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use crate::descriptor::{SynchronizationType, UsageType};
use crate::driver::{Driver, Endpoint, EndpointError, EndpointType};
use crate::types::InterfaceNumber;
use crate::{Builder, Handler};

/// Parameters of the sample rate channel
const SR_CH_CAP: usize = 4;
const SR_CH_SUBS: usize = 4;
const SR_CH_PUBS: usize = 4;

type SampleRateChannel = PubSubChannel<CriticalSectionRawMutex, u32, SR_CH_CAP, SR_CH_SUBS, SR_CH_PUBS>;
type SampleRatePub = Publisher<'static, CriticalSectionRawMutex, u32, SR_CH_CAP, SR_CH_PUBS, SR_CH_PUBS>;
type SampleRateSub = Subscriber<'static, CriticalSectionRawMutex, u32, SR_CH_CAP, SR_CH_SUBS, SR_CH_PUBS>;

/// Channel for sharing new sample rates
static SAMPLE_RATE_CHANNEL: SampleRateChannel = SampleRateChannel::new();

/// API for acces to the channel's publisher
fn sample_rate_publisher() -> SampleRatePub {
    SAMPLE_RATE_CHANNEL.publisher().unwrap()
}

/// API for acces to the channel's subscriber
pub fn sample_rate_subscriber() -> SampleRateSub {
    SAMPLE_RATE_CHANNEL.subscriber().unwrap()
}

/// Arbitrary unique identifier for the input unit.
const INPUT_UNIT_ID: u8 = 0x01;

/// Arbitrary unique identifier for the feature unit.
const FEATURE_UNIT_ID: u8 = 0x02;

/// Arbitrary unique identifier for the output unit.
const OUTPUT_UNIT_ID: u8 = 0x03;

/// Audio channel count in stream
const MAX_AUDIO_CHANNEL_COUNT: usize = 0x02;

// Maximum number of supported discrete sample rates.
fn calculate_max_packet_size(sample_rate_hz: u32, num_channels: u8, b_subframe_size: u8) -> u16 {
    let bytes_per_ms = (sample_rate_hz * num_channels as u32 * b_subframe_size as u32) / 1000;

    debug!(
        "calculate_max_packet_size: {}Hz × {}ch × {}bytes = {} bytes/ms",
        sample_rate_hz, num_channels, b_subframe_size, bytes_per_ms
    );

    bytes_per_ms as u16
}

/// Used for writing sample rate information over the feedback endpoint.
/// Look at In\Out directions from the host side.
pub struct AudioSourceEpIn<'d, D: Driver<'d>> {
    ep: D::EndpointIn,
}

impl<'d, D: Driver<'d>> AudioSourceEpIn<'d, D> {
    /// Write feedback data to the endpoint.
    pub async fn write(&mut self, buf: &[u8]) -> Result<(), EndpointError> {
        self.ep.write(buf).await
    }

    /// Write all the data from buf to the endpoint one wMaxPacketSize chunk at a time.
    pub async fn write_as_chunks(&mut self, buf: &[u8], needs_zlp: bool) -> Result<(), EndpointError> {
        self.ep.write_transfer(buf, needs_zlp).await
    }

    /// Wait until the endpoint is enabled by the host (i.e., after the host sets the alternate setting with this endpoint). This is critical to call before writing to the endpoint, otherwise writes will fail with EndpointError::Disabled. The endpoint will be disabled again when the host deactivates the streaming interface or unconfigures the device.
    pub async fn wait_enabled(&mut self) {
        self.ep.wait_enabled().await
    }
}

/// Used for reading audio data from the host to the device.
pub struct AudioSourceEpOut<'d, D: Driver<'d>> {
    ep: D::EndpointOut,
}
impl<'d, D: Driver<'d>> AudioSourceEpOut<'d, D> {
    /// Read audio data from the endpoint.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, EndpointError> {
        self.ep.read(buffer).await
    }

    /// Wait until the endpoint is enabled by the host (i.e., after the host sets the alternate setting with this endpoint). This is critical to call before writing to the endpoint, otherwise writes will fail with EndpointError::Disabled. The endpoint will be disabled again when the host deactivates the streaming interface or unconfigures the device.
    pub async fn wait_enabled(&mut self) {
        self.ep.wait_enabled().await
    }
}

/// Implementation of the Audio Source interface
pub struct AudioSource<'d, D: Driver<'d>> {
    phantom: PhantomData<&'d D>,
}

impl<'d, D: Driver<'d>> AudioSource<'d, D> {
    /// Create the Audio Control interface descriptors
    fn create_control_function(
        b: &mut InterfaceAltBuilder<'_, 'd, D>,
        streaming_interface: u8,
        terminal_type: Option<TerminalType>,
    ) {
        // USB Device Class Definition for Audio Devices
        // 4.3.2.1 Input Terminal Descriptor
        let mut w_terminal_type: u16 = TerminalType::InMicrophone.into();
        if terminal_type.is_some() {
            w_terminal_type = terminal_type.unwrap().into();
        }

        let channels_cfg: u16 = ChannelConfig::LeftFront as u16 | ChannelConfig::RightFront as u16;
        let input_terminal_descriptor: [u8; 10] = [
            INPUT_TERMINAL,                // bDescriptorSubtype
            INPUT_UNIT_ID,                 // bTerminalID
            w_terminal_type as u8,         // wTerminalType[0]
            (w_terminal_type >> 8) as u8,  // wTerminalType[1]
            0x00,                          // bAssocTerminal (none)
            MAX_AUDIO_CHANNEL_COUNT as u8, // bNrChannels
            channels_cfg as u8,            // wChannelConfig[0]
            (channels_cfg >> 8) as u8,     // wChannelConfig[1]
            0x00,                          // iChannelNames (none)
            0x00,                          // iTerminal (none)
        ];

        // USB Device Class Definition for Audio Devices
        // 4.3.2.5 Feature Unit Descriptor
        let controls: u8 = MUTE_CONTROL | VOLUME_CONTROL;
        const FEATURE_UNIT_DESCRIPTOR_SIZE: usize = 5;
        let mut feature_unit_descriptor: Vec<u8, { FEATURE_UNIT_DESCRIPTOR_SIZE + MAX_AUDIO_CHANNEL_COUNT + 1 }> =
            Vec::from_slice(&[
                FEATURE_UNIT,    // bDescriptorSubtype (Feature Unit)
                FEATURE_UNIT_ID, // bUnitID
                INPUT_UNIT_ID,   // bSourceID
                1,               // bControlSize (one byte per control)
                controls,        // Master controls for Mute and Volume
            ])
            .unwrap();

        // Add per-channel controls (no controls for individual channels, only master channel has controls)
        for _ in 0..MAX_AUDIO_CHANNEL_COUNT {
            feature_unit_descriptor.push(controls).unwrap();
        }
        feature_unit_descriptor.push(0x00).unwrap(); // iFeature (none)

        // USB Device Class Definition for Audio Devices
        // 4.3.2.2 Output Terminal Descriptor
        let terminal_type: u16 = TerminalType::UsbStreaming.into();
        let output_terminal_descriptor = [
            OUTPUT_TERMINAL,            // bDescriptorSubtype
            OUTPUT_UNIT_ID,             // bTerminalID
            terminal_type as u8,        // wTerminalType[0]
            (terminal_type >> 8) as u8, // wTerminalType[1]
            0x00,                       // bAssocTerminal (none)
            FEATURE_UNIT_ID,            // bSourceID (directly from the input terminal, no feature unit in between)
            0x00,                       // iTerminal (none)
        ];

        // USB Device Class Definition for Audio Devices
        // 4.3.2 Class-Specific AC Interface Descriptor
        const AC_HEADER_SIZE: usize = 2; // bLength + bDescriptorType
        const INTERFACE_DESCRIPTOR_SIZE: usize = 7;
        let mut total_descriptor_length: usize = 0;

        for size in [
            INTERFACE_DESCRIPTOR_SIZE,
            input_terminal_descriptor.len(),
            feature_unit_descriptor.len(),
            output_terminal_descriptor.len(),
        ] {
            total_descriptor_length += size + AC_HEADER_SIZE;
        }

        let interface_descriptor: [u8; INTERFACE_DESCRIPTOR_SIZE] = [
            HEADER_SUBTYPE,                       // bDescriptorSubtype (Header)
            ADC_VERSION as u8,                    // bcdADC[0]
            (ADC_VERSION >> 8) as u8,             // bcdADC[1]
            total_descriptor_length as u8,        // wTotalLength[0]
            (total_descriptor_length >> 8) as u8, // wTotalLength[1]
            0x01,                                 // bInCollection (1 streaming interface)
            streaming_interface,                  // baInterfaceNr
        ];

        b.descriptor(CS_INTERFACE, &interface_descriptor);
        b.descriptor(CS_INTERFACE, &input_terminal_descriptor);
        b.descriptor(CS_INTERFACE, &feature_unit_descriptor);
        b.descriptor(CS_INTERFACE, &output_terminal_descriptor);
    }

    fn create_streaming_iface_active(
        b: &mut InterfaceAltBuilder<'_, 'd, D>,
        sample_rates: &[u32],
        sample_width: SampleWidth,
        feedback_refresh_period_ms: u8,
    ) -> (<D as Driver<'d>>::EndpointIn, <D as Driver<'d>>::EndpointIn) {
        b.descriptor(
            CS_INTERFACE,
            &[
                AS_GENERAL,       // bDescriptorSubtype
                OUTPUT_UNIT_ID,   // bTerminalLink
                0x01,             // bDelay (1 frame)
                PCM as u8,        // wFormatTag[0]: LSB of PCM (0x0001)
                (PCM >> 8) as u8, // wFormatTag[1]: MSB
            ],
        );

        // Determine min and max sample rates
        let min_rate = sample_rates.iter().min().unwrap();
        let max_rate = sample_rates.iter().max().unwrap();

        // Format Type I Descriptor (UAC1, 4.5.1) for one sample rate (e.g., 16 kHz)
        // Total Length: 11 bytes (8 + 3 + 2 header). Body: 9 bytes.
        let format_type_i_body: [u8; 12] = [
            FORMAT_TYPE,                     // bDescriptorSubtype: FORMAT_TYPE (0x02)
            FORMAT_TYPE_I,                   // bFormatType: FORMAT_TYPE_I (0x01)
            MAX_AUDIO_CHANNEL_COUNT as u8,   // bNrChannels
            sample_width as u8,              // bSubframeSize
            sample_width.in_bit() as u8,     // bBitResolution
            0x00,                            // bSamFreqType: 0 - continues range
            (min_rate & 0xFF) as u8,         // tSamFreq[0]: LSB
            ((min_rate >> 8) & 0xFF) as u8,  // tSamFreq[1]
            ((min_rate >> 16) & 0xFF) as u8, // tSamFreq[2]
            (max_rate & 0xFF) as u8,         // tSamFreq[0]: LSB
            ((max_rate >> 8) & 0xFF) as u8,  // tSamFreq[1]
            ((max_rate >> 16) & 0xFF) as u8, // tSamFreq[2]
        ];
        b.descriptor(CS_INTERFACE, &format_type_i_body);

        // ALLOCATE the Isochronous IN Endpoint for Audio Data (AudioStream -> Host)
        let max_packet_size: u16 =
            calculate_max_packet_size(*max_rate as u32, MAX_AUDIO_CHANNEL_COUNT as u8, sample_width as u8);
        let audio_in_endpoint: <D as Driver<'d>>::EndpointIn = b.alloc_endpoint_in(
            EndpointType::Isochronous, // Endpoint type
            None,                      // Specific address (None lets the driver assign it, e.g., 0x81)
            max_packet_size,           // wMaxPacketSize
            1,                         // bInterval (1 ms for Full-Speed)
        );

        debug!(
            "AudioStream: Audio endpoint allocated: addr={:?}, type={:?}, max_packet_size={}, interval={} ",
            audio_in_endpoint.info().addr,
            audio_in_endpoint.info().ep_type,
            audio_in_endpoint.info().max_packet_size,
            audio_in_endpoint.info().interval_ms,
        );

        // ALLOCATE the optional Isochronous IN Endpoint for Feedback (AudioStream -> Host)
        let feedback_in_endpoint = b.alloc_endpoint_in(
            EndpointType::Isochronous,
            None,
            4,                          // Feedback packets are 3 bytes, rounded to 4.
            feedback_refresh_period_ms, // Feedback interval - CRITICAL: Must match bRefresh!
        );

        debug!(
            "Feedback endpoint allocated: addr={:?}, type={:?}, max_packet_size={}, interval={}",
            feedback_in_endpoint.info().addr,
            feedback_in_endpoint.info().ep_type,
            feedback_in_endpoint.info().max_packet_size,
            feedback_in_endpoint.info().interval_ms,
        );

        // Write the STANDARD Descriptor for the AUDIO IN endpoint.
        b.endpoint_descriptor(
            audio_in_endpoint.info(),
            SynchronizationType::Asynchronous,
            UsageType::DataEndpoint,
            &[
                feedback_refresh_period_ms,              // bRefresh
                feedback_in_endpoint.info().addr.into(), // bSynchAddress: Links to feedback EP
            ],
        );

        // Write the Class-Specific Descriptor for the AUDIO endpoint (EP_GENERAL)
        b.descriptor(
            CS_ENDPOINT,
            &[
                EP_GENERAL, // bDescriptorSubtype
                0x01,       // bmAttributes: 0x01 use sampling frequency control / No control (0x00). .
                0x02,       // bLockDelayUnits: PCM samples (0x02) / Undefined (0x00)
                0x00,       // wLockDelay[0]
                0x00,       // wLockDelay[1]
            ],
        );

        // Write the STANDARD Descriptor for the FEEDBACK IN endpoint.
        b.endpoint_descriptor(
            feedback_in_endpoint.info(),
            SynchronizationType::NoSynchronization,
            UsageType::FeedbackEndpoint,
            &[], // No extra bytes for this standard descriptor.
        );

        (audio_in_endpoint, feedback_in_endpoint)
    }

    /// Create a new Audio Source interface with control and streaming endpoints    
    pub fn new(
        b: &mut Builder<'d, D>,
        sample_rates: &'static [u32],
        sample_width: SampleWidth,
        fedback_refresh_period_ms: u8,
        terminal_type: Option<TerminalType>,
    ) -> (
        AudioSourceEpIn<'d, D>,
        AudioSourceEpIn<'d, D>,
        AudioSourceControlHandler,
    ) {
        // Create the Audio Function (IAD groups IF0 & IF1)
        let mut func = b.function(USB_AUDIO_CLASS, USB_AUDIOCONTROL_SUBCLASS, PROTOCOL_NONE);

        // Create Audio Control Interface (IF 0) - Single Alt Setting
        let mut iface_ctrl = func.interface(); // This is Interface 0
        let iface_ctrl_num = iface_ctrl.interface_number();
        let ba_iface_nr = u8::from(iface_ctrl_num) + 1;
        {
            let mut alt_ctrl = iface_ctrl.alt_setting(USB_AUDIO_CLASS, USB_AUDIOCONTROL_SUBCLASS, PROTOCOL_NONE, None);
            Self::create_control_function(&mut alt_ctrl, ba_iface_nr.into(), terminal_type);
        }
        // Create Audio Source Interface (IF 1) - TWO Alternate Settings
        // Alternate Setting 1: ACTIVE with Isochronous IN Endpoint
        // Host selects this when ready to receive audio data
        let mut iface_stream = func.interface();
        let iface_stream_num = iface_stream.interface_number();

        // Alternate Setting 0 (INACTIVE - Zero Bandwidth)
        let alt_setting = iface_stream.alt_setting(USB_AUDIO_CLASS, USB_AUDIOSTREAMING_SUBCLASS, PROTOCOL_NONE, None);
        drop(alt_setting);

        // Alternate Setting 1 (ACTIVE - With Endpoints)
        let mut alt_setting =
            iface_stream.alt_setting(USB_AUDIO_CLASS, USB_AUDIOSTREAMING_SUBCLASS, PROTOCOL_NONE, None);

        let (ep_audio_in, ep_feedback_in) = Self::create_streaming_iface_active(
            &mut alt_setting,
            sample_rates,
            sample_width,
            fedback_refresh_period_ms,
        );

        let ep_audio_addr = ep_audio_in.info().addr;
        let ep_feedback_addr = ep_feedback_in.info().addr;

        (
            AudioSourceEpIn { ep: ep_audio_in },
            AudioSourceEpIn { ep: ep_feedback_in },
            AudioSourceControlHandler::new(
                sample_rates,
                ep_audio_addr,
                ep_feedback_addr,
                iface_ctrl_num,
                iface_stream_num,
            ),
        )
    }
}

/// Implementation of the logic for Audio Control requests (e.g., volume, mute)
pub struct AudioSourceControlHandler {
    current_volume: [i16; 3],               // [Master, Left, Right] in 1/256 dB units
    current_mute: [u8; 3],                  // 0 = unmuted, 1 = muted
    current_sample_rate_index: usize,       // Current sample rate array index
    supported_sample_rates: &'static [u32], // Supported sample rates for feedback endpoint
    sample_rate_ch_pub: SampleRatePub,
    ep_audio_addr: EndpointAddress,
    ep_feedback_addr: EndpointAddress,
    iface_ctrl_num: InterfaceNumber,
    iface_stream_num: InterfaceNumber,
}

impl AudioSourceControlHandler {
    /// Create a new AudioSourceControlHandler
    pub fn new(
        sample_rates: &'static [u32],
        ep_audio_addr: EndpointAddress,
        ep_feedback_addr: EndpointAddress,
        iface_ctrl_num: InterfaceNumber,
        iface_stream_num: InterfaceNumber,
    ) -> Self {
        let obj = AudioSourceControlHandler {
            current_volume: [0, 0, 0],
            current_mute: [0, 0, 0],
            current_sample_rate_index: 0, // Default to the first supported sample rate index
            supported_sample_rates: sample_rates,
            sample_rate_ch_pub: sample_rate_publisher(),
            ep_audio_addr,
            ep_feedback_addr,
            iface_ctrl_num,
            iface_stream_num,
        };
        obj
    }

    fn handle_control_in<'r>(&'r mut self, req: Request, data: &'r mut [u8]) -> Option<InResponse<'r>> {
        debug!(
            "AudioSourceControlHandler::handle_control_in(req:{:?}, data:{:?})",
            req, data
        );

        // Only handle class-specific requests for the Audio Control interface
        if req.request_type != RequestType::Class || req.recipient != Recipient::Interface {
            error!("AudioSourceControlHandler: Unsupported request type/recipient/index");
            return Some(InResponse::Rejected);
        }

        let interface_num = (req.index & 0xFF) as u8;
        if interface_num != 0 {
            error!("Request for wrong interface: {}", interface_num);
            return Some(InResponse::Rejected);
        }

        let control_selector = (req.value >> 8) as u8;
        let channel = (req.value & 0xFF) as usize;

        debug!(
            "AudioSourceControlHandler::control_selector: {}, channel: {}",
            control_selector, channel
        );

        match req.request {
            // Get current value
            GET_CUR => {
                if control_selector == VOLUME_CONTROL {
                    // Volume control
                    if channel < 3 {
                        let vol = self.current_volume[channel];
                        data[0..2].copy_from_slice(&vol.to_le_bytes());
                        return Some(InResponse::Accepted(&data[0..2]));
                    }
                } else if control_selector == MUTE_CONTROL {
                    // Mute control
                    if channel < 3 {
                        data[0] = self.current_mute[channel];
                        return Some(InResponse::Accepted(&data[0..1]));
                    }
                }
            }
            // Retun MIN/MAX/RES
            GET_MIN | GET_MAX | GET_RES => {
                if control_selector == VOLUME_CONTROL && channel < 3 {
                    let value = match req.request {
                        GET_MIN => -12750i16, // Minimum: -50 dB
                        GET_MAX => 0i16,      // Maximum: 0 dB
                        GET_RES => 256i16,    // Resolution: 1 dB
                        _ => unreachable!(),
                    };

                    data[0..2].copy_from_slice(&value.to_le_bytes());
                    return Some(InResponse::Accepted(&data[0..2]));
                }
            }
            _ => {}
        }
        error!(
            "AudioSourceControlHandler: Unsupported request {:?}. Rejected!",
            req.request
        );
        Some(InResponse::Rejected)
    }

    fn handle_control_out(&mut self, req: Request, data: &[u8]) -> Option<OutResponse> {
        debug!(
            "AudioSourceControlHandler:handle_control_out(req:{:?}, data:{:?})",
            req, data
        );

        // Filter for Class requests to Audio Control interface
        if req.request_type != RequestType::Class || req.recipient != Recipient::Interface {
            error!("AudioSourceControlHandler: Unsupported request type/recipient/index");
            return Some(OutResponse::Rejected);
        }

        let interface_num = (req.index & 0xFF) as u8;
        if interface_num != 0 {
            error!("Request for wrong interface: {}", interface_num);
            return Some(OutResponse::Rejected);
        }

        let control_selector = (req.value >> 8) as u8;
        let channel = (req.value & 0xFF) as usize;

        debug!(
            "AudioSourceControlHandler: control selector: {}, channel: {}",
            control_selector, channel
        );

        match req.request {
            SET_CUR | SET_RES => match control_selector as u8 {
                VOLUME_CONTROL if channel < 3 && data.len() >= 2 => {
                    self.current_volume[channel] = i16::from_le_bytes([data[0], data[1]]);
                    debug!(
                        "AudioSourceControlHandler: Volume set: ch{} = {} (raw)",
                        channel, self.current_volume[channel]
                    );
                    return Some(OutResponse::Accepted);
                }
                MUTE_CONTROL if channel < 3 && data.len() >= 1 => {
                    self.current_mute[channel] = data[0];
                    debug!(
                        "AudioSourceControlHandler: Mute set: ch{} = {}",
                        channel, self.current_mute[channel]
                    );
                    return Some(OutResponse::Accepted);
                }
                _ => {
                    debug!(
                        "AudioSourceControlHandler: Unsupported control selector {:?}. Rejected!",
                        control_selector
                    );
                    return Some(OutResponse::Rejected);
                }
            },
            _ => {
                debug!(
                    "AudioSourceControlHandler: Unsupported request {:#02X}. Rejected!",
                    req.request
                );
                return Some(OutResponse::Rejected);
            }
        }
    }

    fn handle_ep_in<'a>(&mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        // Only handle class-specific requests for the Audio Control interface
        if req.request_type != RequestType::Class {
            error!(
                "AudioSourceControlHandler: Unsupported request type: {:?}",
                req.request_type
            );
            return Some(InResponse::Rejected);
        }

        match req.request {
            GET_CUR => {
                // This is a request from the host to read the current sample rate from the feedback endpoint.
                // We need to check if the request is for our feedback endpoint and return the appropriate data.
                if req.index & 0xFF == u8::from(self.ep_audio_addr) as u16 {
                    debug!(
                        "AudioSourceControlHandler: GET_CUR for endpoint:{:#02X}",
                        req.index & 0xFF
                    );
                    buf[0] = (self.supported_sample_rates[self.current_sample_rate_index] & 0xFF) as u8;
                    buf[1] = (self.supported_sample_rates[self.current_sample_rate_index] >> 8 & 0xFF) as u8;
                    buf[2] = (self.supported_sample_rates[self.current_sample_rate_index] >> 16 & 0xFF) as u8;
                    return Some(InResponse::Accepted(&buf[0..3]));
                } else {
                    error!(
                        "AudioSourceControlHandler: GET_CUR for unknown endpoint: {:#02X}. Rejected!",
                        req.index & 0xFF
                    );
                }
            }
            _ => {
                error!(
                    "AudioSourceControlHandler: Unsupported request: {:#02X}. Rejected!",
                    req.request
                );
            }
        }
        return Some(InResponse::Rejected);
    }

    fn handle_ep_out(&mut self, req: Request, buf: &[u8]) -> Option<OutResponse> {
        if req.request_type != RequestType::Class {
            error!(
                "AudioSourceControlHandler: Unsupported request type: {:?}",
                req.request_type
            );
            return Some(OutResponse::Rejected);
        }

        match req.request {
            SET_CUR => {
                // This is a request from the host to set the current sample rate via the feedback endpoint.
                if req.index & 0xFF == u8::from(self.ep_audio_addr) as u16 {
                    debug!(
                        "AudioSourceControlHandler: SET_CUR for endpoint:{:#02X}, data:{:?}",
                        req.index & 0xFF,
                        buf
                    );

                    let rate = ((buf[0] as u32) | (buf[1] as u32) << 8 | (buf[2] as u32) << 16) as u32;
                    let current_rate = self.supported_sample_rates[self.current_sample_rate_index];

                    if rate == current_rate {
                        debug!("AudioSourceControlHandler: Sample rate unchanged: {} Hz", rate);
                        if let Err(e) = self.sample_rate_ch_pub.try_publish(rate) {
                            error!("Failed to publish sample rate: {:?}", e);
                        }
                        return Some(OutResponse::Accepted);
                    }

                    match self.supported_sample_rates.binary_search(&rate) {
                        Ok(index) => {
                            info!(
                                "AudioSourceControlHandler: Sample rate changed: {} Hz -> {} Hz",
                                current_rate, self.supported_sample_rates[index]
                            );
                            self.current_sample_rate_index = index;
                            if let Err(e) = self.sample_rate_ch_pub.try_publish(rate) {
                                error!("Failed to publish sample rate: {:?}", e);
                            }
                            return Some(OutResponse::Accepted);
                        }
                        Err(_e) => {
                            error!("AudioSourceControlHandler: SET_CUR: unsupported sample rate: {}", rate);
                        }
                    }
                } else {
                    error!(
                        "AudioSourceControlHandler: SET_CUR for unknown endpoint: {:#02X}. Rejected!",
                        req.index & 0xFF
                    );
                }
            }
            _ => {
                error!(
                    "AudioSourceControlHandler: Unsupported request: {:#02X}. Rejected!",
                    req.request
                );
            }
        }

        Some(OutResponse::Rejected)
    }

    /// Gets the audio endpoint address
    pub fn get_audio_ep_addr(&self) -> u8 {
        u8::from(self.ep_audio_addr)
    }

    /// Gets the feedback endpoint address
    pub fn get_feedback_ep_addr(&self) -> u8 {
        u8::from(self.ep_feedback_addr)
    }

    /// Gets the index part of the audio endpoint address
    pub fn get_audio_ep_index(&self) -> usize {
        self.ep_audio_addr.index()
    }

    /// Gets the index part of the feedback endpoint address
    pub fn get_feedback_ep_index(&self) -> usize {
        self.ep_feedback_addr.index()
    }

    /// Gets the bInterfaceNumber of the control interface
    pub fn get_ctrl_iface_num(&self) -> u8 {
        u8::from(self.iface_ctrl_num)
    }

    /// Gets the bInterfaceNumber of the streaming interface
    pub fn get_stream_iface_num(&self) -> u8 {
        u8::from(self.iface_stream_num)
    }
}

impl<'d> Handler for AudioSourceControlHandler {
    /// Called when the host has set the address of the device to `addr`.
    fn addressed(&mut self, addr: u8) {
        debug!("AudioSourceControlHandler: Host set address to: {:#02X}", addr);
    }

    /// Called when the host has enabled or disabled the configuration of the device.
    fn configured(&mut self, configured: bool) {
        debug!("AudioSourceControlHandler: USB device configured: {}", configured);
    }

    /// Called when remote wakeup feature is enabled or disabled.
    fn remote_wakeup_enabled(&mut self, enabled: bool) {
        debug!("AudioSourceControlHandler: USB remote wakeup enabled: {}", enabled);
    }

    /// Called when a "set alternate setting" control request is done on the interface.
    fn set_alternate_setting(&mut self, iface: InterfaceNumber, alternate_setting: u8) {
        debug!(
            "AudioSourceControlHandler: USB set interface({}) to alt-setting({})",
            iface, alternate_setting
        );

        if iface == InterfaceNumber(1) {
            if alternate_setting == 0 {
                info!(
                    "AudioSourceControlHandler: Audio streaming interface({}) deactivated!",
                    iface
                );
            } else if alternate_setting == 1 {
                info!(
                    "AudioSourceControlHandler: Audio streaming interface({}) activated!",
                    iface
                );
            }
        }
    }

    /// Called after a USB reset after the bus reset sequence is complete.
    fn reset(&mut self) {
        debug!("AudioSourceControlHandler: USB device reset");
    }

    /// Called when the bus has entered or exited the suspend state.
    fn suspended(&mut self, suspended: bool) {
        debug!("AudioSourceControlHandler: USB device suspended: {}", suspended);
    }

    /// Called when a control request is received with direction HostToDevice.    
    fn control_out(&mut self, req: Request, buf: &[u8]) -> Option<OutResponse> {
        debug!(
            "AudioSourceControlHandler: USB device control OUT EP({:#02X}): {:?}, Data: {:?}",
            (req.index & 0xFF),
            req,
            buf,
        );

        match req.request_type {
            RequestType::Class => match req.recipient {
                Recipient::Interface => self.handle_control_out(req, buf),
                Recipient::Endpoint => self.handle_ep_out(req, buf),
                _ => None,
            },
            _ => None,
        }
    }

    /// Called when a control request is received with direction DeviceToHost.    
    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        debug!(
            "AudioSourceControlHandler: USB device control IN EP({:#02X}): {:?}, Data: {:?}",
            (req.index & 0xFF),
            req,
            buf,
        );

        match req.request_type {
            RequestType::Class => match req.recipient {
                Recipient::Interface => self.handle_control_in(req, buf),
                Recipient::Endpoint => self.handle_ep_in(req, buf),
                _ => None,
            },
            _ => None,
        }
    }
}
