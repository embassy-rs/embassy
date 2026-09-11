#![no_std]
#![no_main]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_nxp::Peri;
use embassy_nxp::casper::{CASPER, CasperDriver, Opcode};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_nxp::init(Default::default());

    let mut casper = CasperDriver::new(unsafe { Peri::new_unchecked(CASPER) });
    info!("CASPER tests starting...");

    test_sramx_read_write(&mut casper);
    test_sramx_clear(&mut casper);

    test_execute_copy(&mut casper);
    test_execute_zero(&mut casper);
    test_execute_xor(&mut casper);
    test_execute_double_basic(&mut casper);
    test_execute_double_multiword(&mut casper);
    test_execute_add_basic(&mut casper);
    test_execute_add_multiword(&mut casper);
    test_execute_sub_basic(&mut casper);
    test_execute_sub_multiword(&mut casper);
    test_execute_rsub_basic(&mut casper);
    test_execute_rsub_multiword(&mut casper);
    test_execute_mul_nosum_basic(&mut casper);
    test_execute_mul_nosum_multiword(&mut casper);
    test_execute_mul_sum_basic(&mut casper);
    test_execute_mul_sum_multiword(&mut casper);
    test_execute_mul_sum_vs_mul_fullsum(&mut casper);
    test_execute_mul_fullsum_basic(&mut casper);
    test_execute_mul_fullsum_multiword(&mut casper);
    test_execute_mul_reduce(&mut casper);

    test_copy_values(&mut casper);
    test_zero(&mut casper);
    test_xor(&mut casper);
    test_double(&mut casper);
    test_double_multiword(&mut casper);
    test_add(&mut casper);
    test_add_multiword(&mut casper);
    test_sub(&mut casper);
    test_sub_multiword(&mut casper);
    test_rsub(&mut casper);
    test_rsub_multiword(&mut casper);
    test_mul_nosum(&mut casper);
    test_mul_nosum_multiword(&mut casper);
    test_mul_sum(&mut casper);
    test_mul_sum_multiword(&mut casper);
    test_mul_fullsum(&mut casper);
    test_mul_fullsum_multiword(&mut casper);
    test_mul_reduce(&mut casper);

    info!("CASPER tests completed.");
}

fn test_sramx_read_write(casper: &mut CasperDriver<'_>) {
    // CASPER SRAMX read/write TEST

    // Test with some random data
    let a: u64 = 0x1122334455667788;
    let b: u32 = 0xAABBCCDD;

    // 1. Write data to SRAMX at a certain offset (0x000 for example)
    casper.write_dword(0x000usize, a);
    casper.write_word(0x000usize + 8, b);

    // 2. Read data back from SRAMX
    let read_a = casper.read_dword(0x000usize);
    let read_b = casper.read_word(0x000usize + 8);

    // 3. Verifying
    if read_a == a && read_b == b {
        info!("SRAMX read/write TEST: PASS");
    } else {
        error!("SRAMX read/write TEST: FAIL");
    }
}

fn test_sramx_clear(casper: &mut CasperDriver<'_>) {
    // Clear SRAMX TEST
    let c: u64 = 0xFFFFFFFFFFFFFFFF;
    let d: u32 = 0xFFFFFFFF;

    casper.write_dword(0x400usize, c);
    casper.write_word(0x400usize + 8, d);

    casper.clear(0x400usize, 12);
    let read_c = casper.read_dword(0x400usize);
    let read_d = casper.read_word(0x400usize + 8);

    if read_c == 0 && read_d == 0 {
        info!("SRAMX clear TEST: PASS");
    } else {
        error!("SRAMX clear TEST: FAIL");
    }
}

fn test_execute_copy(casper: &mut CasperDriver<'_>) {
    // COPY TEST - hardware test (execute_op_sync)
    let src0: u64 = 0x1122334455667788;
    let src1: u64 = 0x2211009988776655;

    casper.write_dword(0x000usize, src0);
    casper.write_dword(0x000usize + 8, src1);
    casper.clear(0x400usize, 16);

    casper.execute_op_sync(
        Opcode::Copy,
        1,          // iter = 1 (process 2 64-bit words)
        0x000usize, // ab_offset
        0,
        0x400usize, // res_offset
    );

    let dst0 = casper.read_dword(0x400usize);
    let dst1 = casper.read_dword(0x400usize + 8);

    if dst0 == src0 && dst1 == src1 {
        info!("Opcode::Copy TEST: PASS");
    } else {
        error!("Opcode::Copy TEST: FAIL");
    }
}

fn test_execute_zero(casper: &mut CasperDriver<'_>) {
    // ZERO TEST - hardware test (execute_op_sync)
    let src0: u64 = 0x1122334455667788;
    let src1: u64 = 0x2211009988776655;

    casper.write_dword(0x400usize, src0);
    casper.write_dword(0x400usize + 8, src1);

    casper.execute_op_sync(
        Opcode::Zero,
        1, // iter = 1 (process 2 64-bit words)
        0,
        0,
        0x400usize, // res_offset
    );

    let dst0 = casper.read_dword(0x400usize);
    let dst1 = casper.read_dword(0x400usize + 8);

    if dst0 == 0 && dst1 == 0 {
        info!("Opcode::Zero TEST: PASS");
    } else {
        error!("Opcode::Zero TEST: FAIL");
    }
}

fn test_execute_xor(casper: &mut CasperDriver<'_>) {
    // XOR TEST - hardware test (execute_op_sync)
    let a: u64 = 0x1122334455667788;
    let b: u64 = 0xFFFF0000AAAA5555;
    let expected = a ^ b;
    let c: u64 = 0xAABBCCDD11223344;
    let d: u64 = 0x00000000FFFFFFFF;
    let expected2 = c ^ d;

    casper.write_dword(0x000usize, a);
    casper.write_dword(0x000usize + 8, c);
    casper.write_dword(0x600usize, b);
    casper.write_dword(0x600usize + 8, d);

    casper.execute_op_sync(
        Opcode::Xor64,
        1,          // iter = 1 (2 XOR operations)
        0x000usize, // A (ab_offset)
        0,
        0x600usize, // R (res_offset)
    );

    let result = casper.read_dword(0x600usize);
    let result2 = casper.read_dword(0x600usize + 8);

    if result == expected && result2 == expected2 {
        info!("Opcode::Xor64 TEST: PASS");
    } else {
        error!("Opcode::Xor64 TEST: FAIL");
    }
}

fn test_execute_double_basic(casper: &mut CasperDriver<'_>) {
    // DOUBLE TEST - hardware test (execute_op_sync)
    let a: u64 = 0x1122334455667788;
    // let b: u64 = 5; // 2nd test with b = 0x0
    let (expected, carry) = a.overflowing_add(a);

    casper.write_dword(0x600usize, a);

    casper.execute_op_sync(
        Opcode::Double64,
        0, // iter = 0 (1 DOUBLE operation)
        0,
        0,
        0x600usize, // R (res_offset)
    );

    let result = casper.read_dword(0x600usize);
    let carry_bit = casper.carry();

    if (result == expected) && (carry_bit == carry) {
        info!("Opcode::Double64 TEST: PASS");
    } else {
        error!("Opcode::Double64 TEST: FAIL");
    }
}

fn test_execute_double_multiword(casper: &mut CasperDriver<'_>) {
    // DOUBLE WALKING (MULTIWORD) TEST - hardware test (execute_op_sync)
    let w0: u64 = 5;
    let w1: u64 = 0xF0F0F0F0F0F0F0F0;
    let (expected0, _) = w0.overflowing_add(w0);
    let (expected1, expected_carry) = w1.overflowing_add(w1);

    casper.write_dword(0x600usize, w0);
    casper.write_dword(0x600usize + 8, w1 as u64);

    casper.execute_op_sync(
        Opcode::Double64,
        1, // iter = 1 (2 DOUBLE operations - process 2 64-bit words)
        0,
        0,
        0x600usize, // R (res_offset)
    );

    let result0 = casper.read_dword(0x600usize);
    let result1 = casper.read_dword(0x600usize + 8);
    let carry = casper.carry();

    // info!(
    //     "Opcode::Double64 WALKING:\n\
    //     W0 = {:#x}, expected = {:#x}\n\
    //     W1 = {:#x}, expected = {:#x}\n\
    //     CARRY = {}, EXPECTED_CARRY = {}",
    //     result0,
    //     expected0,
    //     result1,
    //     expected1,
    //     carry, expected_carry
    // );

    if (result0 == expected0) && (result1 == expected1) && (carry == expected_carry) {
        info!("Opcode::Double64 MULTIWORD TEST: PASS");
    } else {
        error!("Opcode::Double64 MULTIWORD TEST: FAIL");
    }
}

