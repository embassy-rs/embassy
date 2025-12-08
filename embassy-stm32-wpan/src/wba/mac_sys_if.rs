#![cfg(feature = "wba")]
#![allow(non_snake_case)]

//
// /* USER CODE BEGIN Header */
// /**
//   ******************************************************************************
//   * @file    mac_sys_if.c
//   * @author  MCD Application Team
//   * @brief   Source file for using MAC Layer with a RTOS
//   ******************************************************************************
//   * @attention
//   *
//   * Copyright (c) 2025 STMicroelectronics.
//   * All rights reserved.
//   *
//   * This software is licensed under terms that can be found in the LICENSE file
//   * in the root directory of this software component.
//   * If no LICENSE file comes with this software, it is provided AS-IS.
//   *
//   ******************************************************************************
//   */
// /* USER CODE END Header */
//
// #include "main.h"
// #include "app_common.h"
// #include "app_conf.h"
// #include "log_module.h"
// #include "stm32_rtos.h"
// #include "st_mac_802_15_4_sys.h"
//
// extern void mac_baremetal_run(void);
//
// /* Private defines -----------------------------------------------------------*/
// /* USER CODE BEGIN PD */
//
// /* USER CODE END PD */
//
// /* Private macros ------------------------------------------------------------*/
// /* USER CODE BEGIN PM */
//
// /* USER CODE END PM */
//
// /* Private variables ---------------------------------------------------------*/
// /* USER CODE BEGIN PV */
//
// /* USER CODE END PV */
//
// /* Global variables ----------------------------------------------------------*/
// /* USER CODE BEGIN GV */
//
// /* USER CODE END GV */
//
// /* Functions Definition ------------------------------------------------------*/
//
// /**
//   * @brief  Mac Layer Initialisation
//   * @param  None
//   * @retval None
//   */
// void MacSys_Init(void)
// {
//   /* Register tasks */
//   UTIL_SEQ_RegTask( TASK_MAC_LAYER, UTIL_SEQ_RFU, mac_baremetal_run);
// }
//
// /**
//   * @brief  Mac Layer Resume
//   * @param  None
//   * @retval None
//   */
// void MacSys_Resume(void)
// {
//   UTIL_SEQ_ResumeTask( TASK_MAC_LAYER );
// }
//
// /**
//   * @brief  MAC Layer set Task.
//   * @param  None
//   * @retval None
//   */
// void MacSys_SemaphoreSet(void)
// {
//   UTIL_SEQ_SetTask( TASK_MAC_LAYER, TASK_PRIO_MAC_LAYER );
// }
//
// /**
//   * @brief  MAC Layer Task wait.
//   * @param  None
//   * @retval None
//   */
// void MacSys_SemaphoreWait( void )
// {
//   /* Not used */
// }
//
// /**
//   * @brief  MAC Layer set Event.
//   * @param  None
//   * @retval None
//   */
// void MacSys_EventSet( void )
// {
//   UTIL_SEQ_SetEvt( EVENT_MAC_LAYER );
// }
//
// /**
//   * @brief  MAC Layer wait Event.
//   * @param  None
//   * @retval None
//   */
// void MacSys_EventWait( void )
// {
//   UTIL_SEQ_WaitEvt( EVENT_MAC_LAYER );
// }
//

use super::util_seq;
use crate::bindings::mac;

/// Placeholder value used by the original ST middleware when registering tasks.
const UTIL_SEQ_RFU: u32 = 0;

/// Bit mask identifying the MAC layer task within the sequencer.
const TASK_MAC_LAYER_MASK: u32 = 1 << mac::CFG_TASK_ID_T_CFG_TASK_MAC_LAYER;

/// Sequencer priority assigned to the MAC layer task.
const TASK_PRIO_MAC_LAYER: u32 = mac::CFG_SEQ_PRIO_ID_T_CFG_SEQ_PRIO_0 as u32;

/// Event flag consumed by the MAC task while waiting on notifications.
const EVENT_MAC_LAYER_MASK: u32 = 1 << 0;

/// Registers the MAC bare-metal runner with the lightweight sequencer.
///
/// Mirrors the behaviour of the reference implementation:
/// `UTIL_SEQ_RegTask(TASK_MAC_LAYER, UTIL_SEQ_RFU, mac_baremetal_run);`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn MacSys_Init() {
    util_seq::UTIL_SEQ_RegTask(TASK_MAC_LAYER_MASK, UTIL_SEQ_RFU, Some(mac::mac_baremetal_run));
}

/**
 * @brief  Mac Layer Resume
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn MacSys_Resume() {
    util_seq::UTIL_SEQ_ResumeTask(TASK_MAC_LAYER_MASK);
}

/**
 * @brief  MAC Layer set Task.
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn MacSys_SemaphoreSet() {
    util_seq::UTIL_SEQ_SetTask(TASK_MAC_LAYER_MASK, TASK_PRIO_MAC_LAYER);
}

/**
 * @brief  MAC Layer Task wait.
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn MacSys_SemaphoreWait() {}

/**
 * @brief  MAC Layer set Event.
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn MacSys_EventSet() {
    util_seq::UTIL_SEQ_SetEvt(EVENT_MAC_LAYER_MASK);
}

/**
 * @brief  MAC Layer wait Event.
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
pub unsafe extern "C" fn MacSys_EventWait() {
    util_seq::UTIL_SEQ_WaitEvt(EVENT_MAC_LAYER_MASK);
}
