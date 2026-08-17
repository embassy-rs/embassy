/*
Memory configuration for the MPS3-AN536 machine.

See https://github.com/qemu/qemu/blob/master/hw/arm/mps3r.c
*/

MEMORY {
    QSPI : ORIGIN = 0x08000000, LENGTH = 8M
    BRAM : ORIGIN = 0x10000000, LENGTH = 512K
    DDR  : ORIGIN = 0x20000000, LENGTH = 1536M
}

REGION_ALIAS("VECTORS", QSPI);
REGION_ALIAS("CODE", QSPI);
REGION_ALIAS("DATA", BRAM);
REGION_ALIAS("STACKS", BRAM);
PROVIDE(num_cores = 2);
PROVIDE(kmain2 = default_kmain2);

PROVIDE(_hyp_stack_size = 64);
PROVIDE(_und_stack_size = 64);
PROVIDE(_svc_stack_size = 64);
PROVIDE(_abt_stack_size = 64);
PROVIDE(_irq_stack_size = 64);
PROVIDE(_fiq_stack_size = 64);
PROVIDE(_sys_stack_size = 256K);
