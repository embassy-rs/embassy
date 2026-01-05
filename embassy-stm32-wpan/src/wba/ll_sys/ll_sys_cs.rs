use crate::bindings::link_layer::{
    LINKLAYER_PLAT_DisableIRQ, LINKLAYER_PLAT_DisableSpecificIRQ, LINKLAYER_PLAT_EnableIRQ,
    LINKLAYER_PLAT_EnableSpecificIRQ, LINKLAYER_PLAT_PhyStartClbr, LINKLAYER_PLAT_PhyStopClbr,
};

// /**
//   ******************************************************************************
//   * @file    ll_sys_cs.c
//   * @author  MCD Application Team
//   * @brief   Link Layer IP system interface critical sections management
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
// #include "linklayer_plat.h"
// #include "ll_sys.h"
// #include <stdint.h>
//
/**
 * @brief  Enable interrupts
 * @param  None
 * @retval None
 */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_enable_irq() {
    LINKLAYER_PLAT_EnableIRQ();
}
//
// /**
//   * @brief  Disable interrupts
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_disable_irq() {
    LINKLAYER_PLAT_DisableIRQ();
}
//
// /**
//   * @brief  Set the Current Interrupt Priority Mask.
//   *         All interrupts with low priority level will be masked.
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_enable_specific_irq(isr_type: u8) {
    LINKLAYER_PLAT_EnableSpecificIRQ(isr_type);
}
//
// /**
//   * @brief  Restore the previous interrupt priority level
//   * @param  None
//   * @retval None
//   */
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_disable_specific_irq(isr_type: u8) {
    LINKLAYER_PLAT_DisableSpecificIRQ(isr_type);
}
//
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_phy_start_clbr() {
    LINKLAYER_PLAT_PhyStartClbr();
}
//
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_phy_stop_clbr() {
    LINKLAYER_PLAT_PhyStopClbr();
}
