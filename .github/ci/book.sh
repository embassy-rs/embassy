#!/bin/bash
## on push branch~=gh-readonly-queue/main/.*
## on pull_request

set -euxo pipefail

make -C docs
