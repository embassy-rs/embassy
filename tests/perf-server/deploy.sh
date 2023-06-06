#!/bin/bash

set -euxo pipefail

HOST=root@192.168.1.3

cargo build --release
ssh $HOST -- systemctl stop perf-server
scp target/release/perf-server $HOST:/root
scp perf-server.service $HOST:/etc/systemd/system/
ssh $HOST -- 'systemctl daemon-reload; systemctl restart perf-server'