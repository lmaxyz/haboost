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

cross build --release --target x86_64-unknown-linux-gnu

cp ./target/x86_64-unknown-linux-gnu/release/egui_demo ./com.lmaxyz.EguiAuroraExample

$SFDK_PATH config target="$BUILD_TARGET"
$SFDK_PATH config device="$DEPLOY_DEVICE"

$SFDK_PATH build
$SFDK_PATH engine exec -tt sb2 -t $BUILD_TARGET rpmsign-external sign -k $CURRENT_DIR/../.auroraos-regular-keys/regular_key.pem -c $CURRENT_DIR/../.auroraos-regular-keys/regular_cert.pem $CURRENT_DIR/RPMS/com.lmaxyz.EguiAuroraExample-0.2-1.x86_64.rpm

set +e

$SFDK_PATH deploy --sdk --silent RPMS/com.lmaxyz.EguiAuroraExample-0.2-1.x86_64.rpm
# scp -P 2223 -i $AURORA_SDK_PATH/vmshare/ssh/private_keys/sdk ./RPMS/com.lmaxyz.EguiAuroraExample-0.2-1.x86_64.rpm  root@127.0.0.1:/home/defaultuser/RPMS/
# ssh -p 2223 -i $AURORA_SDK_PATH/vmshare/ssh/private_keys/sdk root@127.0.0.1 "pkcon install-local -y /home/defaultuser/RPMS/com.lmaxyz.EguiAuroraExample-0.2-1.x86_64.rpm"

rm ./com.lmaxyz.EguiAuroraExample
rm ./documentation.list
