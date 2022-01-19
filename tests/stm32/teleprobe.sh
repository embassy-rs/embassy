echo Running target=$1 elf=$2
STATUSCODE=$(
    curl \
        -sS \
        --output /dev/stderr \
        --write-out "%{http_code}" \
        -H "Authorization: Bearer $TELEPROBE_TOKEN" \
        https://teleprobe.embassy.dev/targets/$1/run --data-binary @$2
)
echo
echo HTTP Status code: $STATUSCODE
test "$STATUSCODE" -eq 200