fn test_execute_add_basic(casper: &mut CasperDriver<'_>) {
    // ADD TEST - hardware test (execute_op_sync)
    let a: u64 = 0xfffffffffffffff9;
    let b: u64 = 0x000000000000000a;
    let (expected, carry) = a.overflowing_add(b);

    casper.write_dword(0x000usize, a);
    casper.write_dword(0x600usize, b);

    casper.execute_op_sync(
        Opcode::Add64,
        0,          // iter = 0 (1 ADD operation: a + b)
        0x000usize, // ab_offset
        0,
        0x600usize, // res_offset
    );

    let result = casper.read_dword(0x600usize);
    let carry_bit = casper.carry();

    if (result == expected) && (carry_bit == carry) {
        info!("Opcode::Add64 TEST: PASS");
    } else {
        error!("Opcode::Add64 TEST: FAIL");
    }
}

fn test_execute_add_multiword(casper: &mut CasperDriver<'_>) {
    // ADD 128-bit (multiword) TEST - hardware test (execute_op_sync)
    // A = -7
    // B = 10
    // A = 0xFFFFFFFFFFFFFFFF_FFFFFFFFFFFFFFF9
    // B = 0x0000000000000000_000000000000000A
    // Expected:
    // R = 0x0000000000000000_0000000000000003
    // CARRY = TRUE

    let a0: u64 = 0xFFFFFFFFFFFFFFF9;
    let a1: u64 = 0xFFFFFFFFFFFFFFFF;
    let b0: u64 = 0x000000000000000A;
    let b1: u64 = 0x0000000000000000;
    let expected0: u64 = 0x0000000000000003;
    let expected1: u64 = 0x0000000000000000;

    // A = A1:A0
    casper.write_dword(0x000usize, a0);
    casper.write_dword(0x000usize + 8, a1);

    // R = B1:B0
    casper.write_dword(0x600usize, b0);
    casper.write_dword(0x600usize + 8, b1);

    casper.execute_op_sync(
        Opcode::Add64,
        1,          // 2 64-bit cycles
        0x000usize, // ab_offset
        0,
        0x600usize, // res_offset
    );

    let result0 = casper.read_dword(0x600usize);
    let result1 = casper.read_dword(0x600usize + 8);

    //let carry = casper.carry();
    // info!(
    //     "Opcode::Add64 128-bit TEST :\n\
    //     A = {:#018x}_{:016x}\n\
    //     B = {:#018x}_{:016x}\n\
    //     RESULT = {:#018x}_{:016x}\n\
    //     EXPECTED = {:#018x}_{:016x}\n\
    //     CARRY = {}",
    //     a1, a0,
    //     b1, b0,
    //     result1, result0,
    //     expected1, expected0,
    //     carry
    // );

    if result0 == expected0 && result1 == expected1 {
        info!("Opcode::Add64 MULTIWORD TEST: PASS");
    } else {
        error!("Opcode::Add64 MULTIWORD TEST: FAIL");
    }
}

fn test_execute_sub_basic(casper: &mut CasperDriver<'_>) {
    // SUB TEST - hardware test (execute_op_sync)

    // 1st test (no borrow)
    // let r: u64 = 0x1122334455667788;
    // let a: u64 = 0x0011223344556677;

    // 2nd test to check borrow
    let r: u64 = 0x0000000000000003;
    let a: u64 = 0x000000000000000A;

    let (expected, borrow) = r.overflowing_sub(a);
    casper.write_dword(0x000usize, a);
    casper.write_dword(0x600usize, r);

    casper.execute_op_sync(
        Opcode::Sub64,
        0,          // iter = 0 (1 64-bit cycle)
        0x000usize, // A (ab_offset)
        0,
        0x600usize, // R (res_offset)
    );

    let result = casper.read_dword(0x600usize);
    let carry = casper.carry();

    // info!(
    //         "Opcode::Sub64 TEST:\n\
    //         R = {:#018x}\n\
    //         A = {:#018x}\n\
    //         RESULT = {:#018x}\n\
    //         EXPECTED = {:#018x}\n\
    //         BORROW (EXPECTED_CARRY) = {}\n\
    //         BORROW (CARRY) = {}",
    //         r, a, result, expected, borrow, carry
    //     );

    if result == expected && carry == borrow {
        info!("Opcode::Sub64 TEST: PASS");
    } else {
        error!("Opcode::Sub64 TEST: FAIL");
    }
}

fn test_execute_sub_multiword(casper: &mut CasperDriver<'_>) {
    // SUB 128-bit (MULTIWORD) TEST - hardware test (execute_op_sync)
    // A = 0x0000000000000001_0000000000000000
    // B = 0x0000000000000000_0000000000000001
    // A - B =
    // 0x0000000000000000_FFFFFFFFFFFFFFFF
    // Low word:
    // 0 - 1 = FFFFFFFFFFFFFFFF, borrow = 1
    // High word:
    // 1 - 0 - borrow = 0
    // Final borrow = 0

    let a0: u64 = 0x0000000000000000;
    let a1: u64 = 0x0000000000000001;
    let b0: u64 = 0x0000000000000001;
    let b1: u64 = 0x0000000000000000;
    let expected0: u64 = 0xFFFFFFFFFFFFFFFF;
    let expected1: u64 = 0x0000000000000000;

    // A = A1:A0
    casper.write_dword(0x000usize, b0);
    casper.write_dword(0x000usize + 8, b1);

    // R = B1:B0
    casper.write_dword(0x600usize, a0);
    casper.write_dword(0x600usize + 8, a1);

    casper.execute_op_sync(
        Opcode::Sub64,
        1,          // iter = 1 (2 64-bit cycles)
        0x000usize, // A (ab_offset)
        0,
        0x600usize, // R (res_offset)
    );

    let result0 = casper.read_dword(0x600usize);
    let result1 = casper.read_dword(0x600usize + 8);
    let borrow = casper.carry();

    // info!(
    //     "Opcode::Sub64 128-bit (MULTIWORD) TEST:\n\
    //     A = {:#018x}_{:016x}\n\
    //     B = {:#018x}_{:016x}\n\
    //     RESULT = {:#018x}_{:016x}\n\
    //     EXPECTED = {:#018x}_{:016x}\n\
    //     BORROW = {}",
    //     a1, a0,
    //     b1, b0,
    //     result1, result0,
    //     expected1, expected0,
    //     borrow
    // );

    if result0 == expected0 && result1 == expected1 && !borrow {
        info!("Opcode::Sub64 MULTIWORD TEST: PASS");
    } else {
        error!("Opcode::Sub64 MULTIWORD TEST: FAIL");
    }
}

fn test_execute_rsub_basic(casper: &mut CasperDriver<'_>) {
    // RSUB TEST - hardware test (execute_op_sync)
    // 1st test (no borrow)
    let a: u64 = 20;
    let r: u64 = 7;

    // 2nd test to check borrow
    // let a: u64 = 11;
    // let r: u64 = 16;

    let (expected, expected_borrow) = a.overflowing_sub(r);
    casper.write_dword(0x000usize, a);
    casper.write_dword(0x600usize, r);

    casper.execute_op_sync(
        Opcode::Rsub64,
        0,          // iter = 0 (1 64-bit cycle)
        0x000usize, // A (ab_offset)
        0,
        0x600usize, // R (res_offset)
    );

    let result = casper.read_dword(0x600usize);
    let borrow = casper.carry();

    // info!("Opcode::Rsub64 TEST:\n\
    //     A = {:#x}\n\
    //     R = {:#x}\n\
    //     RESULT = {:#x}\n\
    //     EXPECTED = {:#x}\n\
    //     BORROW = {}",
    //     a, r, result, expected, borrow
    // );

    if result == expected && borrow == expected_borrow {
        info!("Opcode::Rsub64 TEST: PASS");
    } else {
        error!("Opcode::Rsub64 TEST: FAIL");
    }
}

