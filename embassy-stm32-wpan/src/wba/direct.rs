//! Direct HCI command implementations (`direct` feature).
//!
//! By default [`ControllerAdapter`] implements every bt-hci command the same
//! way: it encodes the command as an HCI packet, hands it to
//! `BleStack_Request`, and decodes the reply packet. With the `direct` feature
//! those blanket implementations are disabled and each command instead calls
//! its own function in ST's BLE stack, the way `nrf-sdc` does. For example
//! `LeSetAdvParams` calls `hci_le_set_advertising_parameters`.
//!
//! Only the commands listed here are implemented. A command the ST stack has
//! no function for (or whose bt-hci layout does not match it) fails to compile
//! instead of failing at runtime. With `wb-hci`, the ST vendor (ACI) commands
//! of `stm32wb-hci` are implemented the same way, in [`aci`].
//!
//! # Mapping syntax
//!
//! Each entry lists the arguments of the ST function in HCI wire order, read
//! from the command's serialized parameters `p`:
//!
//! - `p.u8()`, `p.u16()`, `p.u32()`: a 1, 2 or 4 byte value
//! - `p.ptr(n)`: pointer to an `n` byte array (address, key, 24-bit interval)
//! - `p.array_of::<T>(n)`: pointer to `n` ST structs `T`
//! - `p.list(n)`, `p.list_of::<T>()`: pointer to the entries counted by the
//!   previous `p.u8()` or `p.u16()`, each `n` bytes long (or one ST struct `T`)
//! - `p.uuid()`: pointer to a 16-bit or 128-bit UUID, selected by the previous
//!   `p.u8()` (1 or 2)
//! - `p.rest()`: pointer to all remaining bytes
//!
//! Output arguments are taken from `o`: `o.u8()`, `o.u16()`, `o.u32()`,
//! `o.bytes(n)`, `o.array_of::<T>(n)`, and `o.items(n, max)` or
//! `o.items_of::<T>(max)` for up to `max` entries counted by the previous
//! output. After the call their values, in order, form the command's return
//! parameters. `[echo n]` puts the first `n` parameter bytes (the handle) in
//! front of them, for commands that return their handle.
//!
//! In debug builds, the argument list is first evaluated once as a dry run to
//! check that it consumes exactly the serialized parameters. A mapping that
//! does not match the bt-hci layout returns an error without calling the stack.
//! Release builds skip the dry run.

use core::cell::Cell;

use bt_hci::cmd::{self, Cmd, Error as CmdError};
use bt_hci::controller::{ControllerCmdAsync, ControllerCmdSync};
use bt_hci::{FromHciBytes, WriteHci};
use embedded_io::ErrorKind;
use stm32_bindings::ble::*;

use crate::wba::controller::ControllerAdapter;

/// Largest serialized parameter block of an HCI command.
const PARAMS_LEN: usize = 255;

/// Room for output arguments, in 32-bit words so every output is aligned.
const OUT_WORDS: usize = 72;

/// Most output arguments a single command has.
const OUT_FIELDS: usize = 24;

const INVALID: CmdError<ErrorKind> = CmdError::Io(ErrorKind::InvalidData);

/// Reader over a command's serialized parameters.
struct Params<'a> {
    buf: &'a [u8],
    pos: Cell<usize>,
    /// The last `u8` or `u16` read, which counts a following list.
    last: Cell<u16>,
}

impl<'a> Params<'a> {
    fn new<P: WriteHci>(buf: &'a mut [u8; PARAMS_LEN], params: &P) -> Result<Self, CmdError<ErrorKind>> {
        let len = params.size();
        if len > buf.len() {
            return Err(INVALID);
        }
        params.write_hci(&mut buf[..len]).map_err(|_| INVALID)?;
        Ok(Self {
            buf: &buf[..len],
            pos: Cell::new(0),
            last: Cell::new(0),
        })
    }

    /// Takes the next `n` bytes. Past the end it returns an empty slice; the
    /// debug-build dry run catches that before anything is passed to C.
    fn take(&self, n: usize) -> &'a [u8] {
        let start = self.pos.get();
        let end = start.saturating_add(n);
        self.pos.set(end);
        self.buf.get(start..end).unwrap_or(&[])
    }

    fn u8(&self) -> u8 {
        let v = self.take(1).first().copied().unwrap_or(0);
        self.last.set(v.into());
        v
    }

    fn u16(&self) -> u16 {
        let v = match self.take(2) {
            [a, b] => u16::from_le_bytes([*a, *b]),
            _ => 0,
        };
        self.last.set(v);
        v
    }

    #[allow(dead_code, reason = "only some command sets use it")]
    fn u32(&self) -> u32 {
        match self.take(4) {
            [a, b, c, d] => u32::from_le_bytes([*a, *b, *c, *d]),
            _ => 0,
        }
    }

    fn ptr(&self, n: usize) -> *const u8 {
        self.take(n).as_ptr()
    }

    #[allow(dead_code, reason = "only some command sets use it")]
    fn array_of<T>(&self, n: usize) -> *const T {
        self.ptr(n * core::mem::size_of::<T>()).cast()
    }

    fn list(&self, entry_len: usize) -> *const u8 {
        self.ptr(self.last.get() as usize * entry_len)
    }

    /// Like [`Self::list`], with the entry length taken from ST's packed
    /// struct for the entry.
    fn list_of<T>(&self) -> *const T {
        self.list(core::mem::size_of::<T>()).cast()
    }

    #[allow(dead_code, reason = "only some command sets use it")]
    /// A UUID whose type (1: 16-bit, 2: 128-bit) is the previous `p.u8()`.
    fn uuid(&self) -> *const u8 {
        let len = match self.last.get() {
            1 => 2,
            2 => 16,
            _ => 0,
        };
        self.ptr(len)
    }

    fn rest(&self) -> *const u8 {
        self.ptr(self.buf.len().saturating_sub(self.pos.get()))
    }

    /// The `n` parameter bytes at `start`, without consuming them.
    fn bytes_at(&self, start: usize, n: usize) -> &'a [u8] {
        self.buf.get(start..start + n).unwrap_or(&[])
    }

    /// Ends the dry run: checks that every byte was consumed, then rewinds.
    fn rewind(&self) -> bool {
        let exact = self.pos.get() == self.buf.len();
        self.pos.set(0);
        exact
    }
}

