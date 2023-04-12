#!/bin/sh
set -ex

mkdir /usr/local/mips-linux-musl

# originally from
# https://downloads.openwrt.org/snapshots/trunk/ar71xx/generic/
# OpenWrt-Toolchain-ar71xx-generic_gcc-5.3.0_musl-1.1.16.Linux-x86_64.tar.bz2
URL="https://ci-mirrors.crablang.org/crablangc"
FILE="OpenWrt-Toolchain-ar71xx-generic_gcc-5.3.0_musl-1.1.16.Linux-x86_64.tar.bz2"
curl -L "$URL/$FILE" | tar xjf - -C /usr/local/mips-linux-musl --strip-components=2

for file in /usr/local/mips-linux-musl/bin/mips-openwrt-linux-*; do
  ln -s $file /usr/local/bin/`basename $file`
done
