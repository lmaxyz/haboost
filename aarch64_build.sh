#!/bin/bash

set -e

if [ -z "$PSDK_DIR" ]; then
    echo '`PSDK_DIR` not found.';
    exit 1;
fi


CURRENT_DIR="$(pwd)";
TARGET=aarch64-unknown-linux-gnu

aurora_psdk="$PSDK_DIR/sdk-chroot"
PKG_VERSION="$(cargo pkgid | cut -d '#' -f 2)"

mkdir -p RPMS/

# Uncomment winit patch for Aurora build
sed -i 's/^# winit =/winit =/' Cargo.toml

# Build and ensure patch is restored even on failure
cleanup() {
    sed -i 's/^winit =/# winit =/' Cargo.toml
}
trap cleanup EXIT

cross build --release --target $TARGET
cargo generate-rpm -a aarch64 --target $TARGET -o RPMS/

$aurora_psdk rpmsign-external sign -k $PSDK_DIR/../../certs/lmaxyz_key.pem -c $PSDK_DIR/../../certs/lmaxyz_cert.pem "$CURRENT_DIR/RPMS/com.lmaxyz.Haboost-$PKG_VERSION-1.aarch64.rpm"
