#!/bin/bash

set -e # 发生错误时终止脚本
PROJECT_ROOT="$(pwd)/"

# 设置本地版本号（替代GitHub Actions中的version变量）
VERSION=$(cat "$PROJECT_ROOT/VERSION" 2>/dev/null || echo "1.0.0")
GIT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "local")
FULL_VERSION="${VERSION}+${GIT_COMMIT}"

# 设置目标目录
TARGET_DIR="$PROJECT_ROOT/publish/ios"
mkdir -p "$TARGET_DIR"

# 清理旧的构建产物
echo "=== 清理旧的构建产物 ==="
rm -rf "$PROJECT_ROOT/target"
mkdir -p "$PROJECT_ROOT/target"

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
pushd "$PROJECT_ROOT/imkey-core/ikc"
cargo lipo --release --targets aarch64-apple-ios
# 生成头文件
cbindgen ./src/lib.rs -l c > "$PROJECT_ROOT/target/connector.h"
popd

# 第二步：构建token-core库
echo "=== 构建token-core库 ==="
pushd "$PROJECT_ROOT/token-core/tcx"
cargo lipo --release --targets aarch64-apple-ios
# 生成头文件
cbindgen ./src/lib.rs -l c > "$PROJECT_ROOT/target/tcx.h"
popd

# 第三步：将构建结果复制到库目录
echo "=== 复制构建结果 ==="
# imkey-core库
LIBS_IKC="$PROJECT_ROOT/imkey-core/mobile-sdk/imKeyCoreX/imKeyCoreX"
mkdir -p "$LIBS_IKC"
cp "$PROJECT_ROOT/target/universal/release/libconnector.a" "$LIBS_IKC/libconnector.a"
cp "$PROJECT_ROOT/target/connector.h" "$LIBS_IKC/connector.h"
# token-core库
LIBS_TCX="$PROJECT_ROOT/token-core/tcx-examples/TokenCoreX/TokenCoreX"
mkdir -p "$LIBS_TCX"
cp "$PROJECT_ROOT/target/universal/release/libtcx.a" "$LIBS_TCX/libtcx.a"
cp "$PROJECT_ROOT/target/tcx.h" "$LIBS_TCX/tcx.h"

# 第四步：构建框架
echo "=== 构建iOS框架 ==="
# 设置构建变量
BUILD_DIR="./Products"
BUILD_ROOT="./Products"
SYMROOT="./Products"
BUILD_PRODUCTS="./Products"
CONFIGURATION="Release"
PROJECT_NAME_IKC="imKeyCoreX"
PROJECT_NAME_TCX="TokenCoreX"

# 构建imkey框架
if [ -d "$PROJECT_ROOT/imkey-core/mobile-sdk/imKeyCoreX" ]; then
    echo "=== 构建imKeyCoreX框架 ==="
    
    pushd "$PROJECT_ROOT/imkey-core/mobile-sdk/imKeyCoreX"
    
    mkdir -p $BUILD_DIR
    UNIVERSAL_OUTPUTFOLDER=$BUILD_DIR/$CONFIGURATION-Universal
    mkdir -p $UNIVERSAL_OUTPUTFOLDER
    
    # 构建真机版本
    xcodebuild -target $PROJECT_NAME_IKC ONLY_ACTIVE_ARCH=NO -configuration $CONFIGURATION -sdk iphoneos BUILD_DIR=$BUILD_DIR BUILD_ROOT=$BUILD_ROOT build
    
    # 构建模拟器版本（排除arm64架构）
    xcodebuild -target $PROJECT_NAME_IKC -configuration Debug -sdk iphonesimulator ONLY_ACTIVE_ARCH=NO BUILD_DIR=$BUILD_DIR BUILD_ROOT=$BUILD_ROOT EXCLUDED_ARCHS=arm64 build
    
    # 复制并合并架构
    cp -R $BUILD_DIR/$CONFIGURATION-iphoneos/$PROJECT_NAME_IKC.framework $UNIVERSAL_OUTPUTFOLDER/
    lipo -create -output $UNIVERSAL_OUTPUTFOLDER/$PROJECT_NAME_IKC.framework/$PROJECT_NAME_IKC $BUILD_PRODUCTS/Debug-iphonesimulator/$PROJECT_NAME_IKC.framework/$PROJECT_NAME_IKC $BUILD_DIR/$CONFIGURATION-iphoneos/$PROJECT_NAME_IKC.framework/$PROJECT_NAME_IKC
    
    # 复制到ios-release目录（修正路径）
    mkdir -p "$PROJECT_ROOT/imkey-core/ios-release"
    cp -R $UNIVERSAL_OUTPUTFOLDER/ "$PROJECT_ROOT/imkey-core/ios-release"
    
    rm -rf $UNIVERSAL_OUTPUTFOLDER 
    popd

    # 创建压缩包（修正变量引用）
    pushd "$PROJECT_ROOT/imkey-core/ios-release"
    PACKAGE_NAME="ios-ikc-${FULL_VERSION}.zip"
    zip -q -r "$PACKAGE_NAME" .
    echo "imKeyCoreX sha256: $(shasum -a 256 $PACKAGE_NAME | awk '{ print $1 }')"
    cp "$PACKAGE_NAME" "$PROJECT_ROOT/"
    popd
