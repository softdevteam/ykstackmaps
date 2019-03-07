#!/bin/sh
#
# Build script for continuous integration.

PATH=/opt/rust/bin:${PATH}
LLVM_READOBJ_PATH=/opt/llvm-stackmapv3-tools/build/bin/llvm-readobj

cargo test