/// Storage for a command's output arguments.
///
/// Every output starts on a 4-byte boundary so the C side can write `u16` and
/// `u32` values through aligned pointers.
struct Out {
    base: *mut u8,
    next: Cell<usize>,
    /// Offset, length, and for a counted list its entry length (else 0).
    fields: [Cell<(u16, u16, u16)>; OUT_FIELDS],
    count: Cell<usize>,
    overflow: Cell<bool>,
}

impl Out {
    fn new(storage: &mut [u32; OUT_WORDS]) -> Self {
        Self {
            base: storage.as_mut_ptr().cast(),
            next: Cell::new(0),
            fields: [const { Cell::new((0, 0, 0)) }; OUT_FIELDS],
            count: Cell::new(0),
            overflow: Cell::new(false),
        }
    }

    fn alloc(&self, len: usize, entry_len: usize) -> *mut u8 {
        let offset = self.next.get();
        let count = self.count.get();
        if count == OUT_FIELDS || offset + len > OUT_WORDS * 4 {
            self.overflow.set(true);
            return self.base;
        }
        self.fields[count].set((offset as u16, len as u16, entry_len as u16));
        self.count.set(count + 1);
        self.next.set(offset + len.next_multiple_of(4));
        // SAFETY: `offset + len` is within the storage.
        unsafe { self.base.add(offset) }
    }

    fn u8(&self) -> *mut u8 {
        self.alloc(1, 0)
    }

    fn u16(&self) -> *mut u16 {
        self.alloc(2, 0).cast()
    }

    fn u32(&self) -> *mut u32 {
        self.alloc(4, 0).cast()
    }

    fn bytes(&self, n: usize) -> *mut u8 {
        self.alloc(n, 0)
    }

    #[allow(dead_code, reason = "only some command sets use it")]
    fn array_of<T>(&self, n: usize) -> *mut T {
        self.alloc(n * core::mem::size_of::<T>(), 0).cast()
    }

    #[allow(dead_code, reason = "only some command sets use it")]
    /// Room for up to `max` entries of `entry_len` bytes, of which the return
    /// parameters keep as many as the previous output counts.
    fn items(&self, entry_len: usize, max: usize) -> *mut u8 {
        self.alloc(max * entry_len, entry_len)
    }

    #[allow(dead_code, reason = "only some command sets use it")]
    fn items_of<T>(&self, max: usize) -> *mut T {
        self.items(core::mem::size_of::<T>(), max).cast()
    }

    #[allow(dead_code, reason = "only some command sets use it")]
    /// Puts parameter bytes the C function does not write back among the
    /// return parameters, at this point in their order.
    fn echo(&self, bytes: &[u8]) {
        let dst = self.alloc(bytes.len(), 0);
        if !self.overflow.get() {
            // SAFETY: `alloc` reserved `bytes.len()` bytes at `dst`.
            unsafe { core::ptr::copy_nonoverlapping(bytes.as_ptr(), dst, bytes.len()) };
        }
    }

    /// Ends the dry run: checks that the outputs fit, then forgets them.
    fn rewind(&self) -> bool {
        let fits = !self.overflow.get();
        self.next.set(0);
        self.count.set(0);
        fits
    }

    /// Builds the return parameters from `echo` followed by the outputs.
    ///
    /// The outputs were written by the C side in native byte order, which on
    /// the little-endian Cortex-M core is also the HCI byte order.
    fn finish<R: for<'a> FromHciBytes<'a>>(&self, echo: &[u8]) -> Result<R, CmdError<ErrorKind>> {
        let mut wire = [0u8; OUT_WORDS * 4 + 8];
        wire[..echo.len()].copy_from_slice(echo);
        let mut len = echo.len();
        let mut previous: &[u8] = &[];
        for field in &self.fields[..self.count.get()] {
            let (offset, n, entry_len) = field.get();
            let (offset, mut n, entry_len) = (offset as usize, n as usize, entry_len as usize);
            // SAFETY: `alloc` recorded this range inside the storage.
            let value = unsafe { core::slice::from_raw_parts(self.base.add(offset), n) };
            if entry_len != 0 {
                let entries = match previous {
                    [count] => *count as usize,
                    [lo, hi] => u16::from_le_bytes([*lo, *hi]) as usize,
                    _ => return Err(INVALID),
                };
                n = n.min(entries * entry_len);
            }
            wire[len..][..n].copy_from_slice(&value[..n]);
            len += n;
            previous = value;
        }
        R::from_hci_bytes_complete(&wire[..len]).map_err(|_| INVALID)
    }
}

