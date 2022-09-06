#!/bin/sh

set -e

cd ringsig
wasm-pack build --out-dir ../webapp/pkg --target no-modules
cd ..