fn test_execute_rsub_multiword(casper: &mut CasperDriver<'_>) {
    // RSUB 128-bit (MULTIWORD) TEST - hardware test (execute_op_sync)
    // A = 0x0000000000000001_0000000000000000
    // R = 0x0000000000000000_0000000000000001
    // A - R =
    // 0x0000000000000000_FFFFFFFFFFFFFFFF
    // Low:
    // 0 - 1 = FFFFFFFFFFFFFFFF, borrow = 1
    // High:
    // 1 - 0 - borrow = 0
    // Final borrow = 0

    let a0: u64 = 0x0000000000000000;
    let a1: u64 = 0x0000000000000001;
    let r0: u64 = 0x0000000000000001;
    let r1: u64 = 0x0000000000000000;
    let expected0: u64 = 0xFFFFFFFFFFFFFFFF;
    let expected1: u64 = 0x0000000000000000;

    // A = A1:A0
    casper.write_dword(0x000usize, a0);
    casper.write_dword(0x000usize + 8, a1);

    // R = R1:R0
    casper.write_dword(0x600usize, r0);
    casper.write_dword(0x600usize + 8, r1);

    casper.execute_op_sync(
        Opcode::Rsub64,
        1,          // iter = 1 (2 64-bit cycles)
        0x000usize, // A (ab_offset)
        0,
        0x600usize, // R (res_offset)
    );

    let result0 = casper.read_dword(0x600usize);
    let result1 = casper.read_dword(0x600usize + 8);
    let borrow = casper.carry();

    // info!(
    //     "Opcode::Rsub64 128-bit (MULTIWORD) TEST:\n\
    //     A = {:#018x}_{:016x}\n\
    //     R = {:#018x}_{:016x}\n\
    //     RESULT = {:#018x}_{:016x}\n\
    //     EXPECTED = {:#018x}_{:016x}\n\
    //     BORROW = {}",
    //     a1, a0,
    //     r1, r0,
    //     result1, result0,
    //     expected1, expected0,
    //     borrow
    // );

    if result0 == expected0 && result1 == expected1 && !borrow {
        info!("Opcode::Rsub64 MULTIWORD TEST: PASS");
    } else {
        error!("Opcode::Rsub64 MULTIWORD TEST: FAIL");
    }
}

fn test_execute_mul_nosum_basic(casper: &mut CasperDriver<'_>) {
    // MUL_NOSUM TEST - hardware test (execute_op_sync)
    // A = 0xFFFFFFFFFFFFFFFF
    // B = 0xFFFFFFFFFFFFFFFF
    // A * B = 0xFFFFFFFFFFFFFFFE_0000000000000001

    let a: u64 = 0xFFFFFFFFFFFFFFFF;
    let b: u64 = 0xFFFFFFFFFFFFFFFF;
    let expected_high: u64 = 0xFFFFFFFFFFFFFFFE;
    let expected_low: u64 = 0x0000000000000001;

    casper.write_dword(0x000usize, a);
    casper.write_dword(0x400usize, b);

    casper.execute_op_sync(
        Opcode::Mul64Nosum,
        0,          // iter = 0 (1 64-bit multiplication)
        0x000usize, // ab_offset
        0x400usize, // cd_offset
        0x600usize, // res_offset
    );

    let result_low = casper.read_dword(0x600usize);
    let result_high = casper.read_dword(0x600usize + 8);

    // info!(
    //     "Opcode::Mul64Nosum TEST:\n\
    //     A = {:#018x}\n\
    //     B = {:#018x}\n\
    //     RESULT = {:#018x}_{:016x}\n\
    //     EXPECTED = {:#018x}_{:016x}",
    //     a,
    //     b,
    //     result_high,
    //     result_low,
    //     expected_high,
    //     expected_low
    // );

    if result_low == expected_low && result_high == expected_high {
        info!("Opcode::Mul64Nosum TEST: PASS");
    } else {
        error!("Opcode::Mul64Nosum TEST: FAIL");
    }
}

fn test_execute_mul_nosum_multiword(casper: &mut CasperDriver<'_>) {
    // MUL_NOSUM WALKING J-LOOP (MULTIWORD) TEST - hardware test (execute_op_sync)
    // A = 2
    // B[0] = 4
    // B[1] = 3
    // j = 0:
    // A * B[0] = 2 * 4 = 8 = 0x0000000000000000_0000000000000008
    //                             w[1]                w[0]
    // j = 1:
    // A * B[1] = 2 * 3 = 6 = 0x0000000000000000_0000000000000006
    //                             w[2]                w[1]
    // Expected:
    // RES[0] = 0x0000000000000008
    // RES[1] = 0x0000000000000000 + 0x0000000000000006 = 0x0000000000000006
    // So w[1] from j=0 is added to w[1] from j=1 - this is how CASPER handles the "walking j-loop" when doing multiplication operations
    // RES[2] = 0x0000000000000000

    // let a: u64  = 2;
    // let b0: u64 = 4;
    // let b1: u64 = 3;

    // 2nd test to check overflow in the walking j-loop
    // A = 0xFFFFFFFFFFFFFFFF
    // B[0] = 0xFFFFFFFFFFFFFFFF
    // B[1] = 0xFFFFFFFFFFFFFFFE
    // j = 0:
    // W[0] = A * B[0] = 0xFFFFFFFFFFFFFFFF * 0xFFFFFFFFFFFFFFFF = 0xFFFFFFFFFFFFFFFE_0000000000000001
    // j = 1:
    // W[1] = A * B[1] = 0xFFFFFFFFFFFFFFFF * 0xFFFFFFFFFFFFFFFE = 0xFFFFFFFFFFFFFFFD_0000000000000002
    // Expected:
    // RES[0] = 0x0000000000000001
    // RES[1] = 0xFFFFFFFFFFFFFFFE + 0x0000000000000002 = 0x0000000000000000
    // RES[2] = 0xFFFFFFFFFFFFFFFD + 0x0000000000000001 (that carry from RES[1] due to overflow)= 0xFFFFFFFFFFFFFFFE
    let a: u64 = 0xFFFFFFFFFFFFFFFF;
    let b0: u64 = 0xFFFFFFFFFFFFFFFF;
    let b1: u64 = 0xFFFFFFFFFFFFFFFE;
    let expected0: u64 = 0x0000000000000001;
    let expected1: u64 = 0x0000000000000000;
    let expected2: u64 = 0xFFFFFFFFFFFFFFFE;

    casper.write_dword(0x000usize, a);
    casper.write_dword(0x200usize, b0);
    casper.write_dword(0x200usize + 8, b1);

    casper.execute_op_sync(
        Opcode::Mul64Nosum,
        1,          // iter = 1 (2 64-bit multiplications)
        0x000usize, // ab_offset
        0x200usize, // cd_offset
        0x600usize, // res_offset
    );

    let r0 = casper.read_dword(0x600usize);
    let r1 = casper.read_dword(0x600usize + 8);
    let r2 = casper.read_dword(0x600usize + 16);

    // info!(
    //     "Opcode::Mul64Nosum WALKING J-LOOP TEST:\n\
    //     RES+00 = {:#018x}\n\
    //     RES+08 = {:#018x}\n\
    //     RES+10 = {:#018x}",
    //     r0, r1, r2
    // );

    if r0 == expected0 && r1 == expected1 && r2 == expected2 {
        info!("Opcode::Mul64Nosum WALKING J-LOOP TEST: PASS");
    } else {
        error!("Opcode::Mul64Nosum WALKING J-LOOP TEST: FAIL");
    }
}

