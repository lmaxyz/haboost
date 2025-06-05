#!/bin/bash

set -e

if [ -z "$PSDK_DIR" ]; then
    echo '`PSDK_DIR` not found.';
    exit 1;
fi


CURRENT_DIR="$(pwd)";
TARGET=armv7-unknown-linux-gnueabihf

aurora_psdk="$PSDK_DIR/sdk-chroot"

mkdir -p RPMS/

cross build --release --target $TARGET
cargo generate-rpm -a armv7hl --target $TARGET -o RPMS/

$aurora_psdk rpmsign-external sign -k $PSDK_DIR/../../certs/lmaxyz_key.pem -c $PSDK_DIR/../../certs/lmaxyz_cert.pem $CURRENT_DIR/RPMS/com.lmaxyz.Haboost-0.1.0-1.armv7hl.rpm
