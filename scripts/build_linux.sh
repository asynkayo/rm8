#!/bin/bash

# exit when any command fails
set -e

apt update
apt install -y libsdl2-dev libudev-dev cargo zip

cargo build --release
cp target/release/rm8 .
zip -r "${1}" rm8 *.json README.md