fn test_execute_mul_sum_basic(casper: &mut CasperDriver<'_>) {
    // MUL_SUM TEST - hardware test (execute_op_sync)

    // 1st test (simple numbers)
    // let a: u64 = 2;
    // let b: u64 = 3;
    // let initial: u128 = 5;

    // 2nd test (max numbers)
    let a: u64 = 0xFFFFFFFFFFFFFFFF;
    let b: u64 = 0xFFFFFFFFFFFFFFFF;
    let initial: u128 = 1;

    let expected = initial + (a as u128) * (b as u128);

    casper.write_dword(0x000usize, a);
    casper.write_dword(0x200usize, b);
    casper.write_dword(0x600usize, initial as u64);

    casper.execute_op_sync(
        Opcode::Mul64Sum,
        0,          // iter = 0 (1 64-bit multiplication)
        0x000usize, // ab_offset
        0x200usize, // cd_offset
        0x600usize, // res_offset
    );
    let result_low = casper.read_dword(0x600usize);
    let result_high = casper.read_dword(0x600usize + 8);
    let result = (result_high as u128) << 64 | result_low as u128;

    // info!(
    //     "Opcode::Mul64Sum TEST:\n\
    //     A = {:#018x}\n\
    //     B = {:#018x}\n\
    //     INITIAL = {:#x}\n\
    //     RESULT = {:#034x}\n\
    //     EXPECTED = {:#034x}",
    //     a,
    //     b,
    //     initial,
    //     result,
    //     expected
    // );

    if result == expected {
        info!("Opcode::Mul64Sum TEST: PASS");
    } else {
        error!("Opcode::Mul64Sum TEST: FAIL");
    }
}

fn test_execute_mul_sum_multiword(casper: &mut CasperDriver<'_>) {
    // MUL_SUM WALKING J-LOOP (MULTIWORD) TEST - hardware test (execute_op_sync)
    let a: u64 = 2;
    let b0: u64 = 3;
    let b1: u64 = 4;
    let w0: u64 = 10;
    let w1: u64 = 20;
    let expected0 = (w0 as u128) + (a as u128) * (b0 as u128);
    let expected1 = (w1 as u128) + (a as u128) * (b1 as u128);

    casper.write_dword(0x000usize, a);
    casper.write_dword(0x200usize, b0);
    casper.write_dword(0x200usize + 8, b1);
    casper.write_dword(0x600usize, w0);
    casper.write_dword(0x600usize + 8, w1);

    casper.execute_op_sync(
        Opcode::Mul64Sum,
        1, // 2 64-bit J iterations
        0x000usize,
        0x200usize,
        0x600usize,
    );

    let result0 = casper.read_dword(0x600usize);
    let result1 = casper.read_dword(0x600usize + 8);

    // info!(
    //     "Opcode::Mul64Sum WALKING TEST:\n\
    //     W0 = {:#018x}, expected = {:#018x}\n\
    //     W1 = {:#018x}, expected = {:#018x}",
    //     result0,
    //     expected0,
    //     result1,
    //     expected1
    // );

    if result0 == expected0 as u64 && result1 == expected1 as u64 {
        info!("Opcode::Mul64Sum WALKING J-LOOP TEST: PASS");
    } else {
        error!("Opcode::Mul64Sum WALKING J-LOOP TEST: FAIL");
    }
}

fn test_execute_mul_sum_vs_mul_fullsum(casper: &mut CasperDriver<'_>) {
    // MUL_FULLSUM vs MUL_SUM (MULL_SUM iter = 1 test) - hardware test (execute_op_sync)
    // iter = 1
    // A  = FFFFFFFFFFFFFFFF
    // B0 = FFFFFFFFFFFFFFFF
    // B1 = 0000000000000001
    // Initial result:
    // RES+00 = 1
    // RES+08 = 1
    // RES+10 = 1
    // RES+18 = 1
    // The expected result for MUL64_SUM is: RES+00 = 0x0000000000000002, RES+08 = 0xfffffffffffffffe, RES+10 = 0x0000000000000001, RES+18 = 0x0000000000000001.
    // The expected result for MUL64_FULLSUM is: RES+00 = 0x0000000000000002, RES+08 = 0xfffffffffffffffe, RES+10 = 0x0000000000000002, RES+18 = 0x0000000000000001
    // MUL64_SUM does not read the final 2 32-bit words (1 64-bit word).
    // Therefore the 64 bits of RES+10 (of the initial w[2] = 1, as for iter = 1, w[2] is the last 64-bit word) are ignored by the MUL64_SUM operation,
    // while the MUL64_FULLSUM operation reads all of w, including the MSWs. (Most Significant Words)

    let a: u64 = 0xFFFFFFFFFFFFFFFF;
    let b0: u64 = 0xFFFFFFFFFFFFFFFF;
    let b1: u64 = 0x0000000000000001;
    let w0: u64 = 0x0000000000000001;
    let w1: u64 = 0x0000000000000001;
    let w2: u64 = 0x0000000000000001;
    let w3: u64 = 0x0000000000000001;

    let expected0: u64 = 0x0000000000000002;
    let expected1: u64 = 0xfffffffffffffffe;
    let expected2: u64 = 0x0000000000000001;
    let expected3: u64 = 0x0000000000000001;

    casper.write_dword(0x000usize, a);
    casper.write_dword(0x200usize, b0);
    casper.write_dword(0x200usize + 8, b1);
    casper.write_dword(0x600usize, w0);
    casper.write_dword(0x600usize + 8, w1);
    casper.write_dword(0x600usize + 16, w2);
    casper.write_dword(0x600usize + 24, w3);

    casper.execute_op_sync(
        Opcode::Mul64Sum,
        1,          // iter = 1 (2 64-bit multiplications)
        0x000usize, // ab_offset
        0x200usize, // cd_offset
        0x600usize, // res_offset
    );
    let result0 = casper.read_dword(0x600usize);
    let result1 = casper.read_dword(0x600usize + 8);
    let result2 = casper.read_dword(0x600usize + 16);
    let result3 = casper.read_dword(0x600usize + 24);

    // info!(
    //     "Opcode::Mul64Sum vs Opcode::Mul64Fullsum TEST:\n\
    //     RES+00 = {:#018x}\n\
    //     RES+08 = {:#018x}\n\
    //     RES+10 = {:#018x}\n\
    //     RES+18 = {:#018x}",
    //     result0,
    //     result1,
    //     result2,
    //     result3
    // );

    if result0 == expected0 && result1 == expected1 && result2 == expected2 && result3 == expected3 {
        info!("Opcode::Mul64Sum vs Opcode::Mul64Fullsum TEST: PASS");
    } else {
        error!("Opcode::Mul64Sum vs Opcode::Mul64Fullsum TEST: FAIL");
    }
}

fn test_execute_mul_fullsum_basic(casper: &mut CasperDriver<'_>) {
    // MUL_FULLSUM TEST - hardware test (execute_op_sync)
    // W = W + A * B
    let a: u64 = 0xFFFF_FFFF_FFFF_FFFF;
    let b: u64 = 2;

    // Existing 128-bit W in result memory.
    // W = w_high:w_low
    let w_low: u64 = 0x0000000000000005;
    let w_high: u64 = 0x0000000000000003;

    let old_w: u128 = ((w_high as u128) << 64) | (w_low as u128);
    let product: u128 = (a as u128) * (b as u128);
    let expected: u128 = old_w + product;

    casper.write_dword(0x000usize, a);
    casper.write_dword(0x200usize, b);
    casper.write_dword(0x600usize, w_low);
    casper.write_dword(0x600usize + 8, w_high);

    casper.execute_op_sync(
        Opcode::Mul64Fullsum,
        0,          // iter = 0 (1 64-bit multiplication)
        0x000usize, // ab_offset
        0x200usize, // cd_offset
        0x600usize, // res_offset
    );

    let result_low = casper.read_dword(0x600usize);
    let result_high = casper.read_dword(0x600usize + 8);
    let result: u128 = ((result_high as u128) << 64) | (result_low as u128);

    // info!(
    //     "Opcode::Mul64Fullsum TEST:\n\
    //     A = {:#018x}\n\
    //     B = {:#018x}\n\
    //     OLD W = {:#034x}\n\
    //     PRODUCT = {:#034x}\n\
    //     EXPECTED = {:#034x}\n\
    //     RESULT = {:#034x}",
    //     a,
    //     b,
    //     old_w,
    //     product,
    //     expected,
    //     result
    // );

    if result == expected {
        info!("Opcode::Mul64Fullsum TEST: PASS");
    } else {
        error!("Opcode::Mul64Fullsum TEST: FAIL");
    }
}

