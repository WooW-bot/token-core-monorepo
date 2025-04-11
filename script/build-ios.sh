#!/bin/bash

set -e # 发生错误时终止脚本
PROJECT_ROOT="$(pwd)/"
GITHUB_WORKSPACE="$PROJECT_ROOT"

echo "=== 设置Rust工具链 ==="
rustup toolchain install nightly-2023-06-15
#rustup default nightly-2023-06-15-x86_64-apple-darwin
rustup default nightly-2023-06-15
rustup target add aarch64-apple-ios
rustup show

# 检查cargo-lipo和cbindgen是否已安装
if ! command -v cargo-lipo &> /dev/null; then
    echo "安装cargo-lipo..."
    cargo install cargo-lipo
fi

if ! command -v cbindgen &> /dev/null; then
    echo "安装cbindgen..."
    cargo install cbindgen --version 0.26.0
fi

# 检查protobuf是否安装
if ! command -v protoc &> /dev/null; then
    echo "安装protobuf..."
    brew install protobuf
fi

echo "=== 构建过程开始 ==="
# 第一步：构建imkey-core库
echo "=== 构建imkey核心库 ==="
pushd "$GITHUB_WORKSPACE/imkey-core/ikc"
cargo lipo --release --targets aarch64-apple-ios
# 生成头文件
cbindgen ./src/lib.rs -l c > "$GITHUB_WORKSPACE/target/connector.h"
popd

# 第二步：构建token-core库
echo "=== 构建token-core库 ==="
pushd "$GITHUB_WORKSPACE/token-core/tcx"
cargo lipo --release --targets aarch64-apple-ios
# 生成头文件
cbindgen ./src/lib.rs -l c > "$GITHUB_WORKSPACE/target/tcx.h"
popd

# 第三步：将构建结果复制到库目录
echo "=== 复制构建结果 ==="
# imkey-core库
LIBS_IKC="$GITHUB_WORKSPACE/imkey-core/mobile-sdk/imKeyCoreX/imKeyCoreX"
mkdir -p "$LIBS_IKC"
cp "$GITHUB_WORKSPACE/target/universal/release/libconnector.a" "$LIBS_IKC/libconnector.a"
cp "$GITHUB_WORKSPACE/target/connector.h" "$LIBS_IKC/connector.h"
# token-core库
LIBS_TCX="$GITHUB_WORKSPACE/token-core/tcx-examples/TokenCoreX/TokenCoreX"
mkdir -p "$LIBS_TCX"
cp "$GITHUB_WORKSPACE/target/universal/release/libtcx.a" "$LIBS_TCX/libtcx.a"
cp "$GITHUB_WORKSPACE/target/tcx.h" "$LIBS_TCX/tcx.h"