fn status(status: u8) -> Result<(), CmdError<ErrorKind>> {
    bt_hci::param::Status::from(status).to_result().map_err(CmdError::Hci)
}

fn layout_mismatch<C: Cmd>() -> CmdError<ErrorKind> {
    error!(
        "direct: argument mapping does not match opcode {:04x}",
        C::OPCODE.to_raw()
    );
    INVALID
}

macro_rules! direct {
    // Command completing with Command Status.
    (async $cmd:ident $(<$lt:lifetime>)? => |$p:ident| $f:ident($($arg:expr),* $(,)?)) => {
        impl<'d $(, $lt)?> ControllerCmdAsync<$cmd $(<$lt>)?> for ControllerAdapter<'d> {
            #[allow(unused_variables)]
            async fn exec(&self, cmd: &$cmd $(<$lt>)?) -> Result<(), CmdError<ErrorKind>> {
                let mut buf = [0u8; PARAMS_LEN];
                let $p = Params::new(&mut buf, cmd.params())?;
                if cfg!(debug_assertions) {
                    $(let _ = $arg;)*
                    if !$p.rewind() {
                        return Err(layout_mismatch::<$cmd>());
                    }
                }
                status(self.call(|| unsafe { $f($($arg),*) }))
            }
        }
    };

    // Command completing with Command Complete and no return parameters.
    ($cmd:ident $(<$lt:lifetime>)? => |$p:ident| $f:ident($($arg:expr),* $(,)?)) => {
        impl<'d $(, $lt)?> ControllerCmdSync<$cmd $(<$lt>)?> for ControllerAdapter<'d> {
            #[allow(unused_variables)]
            async fn exec(
                &self,
                cmd: &$cmd $(<$lt>)?,
            ) -> Result<<$cmd $(<$lt>)? as cmd::SyncCmd>::Return, CmdError<ErrorKind>> {
                let mut buf = [0u8; PARAMS_LEN];
                let $p = Params::new(&mut buf, cmd.params())?;
                if cfg!(debug_assertions) {
                    $(let _ = $arg;)*
                    if !$p.rewind() {
                        return Err(layout_mismatch::<$cmd>());
                    }
                }
                status(self.call(|| unsafe { $f($($arg),*) }))
            }
        }
    };

    // Command completing with Command Complete and return parameters.
    ($cmd:ident $(<$lt:lifetime>)? => |$p:ident, $o:ident| $([echo $echo:literal])? $f:ident($($arg:expr),* $(,)?)) => {
        impl<'d $(, $lt)?> ControllerCmdSync<$cmd $(<$lt>)?> for ControllerAdapter<'d> {
            #[allow(unused_variables)]
            async fn exec(
                &self,
                cmd: &$cmd $(<$lt>)?,
            ) -> Result<<$cmd $(<$lt>)? as cmd::SyncCmd>::Return, CmdError<ErrorKind>> {
                let mut buf = [0u8; PARAMS_LEN];
                let mut storage = [0u32; OUT_WORDS];
                let $p = Params::new(&mut buf, cmd.params())?;
                let $o = Out::new(&mut storage);
                if cfg!(debug_assertions) {
                    $(let _ = $arg;)*
                    if !$p.rewind() | !$o.rewind() {
                        return Err(layout_mismatch::<$cmd>());
                    }
                }
                status(self.call(|| unsafe { $f($($arg),*) }))?;
                $o.finish($p.bytes_at(0, 0 $(+ $echo)?))
            }
        }
    };
}

#[cfg(feature = "wb-hci")]
mod aci;

/// Link Control commands (Core Vol 4, Part E, 7.1)
mod link_control {
    use bt_hci::cmd::link_control::*;

    use super::*;

    direct!(async Disconnect => |p| hci_disconnect(p.u16(), p.u8()));
    direct!(async ReadRemoteVersionInformation => |p| hci_read_remote_version_information(p.u16()));
}

/// Controller & Baseband commands (Core Vol 4, Part E, 7.3)
mod controller_baseband {
    use bt_hci::cmd::controller_baseband::*;

    use super::*;

    direct!(SetEventMask => |p| hci_set_event_mask(p.ptr(8)));
    direct!(Reset => |p| hci_reset());
    direct!(ReadConnectionAcceptTimeout => |p, o| hci_read_connection_accept_timeout(o.u16()));
    direct!(WriteConnectionAceeptTimeout => |p| hci_write_connection_accept_timeout(p.u16()));
    direct!(ReadTransmitPowerLevel => |p, o| [echo 2] hci_read_transmit_power_level(p.u16(), p.u8(), o.u8()));
    direct!(SetControllerToHostFlowControl => |p| hci_set_controller_to_host_flow_control(p.u8()));
    direct!(HostBufferSize => |p| hci_host_buffer_size(p.u16(), p.u8(), p.u16(), p.u16()));
    direct!(HostNumberOfCompletedPackets<'a> => |p| hci_host_number_of_completed_packets(
        p.u8(), p.list_of::<Host_Nb_Of_Completed_Pkt_Pair_t>()
    ));
    direct!(ReadAfhChannelAssessmentMode => |p, o| hci_read_afh_channel_assessment_mode(o.u8()));
    direct!(WriteAfhChannelAssessmentMode => |p| hci_write_afh_channel_assessment_mode(p.u8()));
    direct!(SetEventMaskPage2 => |p| hci_set_event_mask_page_2(p.ptr(8)));
    direct!(ReadAuthenticatedPayloadTimeout => |p, o| [echo 2] hci_read_authenticated_payload_timeout(
        p.u16(), o.u16()
    ));
    direct!(WriteAuthenticatedPayloadTimeout => |p, o| [echo 2] hci_write_authenticated_payload_timeout(
        p.u16(), p.u16()
    ));
    direct!(SetEcosystemBaseInterval => |p| hci_set_ecosystem_base_interval(p.u16()));
    direct!(ConfigureDataPath<'a> => |p| hci_configure_data_path(p.u8(), p.u8(), p.u8(), p.rest()));
}

