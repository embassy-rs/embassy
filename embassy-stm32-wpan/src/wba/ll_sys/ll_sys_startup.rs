#[cfg(feature = "wba_ble")]
use crate::bindings::ble::{BleStack_Init, BleStack_init_t, tBleStatus};
use crate::bindings::link_layer::{
    LL_SYS_STATUS_T_LL_SYS_OK, ll_sys_assert, ll_sys_bg_process_init, ll_sys_config_params, ll_sys_dp_slp_init,
    ll_sys_status_t,
};
#[cfg(feature = "wba_ble")]
use crate::bindings::link_layer::{
    ble_buff_hdr_p, hci_dispatch_tbl, hci_get_dis_tbl, hst_cbk, ll_intf_init, ll_intf_rgstr_hst_cbk,
    ll_intf_rgstr_hst_cbk_ll_queue_full,
};

/// BLE status code for success
#[cfg(feature = "wba_ble")]
const BLE_STATUS_SUCCESS: tBleStatus = 0;
#[cfg(feature = "wba_mac")]
use crate::bindings::mac::ST_MAC_preInit;
// /**
//   ******************************************************************************
//   * @file    ll_sys_startup.c
//   * @author  MCD Application Team
//   * @brief   Link Layer IP system interface startup module
//   ******************************************************************************
//   * @attention
//   *
//   * Copyright (c) 2022 STMicroelectronics.
//   * All rights reserved.
//   *
//   * This software is licensed under terms that can be found in the LICENSE file
//   * in the root directory of this software component.
//   * If no LICENSE file comes with this software, it is provided AS-IS.
//   *
//   ******************************************************************************
//   */
//
// #include "ll_fw_config.h"
// #include "ll_sys.h"
// #include "ll_intf.h"
// #include "ll_sys_startup.h"
// #include "common_types.h"
// #if defined(MAC)
// #ifndef OPENTHREAD_CONFIG_FILE
// /* Projects with MAC Layer (i.e. 15.4 except Thread) */
// #include "st_mac_802_15_4_sap.h"
// #endif /* OPENTHREAD_CONFIG_FILE */
// #endif /* MAC */
//

#[allow(dead_code)]
/**
 * @brief  Missed HCI event flag
 */
static mut MISSED_HCI_EVENT_FLAG: u8 = 0;

// static void ll_sys_dependencies_init(void);
// #if SUPPORT_BLE

#[cfg(feature = "wba_ble")]
#[allow(dead_code)]
unsafe extern "C" fn ll_sys_event_missed_cb(_ptr_evnt_hdr: ble_buff_hdr_p) {
    MISSED_HCI_EVENT_FLAG = 1;
}

// ll_sys_ble_cntrl_init is called by BleStack_Init from the library.
// We must provide this function as it's expected as a callback.

#[cfg(feature = "wba_ble")]
/**
 * @brief  Initialize the Link Layer IP BLE controller
 * @param  host_callback - callback function for HCI events
 * @retval None
 *
 * This function is called by BleStack_Init internally.
 */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ll_sys_ble_cntrl_init(host_callback: hst_cbk) {
    #[cfg(feature = "defmt")]
    defmt::trace!("ll_sys_ble_cntrl_init: starting");

    let p_hci_dis_tbl: *const hci_dispatch_tbl = core::ptr::null();

    hci_get_dis_tbl(&p_hci_dis_tbl as *const *const _ as *mut *const _);
    #[cfg(feature = "defmt")]
    defmt::trace!("ll_sys_ble_cntrl_init: hci_get_dis_tbl done");

    ll_intf_init(p_hci_dis_tbl);
    #[cfg(feature = "defmt")]
    defmt::trace!("ll_sys_ble_cntrl_init: ll_intf_init done");

    ll_intf_rgstr_hst_cbk(host_callback);
    #[cfg(feature = "defmt")]
    defmt::trace!("ll_sys_ble_cntrl_init: ll_intf_rgstr_hst_cbk done");

    ll_intf_rgstr_hst_cbk_ll_queue_full(Some(ll_sys_event_missed_cb));
    #[cfg(feature = "defmt")]
    defmt::trace!("ll_sys_ble_cntrl_init: ll_intf_rgstr_hst_cbk_ll_queue_full done");

    ll_sys_dependencies_init();
    #[cfg(feature = "defmt")]
    defmt::trace!("ll_sys_ble_cntrl_init: ll_sys_dependencies_init done");
}

// NOTE: init_ble_link_layer and init_ble_link_layer_minimal have been removed.
// Use init_ble_stack() instead, which uses BleStack_Init for proper initialization.

