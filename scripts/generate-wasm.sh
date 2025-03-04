#!/bin/bash

SCRIPT=$(readlink -f "$0")
SCRIPT_DIR=$(dirname "$SCRIPT")
cd $SCRIPT_DIR/..

CANISTER_NAME=$1
PACKAGE="${CANISTER_NAME}_canister_impl"

if [ -z "${CARGO_HOME}" ]
then
  export CARGO_HOME="${HOME}/.cargo"
fi

if [ -z "${GIT_COMMIT_ID}" ]
then
  export GIT_COMMIT_ID=$(git rev-parse HEAD)
fi

echo Building package $PACKAGE
export RUSTFLAGS="--remap-path-prefix $(readlink -f ${SCRIPT_DIR}/..)=/build --remap-path-prefix ${CARGO_HOME}/bin=/cargo/bin --remap-path-prefix ${CARGO_HOME}/git=/cargo/git"
for l in $(ls ${CARGO_HOME}/registry/src/)
do
  export RUSTFLAGS="--remap-path-prefix ${CARGO_HOME}/registry/src/${l}=/cargo/registry/src/github ${RUSTFLAGS}"
done
cargo build --locked --target wasm32-unknown-unknown --release --package $PACKAGE || exit 1

echo Optimising wasm
if ! cargo install --list | grep -Fxq "ic-wasm v0.9.0:"
then
  echo Installing ic-wasm
  cargo install --version 0.9.0 ic-wasm || exit 1
fi
ic-wasm ./target/wasm32-unknown-unknown/release/$PACKAGE.wasm -o ./target/wasm32-unknown-unknown/release/$PACKAGE-opt.wasm shrink

echo Compressing wasm
mkdir -p wasms
gzip -fckn9 target/wasm32-unknown-unknown/release/$PACKAGE-opt.wasm > ./wasms/$CANISTER_NAME.wasm.gz
