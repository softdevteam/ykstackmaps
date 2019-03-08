#!/bin/sh

# Cleanup code should always happen
cleanup() {
    rm -rf ~/.rustup/
    rm -rf ~/.cargo/
}

# The build script
RUST_TOOLCHAIN="stable"

set -e 

curl -sSf "https://sh.rustup.rs" | sh -s -- --default-toolchain=$RUST_TOOLCHAIN -y
export PATH=/opt/llvm-stackmapv3-tools/build/bin/:$HOME/.cargo/bin:$PATH
export LLVM_READOBJ_PATH=/opt/llvm-stackmapv3-tools/build/bin/llvm-readobj
cargo test

trap cleanup EXIT
