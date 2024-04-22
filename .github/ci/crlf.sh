#!/bin/bash
## on push branch~=gh-readonly-queue/main/.*
## on pull_request

set -euo pipefail

FILES_WITH_CRLF=$(find  ! -path "./.git/*" -not -type d | xargs file -N | (grep " CRLF " || true))

if [ -z "$FILES_WITH_CRLF" ]; then
  echo -e "No files with CRLF endings found."
  exit 0
else
  NR_FILES=$(echo "$FILES_WITH_CRLF" | wc -l)
  echo -e "ERROR: Found ${NR_FILES} files with CRLF endings."
  echo "$FILES_WITH_CRLF"
  exit "$NR_FILES"
fi
