# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) foo-bar.rs --crate-type bin
	[ -f $(TMPDIR)/$(call BIN,foo-bar) ]
	$(CRABLANGC) foo-bar.rs --crate-type lib
	[ -f $(TMPDIR)/libfoo_bar.rlib ]
