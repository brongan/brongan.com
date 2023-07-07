#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

trap 'kill 0' SIGINT
trunk serve --proxy-backend="http://[::1]:8081/api/" &
cargo watch -- cargo run --bin server -- --port 8081 --static-dir ./dist
