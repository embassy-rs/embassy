# Remove duplicates from a string containing compilation flags
function(remove_duplicated_flags FLAGS UNIQFLAGS)
    set(FLAGS_LIST "${FLAGS}")
    # Convert the given flags, as a string, into a CMake list type
    separate_arguments(FLAGS_LIST)
    # Remove all the duplicated flags
    list(REMOVE_DUPLICATES FLAGS_LIST)
    # Convert the list back to a string
    string(REPLACE ";" " " FLAGS_LIST "${FLAGS_LIST}")
    # Return that string to the caller
    set(${UNIQFLAGS} "${FLAGS_LIST}" PARENT_SCOPE)
endfunction()

##############################################################################

set(CMAKE_SYSTEM_NAME Generic)

set(CMAKE_C_COMPILER xtensa-esp32-elf-gcc)
set(CMAKE_CXX_COMPILER xtensa-esp32-elf-g++)
set(CMAKE_ASM_COMPILER xtensa-esp32-elf-gcc)
set(_CMAKE_TOOLCHAIN_PREFIX xtensa-esp32-elf-)

remove_duplicated_flags("-mlongcalls -Wno-frame-address \
                         -fno-builtin-memcpy -fno-builtin-memset -fno-builtin-bzero \
                         -fno-builtin-stpcpy -fno-builtin-strncpy \
                         ${CMAKE_C_FLAGS}" UNIQ_CMAKE_C_FLAGS)
set(CMAKE_C_FLAGS "${UNIQ_CMAKE_C_FLAGS}" CACHE STRING "C Compiler Base Flags" FORCE)
remove_duplicated_flags("-mlongcalls -Wno-frame-address \
                         -fno-builtin-memcpy -fno-builtin-memset -fno-builtin-bzero \
                         -fno-builtin-stpcpy -fno-builtin-strncpy \
                         ${CMAKE_CXX_FLAGS}" UNIQ_CMAKE_CXX_FLAGS)
set(CMAKE_CXX_FLAGS "${UNIQ_CMAKE_CXX_FLAGS}" CACHE STRING "C++ Compiler Base Flags" FORCE)
remove_duplicated_flags("-mlongcalls ${CMAKE_ASM_FLAGS}" UNIQ_CMAKE_ASM_FLAGS)
set(CMAKE_ASM_FLAGS "${UNIQ_CMAKE_ASM_FLAGS}" CACHE STRING "ASM Compiler Base Flags" FORCE)


set(CMAKE_EXE_LINKER_FLAGS "-nostdlib" CACHE STRING "Linker Base Flags")