/// Informational parameters (Core Vol 4, Part E, 7.4)
mod info {
    use bt_hci::cmd::info::*;

    use super::*;

    direct!(ReadLocalVersionInformation => |p, o| hci_read_local_version_information(
        o.u8(), o.u16(), o.u8(), o.u16(), o.u16()
    ));
    direct!(ReadLocalSupportedCmds => |p, o| hci_read_local_supported_commands(o.bytes(64)));
    direct!(ReadLocalSupportedFeatures => |p, o| hci_read_local_supported_features(o.bytes(8)));
    direct!(ReadBdAddr => |p, o| hci_read_bd_addr(o.bytes(6)));
    direct!(ReadLocalSupportedControllerDelay<'a> => |p, o| hci_read_local_supported_controller_delay(
        p.ptr(5), p.u8(), p.u8(), p.u8(), p.rest(), o.bytes(3), o.bytes(3)
    ));
}

/// Status parameters (Core Vol 4, Part E, 7.5)
mod status {
    use bt_hci::cmd::status::*;

    use super::*;

    direct!(ReadRssi => |p, o| [echo 2] hci_read_rssi(p.u16(), o.u8()));
}

/// LE Controller commands (Core Vol 4, Part E, 7.8)
mod le {
    use bt_hci::cmd::le::*;

    use super::*;

    // Device setup
    direct!(LeSetEventMask => |p| hci_le_set_event_mask(p.ptr(8)));
    direct!(LeReadBufferSize => |p, o| hci_le_read_buffer_size(o.u16(), o.u8()));
    direct!(LeReadBufferSizeV2 => |p, o| hci_le_read_buffer_size_v2(o.u16(), o.u8(), o.u16(), o.u8()));
    direct!(LeReadLocalSupportedFeatures => |p, o| hci_le_read_local_supported_features_page_0(o.bytes(8)));
    direct!(LeReadAllLocalSupportedFeatures => |p, o| hci_le_read_all_local_supported_features(o.u8(), o.bytes(248)));
    direct!(LeSetRandomAddr => |p| hci_le_set_random_address(p.ptr(6)));
    direct!(LeReadSupportedStates => |p, o| hci_le_read_supported_states(o.bytes(8)));
    direct!(LeSetHostChannelClassification => |p| hci_le_set_host_channel_classification(p.ptr(5)));
    direct!(LeSetHostFeature => |p| hci_le_set_host_feature(p.u8(), p.u8()));
    direct!(LeReadTransmitPower => |p, o| hci_le_read_transmit_power(o.u8(), o.u8()));
    direct!(LeReadRfPathCompensation => |p, o| hci_le_read_rf_path_compensation(o.u16(), o.u16()));
    direct!(LeWriteRfPathCompensation => |p| hci_le_write_rf_path_compensation(p.u16(), p.u16()));

    // Legacy advertising
    direct!(LeSetAdvParams => |p| hci_le_set_advertising_parameters(
        p.u16(), p.u16(), p.u8(), p.u8(), p.u8(), p.ptr(6), p.u8(), p.u8()
    ));
    direct!(LeReadAdvPhysicalChannelTxPower => |p, o| hci_le_read_advertising_physical_channel_tx_power(o.u8()));
    direct!(LeSetAdvData => |p| hci_le_set_advertising_data(p.u8(), p.ptr(31)));
    direct!(LeSetScanResponseData => |p| hci_le_set_scan_response_data(p.u8(), p.ptr(31)));
    direct!(LeSetAdvEnable => |p| hci_le_set_advertising_enable(p.u8()));

    // Legacy scanning and connecting
    direct!(LeSetScanParams => |p| hci_le_set_scan_parameters(p.u8(), p.u16(), p.u16(), p.u8(), p.u8()));
    direct!(LeSetScanEnable => |p| hci_le_set_scan_enable(p.u8(), p.u8()));
    direct!(async LeCreateConn => |p| hci_le_create_connection(
        p.u16(), p.u16(), p.u8(), p.u8(), p.ptr(6), p.u8(), p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u16()
    ));
    direct!(LeCreateConnCancel => |p| hci_le_create_connection_cancel());

    // Filter accept list
    direct!(LeReadFilterAcceptListSize => |p, o| hci_le_read_filter_accept_list_size(o.u8()));
    direct!(LeClearFilterAcceptList => |p| hci_le_clear_filter_accept_list());
    direct!(LeAddDeviceToFilterAcceptList => |p| hci_le_add_device_to_filter_accept_list(p.u8(), p.ptr(6)));
    direct!(LeRemoveDeviceFromFilterAcceptList => |p| hci_le_remove_device_from_filter_accept_list(p.u8(), p.ptr(6)));

