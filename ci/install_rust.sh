#!/bin/bash

# Fail fast if any commands exists with error
set -e

# Print all executed commands
set -x

# Download rustup script and execute it
curl https://sh.rustup.rs -sSf > ./rustup.sh
chmod +x ./rustup.sh
./rustup.sh -y

# Load new environment
source $HOME/.cargo/env

# Install nightly and beta toolchains, but set stable as a default
rustup install nightly
rustup install beta
rustup default stable

# Install aux components, clippy for linter, rustfmt for formatting
rustup component add clippy
rustup component add rustfmt
