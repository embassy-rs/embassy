INCLUDE memory.x

SECTIONS
{
  .header : AT(0)
  {
    LONG(0xaedb041d)
    LONG(0xaedb041d)
  } > IROM
}

_stext = ORIGIN(IROM) + 8;

INCLUDE riscv-link.x
