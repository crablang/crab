#!/bin/sh
set -ex

groupadd -r crablangbuild && useradd -m -r -g crablangbuild crablangbuild
mkdir /x-tools && chown crablangbuild:crablangbuild /x-tools
