# ignore-cross-compile
include ../tools.mk
all:
	$(CRABLANGC) bar.rs --crate-type=dylib -C prefer-dynamic
	$(CRABLANGC) foo.rs
	$(call RUN,foo)