fn test_execute_mul_fullsum_multiword(casper: &mut CasperDriver<'_>) {
    // MUL_FULLSUM vs MUL_SUM (MULL_FULLSUM iter = 1 test) - hardware test (execute_op_sync)
    // iter = 1
    // A  = FFFFFFFFFFFFFFFF
    // B0 = FFFFFFFFFFFFFFFF
    // B1 = 0000000000000001
    // Initial result:
    // RES+00 = 1
    // RES+08 = 1
    // RES+10 = 1
    // RES+18 = 1
    // The expected result for MUL64_FULLSUM is: RES+00 = 0x0000000000000002, RES+08 = 0xfffffffffffffffe, RES+10 = 0x0000000000000002, RES+18 = 0x0000000000000001
    // The expected result for MUL64_SUM is: RES+00 = 0x0000000000000002, RES+08 = 0xfffffffffffffffe, RES+10 = 0x0000000000000001, RES+18 = 0x0000000000000001,
    // since MUL64_SUM does not read the final 2 32-bit words (1 64-bit word).
    // Therefore the 64 bits of RES+10 (of the initial w[2] = 1, as for iter = 1 w[2] is the last 64-bit word) are ignored by the MUL64_SUM operation,
    // while the MUL64_FULLSUM operation reads all of w, including the MSWs. (Most Significant Word)

    let a: u64 = 0xFFFFFFFFFFFFFFFF;
    let b0: u64 = 0xFFFFFFFFFFFFFFFF;
    let b1: u64 = 0x0000000000000001;
    let w0: u64 = 0x0000000000000001;
    let w1: u64 = 0x0000000000000001;
    let w2: u64 = 0x0000000000000001;
    let w3: u64 = 0x0000000000000001;

    let expected0: u64 = 0x0000000000000002;
    let expected1: u64 = 0xfffffffffffffffe;
    let expected2: u64 = 0x0000000000000002;
    let expected3: u64 = 0x0000000000000001;

    casper.write_dword(0x000usize, a);
    casper.write_dword(0x200usize, b0);
    casper.write_dword(0x200usize + 8, b1);
    casper.write_dword(0x600usize, w0);
    casper.write_dword(0x600usize + 8, w1);
    casper.write_dword(0x600usize + 16, w2);
    casper.write_dword(0x600usize + 24, w3);

    casper.execute_op_sync(
        Opcode::Mul64Fullsum,
        1,          // iter = 1 (2 64-bit multiplications)
        0x000usize, // ab_offset
        0x200usize, // cd_offset
        0x600usize, // res_offset
    );
    let result0 = casper.read_dword(0x600usize);
    let result1 = casper.read_dword(0x600usize + 8);
    let result2 = casper.read_dword(0x600usize + 16);
    let result3 = casper.read_dword(0x600usize + 24);

    // info!(
    //     "Opcode::Mul64Fullsum WALKING J-LOOP TEST:\n\
    //     RES+00 = {:#018x}\n\
    //     RES+08 = {:#018x}\n\
    //     RES+10 = {:#018x}\n\
    //     RES+18 = {:#018x}",
    //     result0,
    //     result1,
    //     result2,
    //     result3
    // );

    if result0 == expected0 && result1 == expected1 && result2 == expected2 && result3 == expected3 {
        info!("Opcode::Mul64Fullsum WALKING J-LOOP TEST: PASS");
    } else {
        error!("Opcode::Mul64Fullsum WALKING J-LOOP TEST: FAIL");
    }
}

fn test_execute_mul_reduce(casper: &mut CasperDriver<'_>) {
    // MUL64_REDUCE TEST - hardware test (execute_op_sync)
    let _n: u64 = 3;
    let _modular_multiplicative_inverse: u64 = 0xaaaaaaaaaaaaaaab; // N^-1 mod 2^64 -> N (3) * N^-1 (0xaaaaaaaaaaaaaaab) mod 2^64 = 1
    let np: u64 = 0x5555555555555555; // N' = -N^-1 mod 2^64 -> 2^64 - 0xaaaaaaaaaaaaaaab = 0x5555555555555555
    let n0: u64 = 1;
    let n1: u64 = 2;
    let n2: u64 = 3;
    let w0: u64 = 2;
    let w1: u64 = 5;
    let w2: u64 = 6;
    let m: u64 = np.wrapping_mul(w0); // m = N' * W0 mod 2^64 = 0xaaaaaaaaaaaaaaaa

    let expected0: u64 = 0x5555555555555559;
    let expected1: u64 = 0x0000000000000005;
    let expected2: u64 = 0x0000000000000002;

    casper.write_dword(0x0000usize, m);

    casper.write_dword(0x0800usize, n0);
    casper.write_dword(0x0800usize + 8, n1);
    casper.write_dword(0x0800usize + 16, n2);

    casper.clear(0x1000usize, 24 * 2); // This operation may access additional RES 64-bit words beyond the n explicitly initialized W 64-bit words.
    casper.write_dword(0x1000usize, w0);
    casper.write_dword(0x1000usize + 8, w1);
    casper.write_dword(0x1000usize + 16, w2);

    casper.execute_op_sync(
        Opcode::Mul64Reduce,
        2,           // 3 64-bit J iterations
        0x0000usize, // ab_offset (m)
        0x0800usize, // cd_offset (N)
        0x1000usize, // res_offset (W)
    );
    let result0 = casper.read_dword(0x1000usize);
    let result1 = casper.read_dword(0x1000usize + 8);
    let result2 = casper.read_dword(0x1000usize + 16);

    // info!(
    //     "Opcode::Mul64Reduce:\n\
    //     N = {:#018x}\n\
    //     W0 = {:#018x}\n\
    //     N'= {:#018x}\n\
    //     M = {:#018x}\n\
    //     N0 = {:#018x}\n\
    //     N1 = {:#018x}\n\
    //     N2 = {:#018x}\n\
    //     RES+00 = {:#018x}\n\
    //     RES+08 = {:#018x}\n\
    //     RES+10 = {:#018x}", _n, w0, np, m, n0, n1, n2, result0, result1, result2
    // );

    if result0 == expected0 && result1 == expected1 && result2 == expected2 {
        info!("Opcode::Mul64Reduce TEST: PASS");
    } else {
        error!("Opcode::Mul64Reduce TEST: FAIL");
    }
}

fn test_copy_values(casper: &mut CasperDriver<'_>) {
    // COPY TEST - higher-level API test (copy_values() method)
    let src0: u64 = 0x8877665544332211;
    let src1: u64 = 0xAABBCCDD11223344;

    casper.copy_values(0x000usize, 0x400usize, &[src0, src1]);
    let result0 = casper.read_dword(0x400usize);
    let result1 = casper.read_dword(0x400usize + 8);
    // info!("Copy TEST:\n RES+00 = {:#018x}\n RES+08 = {:#018x}", result0, result1);
    if result0 == src0 && result1 == src1 {
        info!("Copy TEST: PASS");
    } else {
        error!("Copy TEST: FAIL");
    }
}

fn test_zero(casper: &mut CasperDriver<'_>) {
    // ZERO TEST - higher-level API test (zero() method)
    let src0: u64 = 0x1122334455667788;
    let src1: u64 = 0x2211009988776655;

    casper.write_dword(0x400usize, src0);
    casper.write_dword(0x400usize + 8, src1);
    let _test0 = casper.read_dword(0x400usize);
    let _test1 = casper.read_dword(0x400usize + 8);
    // info!("Before Zero: dst0: {:#018x}, dst1: {:#018x}", _test0, _test1);
    casper.zero(0x400usize, 2);

    let result0 = casper.read_dword(0x400usize);
    let result1 = casper.read_dword(0x400usize + 8);
    // info!("Zero TEST: res0: {:#018x}, res1: {:#018x}", result0, result1);
    if result0 == 0 && result1 == 0 {
        info!("Zero TEST: PASS");
    } else {
        error!("Zero TEST: FAIL");
    }
}

