# ignore-cross-compile
include ../tools.mk
all:
	$(CRABLANGC) bar.rs --crate-type=rlib
	$(CRABLANGC) foo.rs
	$(call RUN,foo)
