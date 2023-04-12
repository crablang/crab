# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) rlib.rs
	$(CRABLANGC) dylib.rs
	$(CRABLANGC) rlib.rs --crate-type=dylib
	$(CRABLANGC) dylib.rs
	$(call REMOVE_DYLIBS,rlib)
	$(CRABLANGC) prog.rs && exit 1 || exit 0
