#!/bin/sh

# Thin wrapper around convert_wav.py so one workflow works for bash users too.

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

exec python3 "$SCRIPT_DIR/convert_wav.py" "$@"