fi

# 构建token-core框架
if [ -d "$PROJECT_ROOT/token-core/tcx-examples/TokenCoreX" ]; then
    echo "=== 构建TokenCoreX框架 ==="
    
    pushd "$PROJECT_ROOT/token-core/tcx-examples/TokenCoreX"
    
    mkdir -p $BUILD_DIR
    UNIVERSAL_OUTPUTFOLDER=$BUILD_DIR/$CONFIGURATION-Universal
    mkdir -p $UNIVERSAL_OUTPUTFOLDER
    
    # 构建真机版本
    xcodebuild -target $PROJECT_NAME_TCX ONLY_ACTIVE_ARCH=NO -configuration $CONFIGURATION -sdk iphoneos BUILD_DIR=$BUILD_DIR BUILD_ROOT=$BUILD_ROOT build
    
    # 构建模拟器版本（排除arm64架构）
    xcodebuild -target $PROJECT_NAME_TCX -configuration Debug -sdk iphonesimulator ONLY_ACTIVE_ARCH=NO BUILD_DIR=$BUILD_DIR BUILD_ROOT=$BUILD_ROOT EXCLUDED_ARCHS=arm64 build

    # 复制并合并架构
    cp -R $BUILD_DIR/$CONFIGURATION-iphoneos/$PROJECT_NAME_TCX.framework $UNIVERSAL_OUTPUTFOLDER/
    lipo -create -output $UNIVERSAL_OUTPUTFOLDER/$PROJECT_NAME_TCX.framework/$PROJECT_NAME_TCX $BUILD_PRODUCTS/Debug-iphonesimulator/$PROJECT_NAME_TCX.framework/$PROJECT_NAME_TCX $BUILD_DIR/$CONFIGURATION-iphoneos/$PROJECT_NAME_TCX.framework/$PROJECT_NAME_TCX

    # 复制到ios-release目录（修正路径）
    mkdir -p "$PROJECT_ROOT/token-core/ios-release"
    cp -R $UNIVERSAL_OUTPUTFOLDER/ "$PROJECT_ROOT/token-core/ios-release"
    
    rm -rf $UNIVERSAL_OUTPUTFOLDER
    popd

    # 创建压缩包（修正变量引用）
    pushd "$PROJECT_ROOT/token-core/ios-release"
    PACKAGE_NAME="ios-tcx-${FULL_VERSION}.zip"
    zip -q -r "$PACKAGE_NAME" .
    echo "TokenCoreX sha256: $(shasum -a 256 $PACKAGE_NAME | awk '{ print $1 }')"
    cp "$PACKAGE_NAME" "$PROJECT_ROOT/"
    popd
fi