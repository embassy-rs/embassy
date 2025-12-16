use crate::bindings::link_layer::{
    LL_SYS_BRIEF_VERSION_MAJOR, LL_SYS_BRIEF_VERSION_MAJOR_MASK, LL_SYS_BRIEF_VERSION_MAJOR_POS,
    LL_SYS_BRIEF_VERSION_MINOR, LL_SYS_BRIEF_VERSION_MINOR_MASK, LL_SYS_BRIEF_VERSION_MINOR_POS,
    LL_SYS_BRIEF_VERSION_PATCH, LL_SYS_BRIEF_VERSION_PATCH_MASK, LL_SYS_BRIEF_VERSION_PATCH_POS,
};

// /**
//   ******************************************************************************
//   * @file    ll_version.c
//   * @author  MCD Application Team
//   * @brief   Link Layer version interface
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
//
// /* Includes ------------------------------------------------------------------*/
// /* Integer types */
// #include <stdint.h>
//
// /* Own header file  */
// #include "ll_version.h"
//
// /* Temporary header file for version tracking */
// #include "ll_tmp_version.h"
//
// /* Private defines -----------------------------------------------------------*/
// /**
//  * @brief Magic keyword to identify the system version when debugging
//  */
//  #define LL_SYS_MAGIC_KEYWORD  0xDEADBEEF

const LL_SYS_MAGIC_KEYWORD: u32 = 0xDEADBEEF;

//
// /* Private macros ------------------------------------------------------------*/
// /* Macro to set a specific field value */
// #define LL_SYS_SET_FIELD_VALUE(value, mask, pos) \
//   (((value) << (pos)) & (mask))

macro_rules! LL_SYS_SET_FIELD_VALUE {
    ($value:expr, $mask:expr, $pos:expr) => {
        ((($value) << ($pos)) & ($mask))
    };
}

//
// /* Private typedef -----------------------------------------------------------*/
// /**
//   * @brief Link Layer system version structure definition
//   */
#[allow(non_camel_case_types)]
struct ll_sys_version_t {
    #[allow(unused)]
    magic_key_word: u32, /* Magic key word to identify the system version */
    version: u32, /* System version - i.e.: short hash of latest commit */
}
//
// /* Private variables ---------------------------------------------------------*/
// /**
//  * @brief Link Layer brief version definition
//  */
const LL_SYS_BRIEF_VERSION: u8 = LL_SYS_SET_FIELD_VALUE!(
    LL_SYS_BRIEF_VERSION_MAJOR as u8,
    LL_SYS_BRIEF_VERSION_MAJOR_MASK as u8,
    LL_SYS_BRIEF_VERSION_MAJOR_POS as u8
) | LL_SYS_SET_FIELD_VALUE!(
    LL_SYS_BRIEF_VERSION_MINOR as u8,
    LL_SYS_BRIEF_VERSION_MINOR_MASK as u8,
    LL_SYS_BRIEF_VERSION_MINOR_POS as u8
) | LL_SYS_SET_FIELD_VALUE!(
    LL_SYS_BRIEF_VERSION_PATCH as u8,
    LL_SYS_BRIEF_VERSION_PATCH_MASK as u8,
    LL_SYS_BRIEF_VERSION_PATCH_POS as u8
);
//
// /**
//  * @brief Link Layer system version structure definition
//  */
const LL_SYS_SYSTEM_VERSION: ll_sys_version_t = ll_sys_version_t {
    magic_key_word: LL_SYS_MAGIC_KEYWORD,
    version: 0, // LL_SYS_SYSTEM_VERSION,
};
//
// /**
//  * @brief Link Layer source version structure definition
//  */
const LL_SYS_SOURCE_VERSION: ll_sys_version_t = ll_sys_version_t {
    magic_key_word: LL_SYS_MAGIC_KEYWORD,
    version: 0, // LL_SYS_SOURCE_VERSION
};
//
// /* Functions Definition ------------------------------------------------------*/
#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_get_brief_fw_version() -> u8 {
    return LL_SYS_BRIEF_VERSION;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_get_system_fw_version() -> u32 {
    return LL_SYS_SYSTEM_VERSION.version;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn ll_sys_get_source_fw_version() -> u32 {
    return LL_SYS_SOURCE_VERSION.version;
}
