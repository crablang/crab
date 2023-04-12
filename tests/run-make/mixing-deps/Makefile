# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) both.rs -C prefer-dynamic
	$(CRABLANGC) dylib.rs -C prefer-dynamic
	$(CRABLANGC) prog.rs
	$(call RUN,prog)
