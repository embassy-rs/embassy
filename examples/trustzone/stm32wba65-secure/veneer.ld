EXTERN(secure_add);
EXTERN(secure_add_body);

ASSERT(__veneer_base == 0x0C0FF000, "
ERROR(trustzone/stm32wba65-secure): CMSE veneers must start at 0x0C0FF000");

ASSERT(__veneer_limit == 0x0C0FF020, "
ERROR(trustzone/stm32wba65-secure): CMSE veneer region must be exactly 32 bytes");
