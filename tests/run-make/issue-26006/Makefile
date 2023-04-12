# ignore-cross-compile
include ../tools.mk

# ignore-windows

OUT := $(TMPDIR)/out

all: time

time: libc
	mkdir -p $(OUT)/time $(OUT)/time/deps
	ln -sf $(OUT)/libc/liblibc.rlib $(OUT)/time/deps/
	$(CRABLANGC) in/time/lib.rs -Ldependency=$(OUT)/time/deps/

libc:
	mkdir -p $(OUT)/libc
	$(CRABLANGC) in/libc/lib.rs --crate-name=libc -Cmetadata=foo -o $(OUT)/libc/liblibc.rlib
