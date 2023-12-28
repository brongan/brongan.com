#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT
trunk serve --proxy-backend="http://[::1]:8081/api/" &
cargo watch -- cargo run --bin server -- --port 8081 --ssl-port 8443 --static-dir client/dist --cert-dir cert --dev