    // Connection management
    direct!(async LeConnUpdate => |p| hci_le_connection_update(
        p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u16()
    ));
    direct!(LeReadChannelMap => |p, o| [echo 2] hci_le_read_channel_map(p.u16(), o.bytes(5)));
    direct!(async LeReadRemoteFeatures => |p| hci_le_read_remote_features_page_0(p.u16()));
    direct!(async LeReadAllRemoteFeatures => |p| hci_le_read_all_remote_features(p.u16(), p.u8()));
    direct!(LeRemoteConnectionParameterRequestReply => |p, o| [echo 2] hci_le_remote_connection_parameter_request_reply(
        p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u16()
    ));
    direct!(LeRemoteConnectionParameterRequestNegativeReply => |p, o| [echo 2]
        hci_le_remote_connection_parameter_request_negative_reply(p.u16(), p.u8())
    );
    direct!(LeSetDataLength => |p, o| [echo 2] hci_le_set_data_length(p.u16(), p.u16(), p.u16()));
    direct!(LeReadSuggestedDefaultDataLength => |p, o| hci_le_read_suggested_default_data_length(o.u16(), o.u16()));
    direct!(LeWriteSuggestedDefaultDataLength => |p| hci_le_write_suggested_default_data_length(p.u16(), p.u16()));
    direct!(LeReadMaxDataLength => |p, o| hci_le_read_maximum_data_length(o.u16(), o.u16(), o.u16(), o.u16()));
    direct!(LeReadPhy => |p, o| [echo 2] hci_le_read_phy(p.u16(), o.u8(), o.u8()));
    direct!(LeSetDefaultPhy => |p| hci_le_set_default_phy(p.u8(), p.u8(), p.u8()));
    direct!(async LeSetPhy => |p| hci_le_set_phy(p.u16(), p.u8(), p.u8(), p.u8(), p.u16()));
    direct!(async LeRequestPeerSca => |p| hci_le_request_peer_sca(p.u16()));
    direct!(LeSetDefaultSubrate => |p| hci_le_set_default_subrate(p.u16(), p.u16(), p.u16(), p.u16(), p.u16()));
    direct!(async LeSubrateRequest => |p| hci_le_subrate_request(p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u16()));
    direct!(async LeFrameSpaceUpdate => |p| hci_le_frame_space_update(p.u16(), p.u16(), p.u16(), p.u8(), p.u16()));

    // Encryption and security
    direct!(LeEncrypt => |p, o| hci_le_encrypt(p.ptr(16), p.ptr(16), o.bytes(16)));
    direct!(LeRand => |p, o| hci_le_rand(o.bytes(8)));
    direct!(async LeEnableEncryption => |p| hci_le_enable_encryption(p.u16(), p.ptr(8), p.u16(), p.ptr(16)));
    direct!(LeLongTermKeyRequestReply => |p, o| [echo 2] hci_le_long_term_key_request_reply(p.u16(), p.ptr(16)));
    direct!(LeLongTermKeyRequestNegativeReply => |p, o| [echo 2] hci_le_long_term_key_request_negative_reply(p.u16()));
    direct!(async LeReadLocalP256PublicKey => |p| hci_le_read_local_p256_public_key());
    direct!(async LeGenerateDhkey => |p| hci_le_generate_dhkey(p.ptr(64)));
    direct!(async LeGenerateDhkeyV2 => |p| hci_le_generate_dhkey_v2(p.ptr(64), p.u8()));

    // Privacy
    direct!(LeAddDeviceToResolvingList => |p| hci_le_add_device_to_resolving_list(
        p.u8(), p.ptr(6), p.ptr(16), p.ptr(16)
    ));
    direct!(LeRemoveDeviceFromResolvingList => |p| hci_le_remove_device_from_resolving_list(p.u8(), p.ptr(6)));
    direct!(LeClearResolvingList => |p| hci_le_clear_resolving_list());
    direct!(LeReadResolvingListSize => |p, o| hci_le_read_resolving_list_size(o.u8()));
    direct!(LeReadPeerResolvableAddr => |p, o| hci_le_read_peer_resolvable_address(p.u8(), p.ptr(6), o.bytes(6)));
    direct!(LeReadLocalResolvableAddr => |p, o| hci_le_read_local_resolvable_address(p.u8(), p.ptr(6), o.bytes(6)));
    direct!(LeSetAddrResolutionEnable => |p| hci_le_set_address_resolution_enable(p.u8()));
    direct!(LeSetResolvablePrivateAddrTimeout => |p| hci_le_set_resolvable_private_address_timeout(p.u16()));
    direct!(LeSetPrivacyMode => |p| hci_le_set_privacy_mode(p.u8(), p.ptr(6), p.u8()));
    direct!(LeSetDataRelatedAddrChanges => |p| hci_le_set_data_related_address_changes(p.u8(), p.u8()));