#[cfg(feature = "wba_ble")]
/// Complete the BLE link layer initialization
/// This should be called after the sequencer is running
pub fn complete_ble_link_layer_init() {
    #[cfg(feature = "defmt")]
    defmt::trace!("complete_ble_link_layer_init: starting");

    unsafe {
        ll_sys_dependencies_init();
    }

    #[cfg(feature = "defmt")]
    defmt::trace!("complete_ble_link_layer_init: done");
}

// ========================================================================
// BleStack_Init based initialization (recommended approach)
// ========================================================================

/// BLE stack configuration parameters
#[cfg(feature = "wba_ble")]
pub mod ble_config {
    /// Maximum number of simultaneous BLE connections
    pub const CFG_BLE_NUM_LINK: u8 = 2;
    /// Maximum ATT MTU size
    pub const CFG_BLE_ATT_MTU_MAX: u16 = 247;
    /// Number of GATT attributes
    pub const CFG_BLE_NUM_GATT_ATTRIBUTES: u16 = 68;
    /// Number of GATT services
    pub const CFG_BLE_NUM_GATT_SERVICES: u16 = 8;
    /// GATT attribute value array size
    pub const CFG_BLE_ATT_VALUE_ARRAY_SIZE: u16 = 1344;
    /// Prepare write list size (BLE_DEFAULT_PREP_WRITE_LIST_SIZE = BLE_PREP_WRITE_X_ATT(512) = DIVC(512,18)*2 = 58)
    pub const CFG_BLE_PREPARE_WRITE_LIST_SIZE: u8 = 58;
    /// Memory block count margin
    pub const CFG_BLE_MBLOCK_COUNT_MARGIN: u16 = 0x15; // 21
    /// Maximum COC number
    pub const CFG_BLE_COC_NBR_MAX: u8 = 0;
    /// Maximum COC MPS
    pub const CFG_BLE_COC_MPS_MAX: u16 = 0;
    /// Maximum COC initiator number
    pub const CFG_BLE_COC_INITIATOR_NBR_MAX: u8 = 0;
    /// Number of EATT bearers per link
    pub const CFG_BLE_EATT_BEARER_PER_LINK: u8 = 0;
    /// NVM maximum size (in 64-bit words)
    pub const CFG_BLE_NVM_SIZE_MAX: u16 = 256;
    /// BLE options
    pub const CFG_BLE_OPTIONS: u16 = 0x0D; // DEV_NAME_READ_ONLY | REDUCED_DB_IN_NVM | CS_ALGO_2

    // Memory block size (from ble_bufsize.h)
    const BLE_MEM_BLOCK_SIZE: usize = 32;
    // Fixed buffer size for full stack (from ble_bufsize.h)
    const BLE_FIXED_BUFFER_SIZE_BYTES: usize = 516;
    // Per-link size for full stack (from ble_bufsize.h)
    const BLE_PER_LINK_SIZE_BYTES: usize = 192;

    /// Helper: divide and round up
    const fn divc(a: usize, b: usize) -> usize {
        (a + b - 1) / b
    }

    /// Calculate memory blocks for TX
    const fn mem_block_x_tx(mtu: usize) -> usize {
        divc(mtu + 4, BLE_MEM_BLOCK_SIZE) + 1
    }

    /// Calculate memory blocks for PTX (parallel TX)
    const fn mem_block_x_ptx(n_link: usize) -> usize {
        n_link // Full stack
    }

    /// Calculate memory blocks for RX
    const fn mem_block_x_rx(mtu: usize, n_link: usize) -> usize {
        (divc(mtu + 4, BLE_MEM_BLOCK_SIZE) + 2) * n_link + 1
    }

    /// Calculate total memory blocks needed
    const fn mem_block_x_mtu(mtu: usize, n_link: usize) -> usize {
        mem_block_x_tx(mtu) + mem_block_x_ptx(n_link) + mem_block_x_rx(mtu, n_link)
    }

    /// Calculate MBLOCK_COUNT based on ble_bufsize.h formula
    pub const fn mblock_count() -> usize {
        let pw = CFG_BLE_PREPARE_WRITE_LIST_SIZE as usize;
        let mtu = CFG_BLE_ATT_MTU_MAX as usize;
        let n_link = CFG_BLE_NUM_LINK as usize;
        let margin = CFG_BLE_MBLOCK_COUNT_MARGIN as usize;

        let mem_blocks = mem_block_x_mtu(mtu, n_link);
        let secure_conn_min = 4;
        let max_blocks = if mem_blocks > secure_conn_min {
            mem_blocks
        } else {
            secure_conn_min
        };

        pw + max_blocks + margin
    }

