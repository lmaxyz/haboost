#!/bin/bash

set -e

source './.env';


if [ -z "$AURORA_SDK_PATH" ]; then
    echo 'No `AURORA_SDK_PATH` variable is set.';
    exit 1;
fi

if [ -z "$BUILD_TARGET" ]; then
    echo 'No `BUILD_TARGET` variable is set.';
    exit 1;
fi

if [ -z "$DEPLOY_DEVICE" ]; then
    echo 'No `DEPLOY_DEVICE` variable is set.';
    exit 1;
fi


SFDK_PATH="$AURORA_SDK_PATH/bin/sfdk";
CURRENT_DIR="$(pwd)";
SFDK_TARGET="AuroraOS-5.1.3.85-MB2-armv7hl"

# CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUSTFLAGS="link-args=-rpath -export-dynamic"

cross build --release --target armv7-unknown-linux-gnueabihf

cp ./target/armv7-unknown-linux-gnueabihf/release/habre ./com.lmaxyz.Habre

$SFDK_PATH config target="$SFDK_TARGET"

$SFDK_PATH build
$SFDK_PATH engine exec -tt sb2 -t "$SFDK_TARGET" rpmsign-external sign -k $CURRENT_DIR/../.auroraos-regular-keys/regular_key.pem -c $CURRENT_DIR/../.auroraos-regular-keys/regular_cert.pem $CURRENT_DIR/RPMS/com.lmaxyz.Habre-0.1-1.armv7hl.rpm

set +e

rm ./com.lmaxyz.Habre
rm ./documentation.list