fn test_xor(casper: &mut CasperDriver<'_>) {
    // XOR TEST - higher-level API test (xor() method)
    let a: u64 = 0x1122334455667788;
    let b: u64 = 0xFFFF0000AAAA5555;
    let expected0 = a ^ b;
    let c: u64 = 0xAABBCCDD11223344;
    let d: u64 = 0x00000000FFFFFFFF;
    let expected1 = c ^ d;

    let mut result = [0u64; 2];
    casper.xor(&[(a, b), (c, d)], &mut result);

    // info!("XOR TEST expected: {:#018x} {:#018x}\n XOR TEST result: {:#018x} {:#018x}", expected0, expected1, result[0], result[1]);
    if result[0] == expected0 && result[1] == expected1 {
        info!("XOR TEST: PASS");
    } else {
        error!("XOR TEST: FAIL");
    }
}

fn test_double(casper: &mut CasperDriver<'_>) {
    // DOUBLE TEST - high-level API test (double() method)
    let a: u64 = 0x1122334455667788; // 2nd test with a = 0x0
    let (expected, carry_expected) = a.overflowing_add(a);

    let mut result = [0u64; 1];
    let carry = casper.double(&[a], &mut result);
    // info!("DOUBLE TEST: A = {:#x}\n RESULT = {:#x}\n EXPECTED = {:#x}\n CARRY = {}\n CARRY_EXPECTED = {}\n", a, result[0], expected, carry, carry_expected);

    if result[0] == expected && carry == carry_expected {
        info!("DOUBLE TEST: PASS");
    } else {
        error!("DOUBLE TEST: FAIL");
    }
}

fn test_double_multiword(casper: &mut CasperDriver<'_>) {
    // DOUBLE 128-bit (MULTIWORD) TEST - high-level API test (double() method)
    let w0: u64 = 5;
    let w1: u64 = 0xF0F0F0F0F0F0F0F0;
    let (expected0, _) = w0.overflowing_add(w0);
    let (expected1, expected_carry1) = w1.overflowing_add(w1);

    let mut result = [0u64; 2];
    let final_carry = casper.double(&[w0, w1], &mut result);

    // info!(
    //     "DOUBLE MULTIWORD TEST:\n\
    //     RESULT0 = {:#018x}, expected0 = {:#018x}\n\
    //     RESULT1 = {:#018x}, expected1 = {:#018x}\n\
    //     FINAL_CARRY = {}, EXPECTED_CARRY1 = {}",
    //     result[0],
    //     expected0,
    //     result[1],
    //     expected1,
    //     final_carry, expected_carry1
    // );

    if (result[0] == expected0) && (result[1] == expected1) && (final_carry == expected_carry1) {
        info!("DOUBLE MULTIWORD TEST: PASS");
    } else {
        error!("DOUBLE MULTIWORD TEST: FAIL");
    }
}

fn test_add(casper: &mut CasperDriver<'_>) {
    // ADD TEST - high-level API test (add() method)
    let a: u64 = 0xfffffffffffffff9;
    let b: u64 = 0x000000000000000a;
    let (expected, carry_expected) = a.overflowing_add(b);

    let mut result = [0u64; 1];
    let carry = casper.add(&[(a, b)], &mut result);
    // info!("ADD TEST: A = {:#018x}; B = {:#018x}; RESULT = {:#018x}, EXPECTED = {:#018x}, CARRY = {}, EXPECTED_CARRY = {}", a, b, result[0], expected, carry, carry_expected);

    if (result[0] == expected) && (carry == carry_expected) {
        info!("ADD TEST: PASS");
    } else {
        error!("ADD TEST: FAIL");
    }
}

fn test_add_multiword(casper: &mut CasperDriver<'_>) {
    // ADD 128-bit (MULTIWORD) TEST - high-level API test (add() method)
    // A = -7
    // B = 10
    // A = 0xFFFFFFFFFFFFFFFF_FFFFFFFFFFFFFFF9
    // B = 0x0000000000000000_000000000000000A
    // Expected:
    // R = 0x0000000000000000_0000000000000003
    // CARRY = TRUE
    let a0: u64 = 0xFFFFFFFFFFFFFFF9;
    let a1: u64 = 0xFFFFFFFFFFFFFFFF;
    let b0: u64 = 0x000000000000000A;
    let b1: u64 = 0x0000000000000000;
    let expected0: u64 = 0x0000000000000003;
    let expected1: u64 = 0x0000000000000000;
    let mut result = [0u64; 2];
    let carry = casper.add(&[(a0, b0), (a1, b1)], &mut result);

    // info!(
    //     "ADD MULTIWORD TEST:\n\
    //     A = {:#018x}_{:016x}\n\
    //     B = {:#018x}_{:016x}\n\
    //     RESULT = {:#018x}_{:016x}\n\
    //     EXPECTED = {:#018x}_{:016x}\n\
    //     CARRY = {}",
    //     a1, a0,
    //     b1, b0,
    //     result[1], result[0],
    //     expected1, expected0,
    //     carry
    // );

    if result[0] == expected0 && result[1] == expected1 && carry {
        info!("ADD MULTIWORD TEST: PASS");
    } else {
        error!("ADD MULTIWORD TEST: FAIL");
    }
}

fn test_sub(casper: &mut CasperDriver<'_>) {
    // SUB TEST - high-level API test (sub() method)
    // 1st test (no borrow)
    let r: u64 = 0x1122334455667788;
    let a: u64 = 0x0011223344556677;

    // 2nd test to check borrow
    // let r: u64 = 0x0000000000000003;
    // let a: u64 = 0x000000000000000A;

    let (expected, borrow) = r.overflowing_sub(a);
    let mut result = [0u64; 1];
    let carry = casper.sub(&[(r, a)], &mut result);

    // info!(
    //     "SUB TEST:\n\
    //     R = {:#018x}\n\
    //     A = {:#018x}\n\
    //     RESULT = {:#018x}\n\
    //     EXPECTED = {:#018x}\n\
    //     BORROW (EXPECTED_CARRY) = {}\n\
    //     REAL_BORROW (CARRY) = {}",
    //     r, a, result[0], expected, borrow, carry
    // );

    if result[0] == expected && carry == borrow {
        info!("SUB TEST: PASS");
    } else {
        error!("SUB TEST: FAIL");
    }
}

fn test_sub_multiword(casper: &mut CasperDriver<'_>) {
    // SUB 128-bit (MULTIWORD) TEST - high-level API test (sub() method)
    // A = 0x0000000000000001_0000000000000000
    // B = 0x0000000000000000_0000000000000001
    // A - B =
    // 0x0000000000000000_FFFFFFFFFFFFFFFF
    // Low word:
    // 0 - 1 = FFFFFFFFFFFFFFFF, borrow = 1
    // High word:
    // 1 - 0 - borrow = 0
    // Final borrow = 0
    let a0: u64 = 0x0000000000000000;
    let a1: u64 = 0x0000000000000001;
    let b0: u64 = 0x0000000000000001;
    let b1: u64 = 0x0000000000000000;
    let expected0: u64 = 0xFFFFFFFFFFFFFFFF;
    let expected1: u64 = 0x0000000000000000;

    let mut result = [0u64; 2];
    let borrow = casper.sub(&[(a0, b0), (a1, b1)], &mut result);

    // info!(
    //     "SUB MULTIWORD TEST:\n\
    //     A = {:#018x}_{:016x}\n\
    //     B = {:#018x}_{:016x}\n\
    //     RESULT = {:#018x}_{:016x}\n\
    //     EXPECTED = {:#018x}_{:016x}\n\
    //     BORROW = {}",
    //     a1, a0,
    //     b1, b0,
    //     result[1], result[0],
    //     expected1, expected0,
    //     borrow
    // );

    if result[0] == expected0 && result[1] == expected1 && !borrow {
        info!("SUB MULTIWORD TEST: PASS");
    } else {
        error!("SUB MULTIWORD TEST: FAIL");
    }
}