    /// Calculate GATT buffer size using BLE_TOTAL_BUFFER_SIZE_GATT macro formula
    /// Formula: ((((att_value_array_size) - 1) | 3) + 1) + (40 * num_gatt_attributes) + (48 * num_gatt_services)
    pub const fn gatt_buffer_size() -> usize {
        let att_val_size = CFG_BLE_ATT_VALUE_ARRAY_SIZE as usize;
        let num_attr = CFG_BLE_NUM_GATT_ATTRIBUTES as usize;
        let num_serv = CFG_BLE_NUM_GATT_SERVICES as usize;

        // Align att_value_array_size to 4-byte boundary
        let aligned_att_val = ((att_val_size - 1) | 3) + 1;

        aligned_att_val + (40 * num_attr) + (48 * num_serv)
    }

    /// Calculate dynamic allocation buffer size using BLE_TOTAL_BUFFER_SIZE macro formula
    /// Formula: 16 + BLE_FIXED_BUFFER_SIZE_BYTES + (BLE_PER_LINK_SIZE_BYTES * n_link) + ((BLE_MEM_BLOCK_SIZE + 8) * mblocks_count)
    pub const fn dyn_alloc_buffer_size() -> usize {
        let n_link = CFG_BLE_NUM_LINK as usize;
        let mblocks = mblock_count();

        16 + BLE_FIXED_BUFFER_SIZE_BYTES + (BLE_PER_LINK_SIZE_BYTES * n_link) + ((BLE_MEM_BLOCK_SIZE + 8) * mblocks)
    }
}

/// Static buffers for BLE stack
#[cfg(feature = "wba_ble")]
mod ble_buffers {
    use super::ble_config;

    /// Dynamic allocation buffer for BLE stack
    #[repr(align(4))]
    pub struct DynAllocBuffer(pub [u8; ble_config::dyn_alloc_buffer_size()]);

    /// GATT buffer
    #[repr(align(4))]
    pub struct GattBuffer(pub [u8; ble_config::gatt_buffer_size()]);

    /// NVM cache buffer
    #[repr(align(8))]
    pub struct NvmCacheBuffer(pub [u64; (ble_config::CFG_BLE_NVM_SIZE_MAX as usize + 7) / 8]);

    pub static mut DYN_ALLOC_BUFFER: DynAllocBuffer = DynAllocBuffer([0u8; ble_config::dyn_alloc_buffer_size()]);
    pub static mut GATT_BUFFER: GattBuffer = GattBuffer([0u8; ble_config::gatt_buffer_size()]);
    pub static mut NVM_CACHE_BUFFER: NvmCacheBuffer =
        NvmCacheBuffer([0u64; (ble_config::CFG_BLE_NVM_SIZE_MAX as usize + 7) / 8]);
}

