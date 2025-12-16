use crate::bindings::link_layer::{
    Evnt_timing_t, HostStack_Process, LINKLAYER_PLAT_AclkCtrl, LINKLAYER_PLAT_Assert, LINKLAYER_PLAT_ClockInit,
    LINKLAYER_PLAT_DelayUs, LINKLAYER_PLAT_GetRNG, LINKLAYER_PLAT_RCOStartClbr, LINKLAYER_PLAT_RCOStopClbr,
    LINKLAYER_PLAT_RequestTemperature, LINKLAYER_PLAT_SCHLDR_TIMING_UPDATE_NOT, LINKLAYER_PLAT_SetupRadioIT,
    LINKLAYER_PLAT_SetupSwLowIT, LINKLAYER_PLAT_StartRadioEvt, LINKLAYER_PLAT_StopRadioEvt,
    LINKLAYER_PLAT_TriggerSwLowIT, LINKLAYER_PLAT_WaitHclkRdy, MAX_NUM_CNCRT_STAT_MCHNS, emngr_can_mcu_sleep,
    emngr_handle_all_events, ll_sys_schedule_bg_process,
};

// /**
//   ******************************************************************************
//   * @file    ll_sys_intf.c
//   * @author  MCD Application Team
//   * @brief   Link Layer IP general system interface
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
// #include <stdint.h>
//
// #include "ll_sys.h"
// #include "linklayer_plat.h"
// #include "event_manager.h"
// #include "ll_intf.h"
//
/**
 * @brief  Initialize the Link Layer SoC dependencies
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_init() {
    LINKLAYER_PLAT_ClockInit();
}
//
/**
 * @brief  Blocking delay in us
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_delay_us(delay: u32) {
    LINKLAYER_PLAT_DelayUs(delay);
}

/**
 * @brief  Assert checking
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_assert(condition: u8) {
    LINKLAYER_PLAT_Assert(condition);
}

/**
 * @brief  Radio active clock management
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_radio_ack_ctrl(enable: u8) {
    LINKLAYER_PLAT_AclkCtrl(enable);
}

/**
 * @brief  Link Layer waits for radio bus clock ready
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_radio_wait_for_busclkrdy() {
    LINKLAYER_PLAT_WaitHclkRdy();
}

/**
 * @brief  Get RNG number for the Link Layer IP
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_get_rng(ptr_rnd: *mut u8, len: u32) {
    LINKLAYER_PLAT_GetRNG(ptr_rnd, len);
}

/**
 * @brief  Initialize the main radio interrupt
 * @param  intr_cb    radio interrupt callback to link with the radio IRQ
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_setup_radio_intr(intr_cb: ::core::option::Option<unsafe extern "C" fn()>) {
    LINKLAYER_PLAT_SetupRadioIT(intr_cb);
}

/**
 * @brief  Initialize the radio SW low interrupt
 * @param  intr_cb    radio SW low interrupt interrupt callback to link
 *                    with the defined interrupt vector
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_setup_radio_sw_low_intr(intr_cb: ::core::option::Option<unsafe extern "C" fn()>) {
    LINKLAYER_PLAT_SetupSwLowIT(intr_cb);
}

/**
 * @brief  Trigger the radio SW low interrupt
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_radio_sw_low_intr_trigger(priority: u8) {
    LINKLAYER_PLAT_TriggerSwLowIT(priority);
}

/**
 * @brief  Link Layer radio activity event notification
 * @param  start      start/end of radio event
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_radio_evt_not(start: u8) {
    if start != 0 {
        LINKLAYER_PLAT_StartRadioEvt();
    } else {
        LINKLAYER_PLAT_StopRadioEvt();
    }
}

/**
 * @brief  Link Layer RCO calibration notification
 * @param  start      start/end of RCO calibration
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_rco_clbr_not(start: u8) {
    if start != 0 {
        LINKLAYER_PLAT_RCOStartClbr();
    } else {
        LINKLAYER_PLAT_RCOStopClbr();
    }
}

/**
 * @brief  Link Layer temperature request
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_request_temperature() {
    LINKLAYER_PLAT_RequestTemperature();
}

/**
 * @brief  Link Layer background task pcoessing procedure
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_bg_process() {
    if emngr_can_mcu_sleep() == 0 {
        emngr_handle_all_events();

        HostStack_Process();
    }

    if emngr_can_mcu_sleep() == 0 {
        ll_sys_schedule_bg_process();
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_schldr_timing_update_not(p_evnt_timing: *mut Evnt_timing_t) {
    LINKLAYER_PLAT_SCHLDR_TIMING_UPDATE_NOT(p_evnt_timing);
}

/**
 * @brief  Get the number of concurrent state machines for the Link Layer
 * @param  None
 * @retval Supported number of concurrent state machines
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_get_concurrent_state_machines_num() -> u8 {
    return MAX_NUM_CNCRT_STAT_MCHNS as u8;
}
//
// __WEAK void HostStack_Process(void)
// {
//
// }
