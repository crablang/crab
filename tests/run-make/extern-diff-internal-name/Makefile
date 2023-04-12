# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) lib.rs
	$(CRABLANGC) test.rs --extern foo=$(TMPDIR)/libbar.rlib