#[cfg(feature = "wba_ble")]
/// Initialize the BLE stack using the high-level BleStack_Init API
///
/// This is the recommended initialization method as it properly sets up
/// all memory management and internal state before calling ll_intf_init.
///
/// Returns Ok(()) on success, Err with status code on failure.
pub fn init_ble_stack() -> Result<(), u8> {
    use ble_config::*;

    use crate::wba::linklayer_plat::LINKLAYER_PLAT_ClockInit;

    #[cfg(feature = "defmt")]
    defmt::info!("init_ble_stack: starting BLE stack initialization");

    #[cfg(feature = "defmt")]
    {
        defmt::debug!("init_ble_stack: buffer sizes:");
        defmt::debug!("  DYN_ALLOC_BUFFER: {} bytes", dyn_alloc_buffer_size());
        defmt::debug!("  GATT_BUFFER: {} bytes", gatt_buffer_size());
        defmt::debug!("  mblockCount: {}", mblock_count());
        defmt::debug!("  numOfLinks: {}", CFG_BLE_NUM_LINK);
        defmt::debug!("  attMtu: {}", CFG_BLE_ATT_MTU_MAX);
    }

    unsafe {
        // 1. Enable radio clock first
        LINKLAYER_PLAT_ClockInit();

        #[cfg(feature = "defmt")]
        defmt::trace!("init_ble_stack: clock init done");

        // 2. Prepare BleStack_init_t structure
        let init_params = BleStack_init_t {
            bleStartRamAddress: ble_buffers::DYN_ALLOC_BUFFER.0.as_mut_ptr(),
            total_buffer_size: ble_buffers::DYN_ALLOC_BUFFER.0.len() as u32,
            nvm_cache_buffer: ble_buffers::NVM_CACHE_BUFFER.0.as_mut_ptr(),
            nvm_cache_size: CFG_BLE_NVM_SIZE_MAX - 1,
            nvm_cache_max_size: CFG_BLE_NVM_SIZE_MAX,
            bleStartRamAddress_GATT: ble_buffers::GATT_BUFFER.0.as_mut_ptr(),
            total_buffer_size_GATT: ble_buffers::GATT_BUFFER.0.len() as u32,
            gatt_long_write_buffer: core::ptr::null_mut(),
            extra_data_buffer: core::ptr::null_mut(),
            extra_data_buffer_size: 0,
            host_event_fifo_buffer: core::ptr::null_mut(),
            host_event_fifo_buffer_size: 0,
            numAttrRecord: CFG_BLE_NUM_GATT_ATTRIBUTES,
            numAttrServ: CFG_BLE_NUM_GATT_SERVICES,
            attrValueArrSize: CFG_BLE_ATT_VALUE_ARRAY_SIZE,
            numOfLinks: CFG_BLE_NUM_LINK,
            prWriteListSize: CFG_BLE_PREPARE_WRITE_LIST_SIZE,
            mblockCount: mblock_count() as u16,
            max_add_eatt_bearers: CFG_BLE_EATT_BEARER_PER_LINK * CFG_BLE_NUM_LINK,
            attMtu: CFG_BLE_ATT_MTU_MAX,
            max_coc_mps: CFG_BLE_COC_MPS_MAX,
            max_coc_nbr: CFG_BLE_COC_NBR_MAX,
            max_coc_initiator_nbr: CFG_BLE_COC_INITIATOR_NBR_MAX,
            options: CFG_BLE_OPTIONS,
            debug: 0x10, // BLE_DEBUG_RAND_ADDR_INIT - required for random address support
        };

        #[cfg(feature = "defmt")]
        defmt::trace!("init_ble_stack: calling BleStack_Init");

        // 3. Initialize the BLE stack
        let status: tBleStatus = BleStack_Init(&init_params);

        if status != BLE_STATUS_SUCCESS {
            #[cfg(feature = "defmt")]
            defmt::error!("init_ble_stack: BleStack_Init failed with status 0x{:02X}", status);
            return Err(status);
        }

        #[cfg(feature = "defmt")]
        defmt::trace!("init_ble_stack: BleStack_Init succeeded");

        // 4. Call ll_sys_dependencies_init after BleStack_Init
        // This is required for deep sleep, background tasks, etc.
        ll_sys_dependencies_init();

        #[cfg(feature = "defmt")]
        defmt::info!("init_ble_stack: BLE stack initialized successfully");
    }

    Ok(())
}
// #endif /* SUPPORT_BLE */
// #if defined(MAC)
// #ifndef OPENTHREAD_CONFIG_FILE
#[cfg(feature = "wba_mac")]
/**
 * @brief  Initialize the Link Layer IP 802.15.4 MAC controller
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_mac_cntrl_init() {
    ST_MAC_preInit();
    ll_sys_dependencies_init();
}
// #endif /* OPENTHREAD_CONFIG_FILE */
// #endif /* MAC */
/**
 * @brief  Start the Link Layer IP in OpenThread configuration
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_thread_init() {
    ll_sys_dependencies_init();
}

/**
 * @brief  Initialize the Link Layer resources for startup.
 *         This includes: - Deep Sleep feature resources
 *                        - Link Layer background task
 * @param  None
 * @retval None
 */
unsafe fn ll_sys_dependencies_init() {
    static mut IS_LL_INITIALIZED: u8 = 0;
    let dp_slp_status: ll_sys_status_t;

    #[cfg(feature = "defmt")]
    defmt::trace!("ll_sys_dependencies_init: starting");

    /* Ensure Link Layer resources are created only once */
    if IS_LL_INITIALIZED == 1 {
        #[cfg(feature = "defmt")]
        defmt::trace!("ll_sys_dependencies_init: already initialized");
        return;
    }
    IS_LL_INITIALIZED = 1;

    /* Deep sleep feature initialization */
    #[cfg(feature = "defmt")]
    defmt::trace!("ll_sys_dependencies_init: calling ll_sys_dp_slp_init");
    dp_slp_status = ll_sys_dp_slp_init();
    #[cfg(feature = "defmt")]
    defmt::trace!(
        "ll_sys_dependencies_init: ll_sys_dp_slp_init done, status={}",
        dp_slp_status
    );
    ll_sys_assert((dp_slp_status == LL_SYS_STATUS_T_LL_SYS_OK) as u8);

    /* Background task initialization */
    #[cfg(feature = "defmt")]
    defmt::trace!("ll_sys_dependencies_init: calling ll_sys_bg_process_init");
    ll_sys_bg_process_init();
    #[cfg(feature = "defmt")]
    defmt::trace!("ll_sys_dependencies_init: ll_sys_bg_process_init done");

    /* Link Layer user parameters application */
    #[cfg(feature = "defmt")]
    defmt::trace!("ll_sys_dependencies_init: calling ll_sys_config_params");
    ll_sys_config_params();
    #[cfg(feature = "defmt")]
    defmt::trace!("ll_sys_dependencies_init: ll_sys_config_params done");
}
