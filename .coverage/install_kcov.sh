#!/bin/sh

set -eu

KCOV_VERSION=33

rm -rf v${KCOV_VERSION}.tar.gz kcov-${KCOV_VERSION}/

wget https://github.com/SimonKagstrom/kcov/archive/v${KCOV_VERSION}.tar.gz
tar xzf v${KCOV_VERSION}.tar.gz
cd kcov-${KCOV_VERSION}
mkdir build
cd build
cmake -DCMAKE_BUILD_TYPE=RelWithDebInfo ..
make
cp src/kcov src/libkcov_sowrapper.so ~/.cargo/bin
