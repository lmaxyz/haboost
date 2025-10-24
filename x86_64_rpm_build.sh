#!/bin/bash

set -e

if [ -z "$PSDK_DIR" ]; then
    echo '`PSDK_DIR` not found.';
    exit 1;
fi


CURRENT_DIR="$(pwd)";
TARGET=x86_64-unknown-linux-gnu

aurora_psdk="$PSDK_DIR/sdk-chroot"
PKG_VERSION="$(cargo pkgid | cut -d @ -f 2)"

mkdir -p RPMS/

cross build --release --target $TARGET
cargo generate-rpm -a x86_64 --target $TARGET -o RPMS/

$aurora_psdk rpmsign-external sign -k $PSDK_DIR/../../certs/lmaxyz_key.pem -c $PSDK_DIR/../../certs/lmaxyz_cert.pem "$CURRENT_DIR/RPMS/com.lmaxyz.Haboost-$PKG_VERSION-1.x86_64.rpm"