    // Extended advertising
    direct!(LeSetAdvSetRandomAddr => |p| hci_le_set_advertising_set_random_address(p.u8(), p.ptr(6)));
    direct!(LeSetExtAdvParams => |p, o| hci_le_set_extended_advertising_parameters(
        p.u8(), p.u16(), p.ptr(3), p.ptr(3), p.u8(), p.u8(), p.u8(), p.ptr(6), p.u8(), p.u8(), p.u8(), p.u8(),
        p.u8(), p.u8(), p.u8(), o.u8()
    ));
    direct!(LeSetExtAdvParamsV2 => |p, o| hci_le_set_extended_advertising_parameters_v2(
        p.u8(), p.u16(), p.ptr(3), p.ptr(3), p.u8(), p.u8(), p.u8(), p.ptr(6), p.u8(), p.u8(), p.u8(), p.u8(),
        p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), o.u8()
    ));
    direct!(LeSetExtAdvData<'a> => |p| hci_le_set_extended_advertising_data(p.u8(), p.u8(), p.u8(), p.u8(), p.rest()));
    direct!(LeSetExtScanResponseData<'a> => |p| hci_le_set_extended_scan_response_data(
        p.u8(), p.u8(), p.u8(), p.u8(), p.rest()
    ));
    direct!(LeSetExtAdvEnable<'a> => |p| hci_le_set_extended_advertising_enable(
        p.u8(), p.u8(), p.list_of::<Adv_Set_t>()
    ));
    direct!(LeReadMaxAdvDataLength => |p, o| hci_le_read_maximum_advertising_data_length(o.u16()));
    direct!(LeReadNumberOfSupportedAdvSets => |p, o| hci_le_read_number_of_supported_advertising_sets(o.u8()));
    direct!(LeRemoveAdvSet => |p| hci_le_remove_advertising_set(p.u8()));
    direct!(LeClearAdvSets => |p| hci_le_clear_advertising_sets());

    // Periodic advertising
    direct!(LeSetPeriodicAdvParams => |p| hci_le_set_periodic_advertising_parameters(
        p.u8(), p.u16(), p.u16(), p.u16()
    ));
    direct!(LeSetPeriodicAdvParamsV2 => |p, o| [echo 1] hci_le_set_periodic_advertising_parameters_v2(
        p.u8(), p.u16(), p.u16(), p.u16(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8()
    ));
    direct!(LeSetPeriodicAdvData<'a> => |p| hci_le_set_periodic_advertising_data(p.u8(), p.u8(), p.u8(), p.rest()));
    direct!(LeSetPeriodicAdvEnable => |p| hci_le_set_periodic_advertising_enable(p.u8(), p.u8()));
    direct!(LeSetPeriodicAdvSubeventData<'a> => |p, o| [echo 1] hci_le_set_periodic_advertising_subevent_data(
        p.u8(), p.u8(), p.rest()
    ));
    direct!(LeSetPeriodicAdvResponseData<'a> => |p, o| [echo 2] hci_le_set_periodic_advertising_response_data(
        p.u16(), p.u16(), p.u8(), p.u8(), p.u8(), p.u8(), p.rest()
    ));
    direct!(LeSetPeriodicSyncSubevent<'a> => |p, o| [echo 2] hci_le_set_periodic_sync_subevent(
        p.u16(), p.u16(), p.u8(), p.list(1)
    ));

    // Extended scanning and connecting
    direct!(LeSetExtScanParams => |p| hci_le_set_extended_scan_parameters(
        p.u8(), p.u8(), p.u8(), p.rest().cast::<Scan_Param_Phy_t>()
    ));
    direct!(LeSetExtScanEnable => |p| hci_le_set_extended_scan_enable(p.u8(), p.u8(), p.u16(), p.u16()));
    direct!(async LeExtCreateConn => |p| hci_le_extended_create_connection(
        p.u8(), p.u8(), p.u8(), p.ptr(6), p.u8(), p.rest().cast::<Init_Param_Phy_t>()
    ));
    direct!(async LeExtCreateConnV2 => |p| hci_le_extended_create_connection_v2(
        p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.ptr(6), p.u8(), p.rest().cast::<Init_Param_Phy_t>()
    ));

    // Periodic advertising sync
    direct!(async LePeriodicAdvCreateSync => |p| hci_le_periodic_advertising_create_sync(
        p.u8(), p.u8(), p.u8(), p.ptr(6), p.u16(), p.u16(), p.u8()
    ));
    direct!(LePeriodicAdvCreateSyncCancel => |p| hci_le_periodic_advertising_create_sync_cancel());
    direct!(LePeriodicAdvTerminateSync => |p| hci_le_periodic_advertising_terminate_sync(p.u16()));
    direct!(LeAddDeviceToPeriodicAdvList => |p| hci_le_add_device_to_periodic_advertiser_list(
        p.u8(), p.ptr(6), p.u8()
    ));
    direct!(LeRemoveDeviceFromPeriodicAdvList => |p| hci_le_remove_device_from_periodic_advertiser_list(
        p.u8(), p.ptr(6), p.u8()
    ));
    direct!(LeClearPeriodicAdvList => |p| hci_le_clear_periodic_advertiser_list());
    direct!(LeReadPeriodicAdvListSize => |p, o| hci_le_read_periodic_advertiser_list_size(o.u8()));
    direct!(LeSetPeriodicAdvReceiveEnable => |p| hci_le_set_periodic_advertising_receive_enable(p.u16(), p.u8()));
    direct!(LePeriodicAdvSyncTransfer => |p, o| [echo 2] hci_le_periodic_advertising_sync_transfer(
        p.u16(), p.u16(), p.u16()
    ));
    direct!(LePeriodicAdvSetInfoTransfer => |p, o| [echo 2] hci_le_periodic_advertising_set_info_transfer(
        p.u16(), p.u16(), p.u8()
    ));
    direct!(LeSetPeriodicAdvSyncTransferParams => |p, o| [echo 2]
        hci_le_set_periodic_advertising_sync_transfer_parameters(p.u16(), p.u8(), p.u16(), p.u16(), p.u8())
    );
    direct!(LeSetDefaultPeriodicAdvSyncTransferParams => |p|
        hci_le_set_default_periodic_advertising_sync_transfer_parameters(p.u8(), p.u16(), p.u16(), p.u8())
    );

    // Constant tone extension
    direct!(LeSetConnectionlessCteTransmitParams<'a> => |p| hci_le_set_connectionless_cte_transmit_parameters(
        p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.list(1)
    ));
    direct!(LeSetConnectionlessCteTransmitEnable => |p| hci_le_set_connectionless_cte_transmit_enable(p.u8(), p.u8()));
    direct!(LeSetConnectionlessIqSamplingEnable<'a> => |p, o| [echo 2] hci_le_set_connectionless_iq_sampling_enable(
        p.u16(), p.u8(), p.u8(), p.u8(), p.u8(), p.list(1)
    ));
    direct!(LeSetConnCteReceiveParams<'a> => |p, o| [echo 2] hci_le_set_connection_cte_receive_parameters(
        p.u16(), p.u8(), p.u8(), p.u8(), p.list(1)
    ));
    direct!(LeSetConnCteTransmitParams<'a> => |p, o| [echo 2] hci_le_set_connection_cte_transmit_parameters(
        p.u16(), p.u8(), p.u8(), p.list(1)
    ));
    direct!(LeConnCteRequestEnable => |p, o| [echo 2] hci_le_connection_cte_request_enable(
        p.u16(), p.u8(), p.u16(), p.u8(), p.u8()
    ));
    direct!(LeConnCteResponseEnable => |p, o| [echo 2] hci_le_connection_cte_response_enable(p.u16(), p.u8()));
    direct!(LeReadAntennaInformation => |p, o| hci_le_read_antenna_information(o.u8(), o.u8(), o.u8(), o.u8()));

    // Power control
    direct!(LeEnhancedReadTransmitPowerLevel => |p, o| [echo 3] hci_le_enhanced_read_transmit_power_level(
        p.u16(), p.u8(), o.u8(), o.u8()
    ));
    direct!(async LeReadRemoteTransmitPowerLevel => |p| hci_le_read_remote_transmit_power_level(p.u16(), p.u8()));
    direct!(LeSetPathLossReportingParams => |p, o| [echo 2] hci_le_set_path_loss_reporting_parameters(
        p.u16(), p.u8(), p.u8(), p.u8(), p.u8(), p.u16()
    ));
    direct!(LeSetPathLossReportingEnable => |p, o| [echo 2] hci_le_set_path_loss_reporting_enable(p.u16(), p.u8()));
    direct!(LeSetTransmitPowerReportingEnable => |p, o| [echo 2] hci_le_set_transmit_power_reporting_enable(
        p.u16(), p.u8(), p.u8()
    ));

    // Monitored advertisers
    direct!(LeAddDeviceToMonitoredAdvList => |p| hci_le_add_device_to_monitored_advertisers_list(
        p.u8(), p.ptr(6), p.u8(), p.u8(), p.u8()
    ));
    direct!(LeRemoveDeviceFromMonitoredAdvList => |p| hci_le_remove_device_from_monitored_advertisers_list(
        p.u8(), p.ptr(6)
    ));
    direct!(LeClearMonitoredAdvList => |p| hci_le_clear_monitored_advertisers_list());
    direct!(LeReadMonitoredAdvListSize => |p, o| hci_le_read_monitored_advertisers_list_size(o.u8()));
    direct!(LeEnableMonitoringAdv => |p| hci_le_enable_monitoring_advertisers(p.u8()));

    // Isochronous channels. `LeSetCigParameters`, `LeSetCigParametersTest` and
    // `LeCreateCis` are left out: their bt-hci 0.11 layouts differ from the
    // spec (and from the ST functions).
    direct!(LeReadIsoTxSync => |p, o| [echo 2] hci_le_read_iso_tx_sync(p.u16(), o.u16(), o.u32(), o.bytes(3)));
    direct!(LeRemoveCig => |p, o| [echo 1] hci_le_remove_cig(p.u8()));
    direct!(async LeAcceptCisRequest => |p| hci_le_accept_cis_request(p.u16()));
    direct!(LeRejectCisRequest => |p, o| [echo 2] hci_le_reject_cis_request(p.u16(), p.u8()));
    direct!(async LeCreateBig => |p| hci_le_create_big(
        p.u8(), p.u8(), p.u8(), p.ptr(3), p.u16(), p.u16(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.ptr(16)
    ));
    direct!(async LeCreateBigTest => |p| hci_le_create_big_test(
        p.u8(), p.u8(), p.u8(), p.ptr(3), p.u16(), p.u8(), p.u16(), p.u16(), p.u8(), p.u8(), p.u8(), p.u8(),
        p.u8(), p.u8(), p.u8(), p.ptr(16)
    ));
    direct!(async LeTerminateBig => |p| hci_le_terminate_big(p.u8(), p.u8()));
    direct!(async LeBigCreateSync<'a> => |p| hci_le_big_create_sync(
        p.u8(), p.u16(), p.u8(), p.ptr(16), p.u8(), p.u16(), p.u8(), p.list(1)
    ));
    direct!(LeBigTerminateSync => |p, o| [echo 1] hci_le_big_terminate_sync(p.u8()));
    direct!(LeSetupIsoDataPath<'a> => |p, o| [echo 2] hci_le_setup_iso_data_path(
        p.u16(), p.u8(), p.u8(), p.ptr(5), p.ptr(3), p.u8(), p.rest()
    ));
    direct!(LeRemoveIsoDataPath => |p, o| [echo 2] hci_le_remove_iso_data_path(p.u16(), p.u8()));
    direct!(LeIsoTransmitTest => |p, o| [echo 2] hci_le_iso_transmit_test(p.u16(), p.u8()));
    direct!(LeIsoReceiveTest => |p, o| [echo 2] hci_le_iso_receive_test(p.u16(), p.u8()));
    direct!(LeIsoReadTestCounters => |p, o| [echo 2] hci_le_iso_read_test_counters(p.u16(), o.u32(), o.u32(), o.u32()));
    direct!(LeIsoTestEnd => |p, o| [echo 2] hci_le_iso_test_end(p.u16(), o.u32(), o.u32(), o.u32()));
    direct!(LeReadIsoLinkQuality => |p, o| [echo 2] hci_le_read_iso_link_quality(
        p.u16(), o.u32(), o.u32(), o.u32(), o.u32(), o.u32(), o.u32(), o.u32()
    ));

    // Channel sounding
    direct!(LeCsReadLocalSupportedCapabilities => |p, o| hci_le_cs_read_local_supported_capabilities(
        o.u8(), o.u16(), o.u8(), o.u8(), o.u8(), o.u8(), o.u8(), o.u8(), o.u8(), o.u8(), o.u16(), o.u16(), o.u8(),
        o.u16(), o.u16(), o.u16(), o.u16(), o.u16(), o.u8(), o.u8()
    ));
    direct!(async LeCsReadRemoteSupportedCapabilities => |p| hci_le_cs_read_remote_supported_capabilities(p.u16()));
    direct!(LeCsWriteCachedRemoteSupportedCapabilities => |p, o| [echo 2]
        hci_le_cs_write_cached_remote_supported_capabilities(
            p.u16(), p.u8(), p.u16(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u16(), p.u16(),
            p.u8(), p.u16(), p.u16(), p.u16(), p.u16(), p.u16(), p.u8(), p.u8()
        )
    );
    direct!(async LeCsSecurityEnableCommand => |p| hci_le_cs_security_enable(p.u16()));
    direct!(LeCsSetDefaultSettings => |p, o| [echo 2] hci_le_cs_set_default_settings(p.u16(), p.u8(), p.u8(), p.u8()));
    direct!(async LeCsReadRemoteFaeTable => |p| hci_le_cs_read_remote_fae_table(p.u16()));
    direct!(LeCsWriteCachedRemoteFaeTable => |p, o| [echo 2] hci_le_cs_write_cached_remote_fae_table(
        p.u16(), p.ptr(72)
    ));
    direct!(async LeCsCreateConfig => |p| hci_le_cs_create_config(
        p.u16(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.ptr(10),
        p.u8(), p.u8(), p.u8(), p.u8(), p.u8()
    ));
    direct!(async LeCsRemoveConfig => |p| hci_le_cs_remove_config(p.u16(), p.u8()));
    direct!(LeCsSetChannelClassification => |p| hci_le_cs_set_channel_classification(p.ptr(10)));
    direct!(LeCsSetProcedureParams => |p, o| [echo 2] hci_le_cs_set_procedure_parameters(
        p.u16(), p.u8(), p.u16(), p.u16(), p.u16(), p.u16(), p.ptr(3), p.ptr(3), p.u8(), p.u8(), p.u8(), p.u8(),
        p.u8(), p.u8()
    ));
    direct!(async LeCsProcedureEnable => |p| hci_le_cs_procedure_enable(p.u16(), p.u8(), p.u8()));
    direct!(LeCsTest<'a> => |p| hci_le_cs_test(
        p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.ptr(3), p.u16(), p.u8(), p.u8(), p.u8(),
        p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u16(), p.u8(), p.u16(), p.u8(), p.rest()
    ));
    direct!(async LeCsTestEnd => |p| hci_le_cs_test_end());

    // Test mode
    direct!(LeReceiverTest => |p| hci_le_receiver_test(p.u8()));
    direct!(LeReceiverTestV2 => |p| hci_le_receiver_test_v2(p.u8(), p.u8(), p.u8()));
    direct!(LeReceiverTestV3<'a> => |p| hci_le_receiver_test_v3(
        p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.list(1)
    ));
    direct!(LeTransmitterTest => |p| hci_le_transmitter_test(p.u8(), p.u8(), p.u8()));
    direct!(LeTransmitterTestV2 => |p| hci_le_transmitter_test_v2(p.u8(), p.u8(), p.u8(), p.u8()));
    direct!(LeTransmitterTestV3<'a> => |p| hci_le_transmitter_test_v3(
        p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.list(1)
    ));
    direct!(LeTransmitterTestV4<'a> => |p| hci_le_transmitter_test_v4(
        p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.u8(), p.list(1), p.u8()
    ));
    direct!(LeTestEnd => |p, o| hci_le_test_end(o.u16()));
}