fn test_rsub(casper: &mut CasperDriver<'_>) {
    // RSUB TEST - high-level API test (rsub() method)
    // 1st test (no borrow)
    let a: u64 = 20;
    let r: u64 = 7;

    // 2nd test to check borrow
    // let a: u64 = 11;
    // let r: u64 = 16;
    let (expected, expected_borrow) = a.overflowing_sub(r);

    let mut result = [0u64; 1];
    let borrow = casper.rsub(&[(a, r)], &mut result);

    // info!(
    //     "RSUB TEST:\n\
    //     A = {:#018x}\n\
    //     R = {:#018x}\n\
    //     RESULT = {:#018x}\n\
    //     EXPECTED = {:#018x}\n\
    //     BORROW = {}\n\
    //     EXPECTED_BORROW = {}",
    //     a, r, result[0], expected, borrow, expected_borrow
    // );

    if result[0] == expected && borrow == expected_borrow {
        info!("RSUB TEST: PASS");
    } else {
        error!("RSUB TEST: FAIL");
    }
}

fn test_rsub_multiword(casper: &mut CasperDriver<'_>) {
    // RSUB 128-bit (MULTIWORD) TEST - high-level API test (rsub() method)
    // A = 0x0000000000000001_0000000000000000
    // R = 0x0000000000000000_0000000000000001
    // A - R =
    // 0x0000000000000000_FFFFFFFFFFFFFFFF
    // Low:
    // 0 - 1 = FFFFFFFFFFFFFFFF, borrow = 1
    // High:
    // 1 - 0 - borrow = 0
    // Final borrow = 0

    let a0: u64 = 0x0000000000000000;
    let a1: u64 = 0x0000000000000001;
    let r0: u64 = 0x0000000000000001;
    let r1: u64 = 0x0000000000000000;
    let expected0: u64 = 0xFFFFFFFFFFFFFFFF;
    let expected1: u64 = 0x0000000000000000;

    let mut result = [0u64; 2];
    let borrow = casper.rsub(&[(a0, r0), (a1, r1)], &mut result);

    // info!(
    //     "RSUB MULTIWORD TEST:\n\
    //     A = {:#018x}_{:016x}\n\
    //     R = {:#018x}_{:016x}\n\
    //     RESULT = {:#018x}_{:016x}\n\
    //     EXPECTED = {:#018x}_{:016x}\n\
    //     BORROW = {}",
    //     a1, a0,
    //     r1, r0,
    //     result[1], result[0],
    //     expected1, expected0,
    //     borrow
    // );

    if result[0] == expected0 && result[1] == expected1 && !borrow {
        info!("RSUB MULTIWORD TEST: PASS");
    } else {
        error!("RSUB MULTIWORD TEST: FAIL");
    }
}

fn test_mul_nosum(casper: &mut CasperDriver<'_>) {
    // MUL_NOSUM TEST - high-level API test (mul_nosum() method)
    // A = 0xFFFFFFFFFFFFFFFF
    // B = 0xFFFFFFFFFFFFFFFF
    // A * B = 0xFFFFFFFFFFFFFFFE_0000000000000001

    let a: u64 = 0xFFFFFFFFFFFFFFFF;
    let b: u64 = 0xFFFFFFFFFFFFFFFF;
    let expected_high: u64 = 0xFFFFFFFFFFFFFFFE;
    let expected_low: u64 = 0x0000000000000001;

    let mut result = [0u64; 2];
    casper.mul_nosum(a, &[b], &mut result);

    // info!(
    //     "MUL_NOSUM TEST:\n\
    //     A = {:#018x}\n\
    //     B = {:#018x}\n\
    //     RESULT = {:#018x}_{:016x}\n\
    //     EXPECTED = {:#018x}_{:016x}",
    //     a,
    //     b,
    //     result[1],
    //     result[0],
    //     expected_high,
    //     expected_low
    // );

    if result[0] == expected_low && result[1] == expected_high {
        info!("MUL_NOSUM TEST: PASS");
    } else {
        error!("MUL_NOSUM TEST: FAIL");
    }
}

fn test_mul_nosum_multiword(casper: &mut CasperDriver<'_>) {
    // MUL_NOSUM WALKING J-LOOP TEST - high-level API test (mul_nosum() method)
    // A = 2
    // B[0] = 4
    // B[1] = 3
    // j = 0:
    // A * B[0] = 2 * 4 = 8 = 0x0000000000000000_0000000000000008
    //                             w[1]                w[0]
    // j = 1:
    // A * B[1] = 2 * 3 = 6 = 0x0000000000000000_0000000000000006
    //                             w[2]                w[1]
    // Expected:
    // RES[0] = 0x0000000000000008
    // RES[1] = 0x0000000000000000 + 0x0000000000000006 = 0x0000000000000006
    // So w[1] from j=0 is added to w[1] from j=1 - this is how CASPER handles the "walking j-loop" when doing multiplication operations
    // RES[2] = 0x0000000000000000

    // 2nd test to check overflow in the walking j-loop
    // A = 0xFFFFFFFFFFFFFFFF
    // B[0] = 0xFFFFFFFFFFFFFFFF
    // B[1] = 0xFFFFFFFFFFFFFFFE
    // j = 0:
    // W[0] = A * B[0] = 0xFFFFFFFFFFFFFFFF * 0xFFFFFFFFFFFFFFFF = 0xFFFFFFFFFFFFFFFE_0000000000000001
    // j = 1:
    // W[1] = A * B[1] = 0xFFFFFFFFFFFFFFFF * 0xFFFFFFFFFFFFFFFE = 0xFFFFFFFFFFFFFFFD_0000000000000002
    // Expected:
    // RES[0] = 0x0000000000000001
    // RES[1] = 0xFFFFFFFFFFFFFFFE + 0x0000000000000002 = 0x0000000000000000
    // RES[2] = 0xFFFFFFFFFFFFFFFD + 0x0000000000000001 (that carry from RES[1] due to overflow)= 0xFFFFFFFFFFFFFFFE

    // let a: u64  = 2;
    // let b0: u64 = 4;
    // let b1: u64 = 3;

    let a: u64 = 0xFFFFFFFFFFFFFFFF;
    let b0: u64 = 0xFFFFFFFFFFFFFFFF;
    let b1: u64 = 0xFFFFFFFFFFFFFFFE;

    let expected0: u64 = 0x0000000000000001;
    let expected1: u64 = 0x0000000000000000;
    let expected2: u64 = 0xFFFFFFFFFFFFFFFE;
    let mut result = [0u64; 3];
    casper.mul_nosum(a, &[b0, b1], &mut result);

    // info!(
    //     "MUL_NOSUM WALKING J-LOOP TEST:\n\
    //     RES+00 = {:#018x}\n\
    //     RES+08 = {:#018x}\n\
    //     RES+10 = {:#018x}",
    //     result[0], result[1], result[2]
    // );

    if result[0] == expected0 && result[1] == expected1 && result[2] == expected2 {
        info!("MUL_NOSUM WALKING J-LOOP TEST: PASS");
    } else {
        error!("MUL_NOSUM WALKING J-LOOP TEST: FAIL");
    }
}

