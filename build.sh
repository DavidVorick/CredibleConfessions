#!/bin/sh

pushd ringsig
wasm-pack build --out-dir ../webapp/pkg --target no-modules
popd

