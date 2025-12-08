cp $1 $1.elf

ELF_FILE="$1.elf"

JLinkExe <<EOF
Device nrf54lm20a_m33
SelectInterface SWD
Speed 4000
LoadFile ${ELF_FILE}
r
g
q
EOF

defmt-print -e $1 tcp