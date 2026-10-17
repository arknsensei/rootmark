#!/usr/bin/env bash
# Cross-compiles `mdk` for x86_64-unknown-linux-gnu inside an arm64 Debian
# trixie Rust container (scripts/release.sh): the prebuilt onnxruntime of
# ort =2.0.0-rc.13 needs glibc >= 2.38 and GCC >= 13 libstdc++.
set -euo pipefail
apt-get update -qq >/dev/null
apt-get install -y -qq gcc-x86-64-linux-gnu g++-x86-64-linux-gnu >/dev/null
rustup target add x86_64-unknown-linux-gnu >/dev/null
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
export CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++
export AR_x86_64_unknown_linux_gnu=x86_64-linux-gnu-ar
cargo build --release --locked --target x86_64-unknown-linux-gnu -p mdkernel-server --bin mdk
