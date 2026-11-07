/*
 * build.rs decides in what cases to define __run_from_ram.
    * Normally, __run_from_ram is NOT defined and FLASH and RAM regions are set as you would expect.
    * When when `__run_from_ram` is defined, the machine code is placed in the main block of striped RAM and the writable data and stack use the 8 KiB scratch banks.

 * TODO This is a hack so that we don't need to maintain the entire link.x file that the cortex-m-rt crate includes this file from.
 * For more information see https://github.com/embassy-rs/embassy/issues/7334
 */
MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
    FLASH : ORIGIN = DEFINED(__run_from_ram) ? 0x20000000 : 0x10000100,
            LENGTH = DEFINED(__run_from_ram) ? 256K : 2048K - 0x100

    /* Pick one of the two options for RAM layout     */

    /* OPTION A: Use all RAM banks as one big block   */
    /* Reasonable, unless you are doing something     */
    /* really particular with DMA or other concurrent */
    /* access that would benefit from striping        */
    RAM   : ORIGIN = DEFINED(__run_from_ram) ? 0x20040000 : 0x20000000,
            LENGTH = DEFINED(__run_from_ram) ? 8K : 264K

    /* OPTION B: Keep the unstriped sections separate */
    /* RAM: ORIGIN = 0x20000000, LENGTH = 256K        */
    /* SCRATCH_A: ORIGIN = 0x20040000, LENGTH = 4K    */
    /* SCRATCH_B: ORIGIN = 0x20041000, LENGTH = 4K    */
}
