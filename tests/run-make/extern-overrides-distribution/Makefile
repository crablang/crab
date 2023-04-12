# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) libc.rs -Cmetadata=foo
	$(CRABLANGC) main.rs --extern libc=$(TMPDIR)/liblibc.rlib