fn test_mul_sum(casper: &mut CasperDriver<'_>) {
    // MUL_SUM TEST - high-level API test (mul_sum() method)
    // 1st test (simple numbers)
    // let a: u64 = 2;
    // let b: u64 = 3;
    // let initial: u128 = 5;

    // 2nd test (max numbers)
    let a: u64 = 0xFFFFFFFFFFFFFFFF;
    let b: u64 = 0xFFFFFFFFFFFFFFFF;
    let initial: u128 = 1;

    let expected = initial + (a as u128) * (b as u128);
    let expected_low = expected as u64;
    let expected_high = (expected >> 64) as u64;

    let mut result = [0u64; 2];
    casper.mul_sum(a, &[b], &[initial as u64], &mut result);

    // info!(
    //     "MUL_SUM TEST:\n\
    //     A = {:#018x}\n\
    //     B = {:#018x}\n\
    //     INITIAL = {:#018x}\n\
    //     RESULT = {:#018x}_{:016x}\n\
    //     EXPECTED = {:#034x}",
    //     a,
    //     b,
    //     initial,
    //     result[1], result[0],
    //     expected
    // );

    if result[0] == expected_low && result[1] == expected_high {
        info!("MUL_SUM TEST: PASS");
    } else {
        error!("MUL_SUM TEST: FAIL");
    }
}

fn test_mul_sum_multiword(casper: &mut CasperDriver<'_>) {
    // MUL_SUM WALKING J-LOOP TEST - high-level API test (mul_sum() method)
    let a: u64 = 2;
    let b0: u64 = 3;
    let b1: u64 = 4;

    let w0: u64 = 10;
    let w1: u64 = 20;

    let expected0 = (w0 as u128) + (a as u128) * (b0 as u128);
    let expected1 = (w1 as u128) + (a as u128) * (b1 as u128);

    let mut result = [0u64; 3];
    casper.mul_sum(a, &[b0, b1], &[w0, w1], &mut result);

    // info!(
    //     "MUL_SUM WALKING J-LOOP TEST:\n\
    //     RESULT0 = {:#018x}, expected0 = {:#018x}\n\
    //     RESULT1 = {:#018x}, expected1 = {:#018x}",
    //     result[0],
    //     expected0,
    //     result[1],
    //     expected1
    // );

    if result[0] == expected0 as u64 && result[1] == expected1 as u64 {
        info!("MUL_SUM WALKING J-LOOP TEST: PASS");
    } else {
        error!("MUL_SUM WALKING J-LOOP TEST: FAIL");
    }
}

fn test_mul_fullsum(casper: &mut CasperDriver<'_>) {
    // MUL_FULLSUM TEST - high-level API test (mul_fullsum() method)
    // W = W + A * B
    let a: u64 = 0xFFFF_FFFF_FFFF_FFFF;
    let b: u64 = 2;

    // Existing 128-bit W in result memory.
    // W = w_high:w_low
    let w_low: u64 = 0x0000000000000005;
    let w_high: u64 = 0x0000000000000003;

    let old_w: u128 = ((w_high as u128) << 64) | (w_low as u128);
    let product: u128 = (a as u128) * (b as u128);
    let expected: u128 = old_w + product;
    let expected_low: u64 = expected as u64;
    let expected_high: u64 = (expected >> 64) as u64;
    // CARRY = FALSE

    let mut result = [0u64; 2];
    let carry = casper.mul_fullsum(a, &[b], &[w_low, w_high], &mut result);

    // info!(
    //     "MUL_FULLSUM TEST:\n\
    //     A = {:#018x}\n\
    //     B = {:#018x}\n\
    //     OLD W = {:#034x}\n\
    //     PRODUCT = {:#034x}\n\
    //     EXPECTED = {:#018x}_{:016x}\n\
    //     RESULT = {:#018x}_{:016x}\n\
    //     CARRY = {}",
    //     a,
    //     b,
    //     old_w,
    //     product,
    //     expected_high, expected_low,
    //     result[1], result[0], carry
    // );

    if result[0] == expected_low && result[1] == expected_high && !carry {
        info!("MUL_FULLSUM TEST: PASS");
    } else {
        error!("MUL_FULLSUM TEST: FAIL");
    }
}

fn test_mul_fullsum_multiword(casper: &mut CasperDriver<'_>) {
    // MUL_FULLSUM vs MUL_SUM (MULL_FULLSUM iter = 1 test) - high-level API test (mul_fullsum() method)
    // iter = 1
    // A  = FFFFFFFFFFFFFFFF
    // B0 = FFFFFFFFFFFFFFFF
    // B1 = 0000000000000001
    // Initial result:
    // RES+00 = 1
    // RES+08 = 1
    // RES+10 = 1
    // The expected result for MUL64_FULLSUM is: RES+00 = 0x0000000000000002, RES+08 = 0xfffffffffffffffe, RES+10 = 0x0000000000000002.
    // The expected result for MUL64_SUM is: RES+00 = 0x0000000000000002, RES+08 = 0xfffffffffffffffe, RES+10 = 0x0000000000000001,
    // since MUL64_SUM does not read the final 2 32-bit words (1 64-bit word).
    // Therefore the 64 bits of RES+10 (of the initial w[2] = 1, as for iter = 1 w[2] is the last 64-bit word) are ignored by the MUL64_SUM operation,
    // while the MUL64_FULLSUM operation reads all of w, including the MSWs. (Most Significant Word)

    let a: u64 = 0xFFFFFFFFFFFFFFFF;
    let b0: u64 = 0xFFFFFFFFFFFFFFFF;
    let b1: u64 = 0x0000000000000001;
    let w0: u64 = 0x0000000000000001;
    let w1: u64 = 0x0000000000000001;
    let w2: u64 = 0x0000000000000001;

    let mut result = [0u64; 3];
    let carry = casper.mul_fullsum(a, &[b0, b1], &[w0, w1, w2], &mut result);

    let expected0: u64 = 0x0000000000000002;
    let expected1: u64 = 0xfffffffffffffffe;
    let expected2: u64 = 0x0000000000000002;
    // EXPECTED_CARRY = FALSE

    // info!(
    //     "MUL_FULLSUM WALKING J-LOOP TEST:\n\
    //     RES+00 = {:#018x}\n\
    //     RES+08 = {:#018x}\n\
    //     RES+10 = {:#018x}",
    //     result[0],
    //     result[1],
    //     result[2]
    // );

    if result[0] == expected0 && result[1] == expected1 && result[2] == expected2 && !carry {
        info!("MUL_FULLSUM WALKING J-LOOP TEST: PASS");
    } else {
        error!("MUL_FULLSUM WALKING J-LOOP TEST: FAIL");
    }
}

fn test_mul_reduce(casper: &mut CasperDriver<'_>) {
    // MUL_REDUCE TEST - high-level API test (mul_reduce() method)
    let _n: u64 = 3;
    let _modular_multiplicative_inverse: u64 = 0xaaaaaaaaaaaaaaab; // N^-1 mod 2^64 -> N (3) * N^-1 (0xaaaaaaaaaaaaaaab) mod 2^64 = 1
    let np: u64 = 0x5555555555555555; // N' = -N^-1 mod 2^64 -> 2^64 - 0xaaaaaaaaaaaaaaab = 0x5555555555555555
    let n0: u64 = 1;
    let n1: u64 = 2;
    let n2: u64 = 3;
    let w0: u64 = 2;
    let w1: u64 = 5;
    let w2: u64 = 6;
    let m: u64 = np.wrapping_mul(w0); // m = N' * W0 mod 2^64 = 0xaaaaaaaaaaaaaaaa

    let expected0: u64 = 0x5555555555555559;
    let expected1: u64 = 0x0000000000000005;
    let expected2: u64 = 0x0000000000000002;

    let mut result = [0u64; 3];
    casper.mul_reduce(m, &[n0, n1, n2], &[w0, w1, w2], &mut result);

    // info!(
    //     "MUL_REDUCE TEST:\n\
    //     N = {:#018x}\n\
    //     W0 = {:#018x}\n\
    //     N'= {:#018x}\n\
    //     M = {:#018x}\n\
    //     N0 = {:#018x}\n\
    //     N1 = {:#018x}\n\
    //     N2 = {:#018x}\n\
    //     RES+00 = {:#018x}\n\
    //     RES+08 = {:#018x}\n\
    //     RES+10 = {:#018x}", _n, w0, np, m, n0, n1, n2, result[0], result[1], result[2]);

    if result[0] == expected0 && result[1] == expected1 && result[2] == expected2 {
        info!("MUL_REDUCE TEST: PASS");
    } else {
        error!("MUL_REDUCE TEST: FAIL");
    }
}
