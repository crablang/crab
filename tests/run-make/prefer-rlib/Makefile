# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) bar.rs --crate-type=dylib --crate-type=rlib
	ls $(TMPDIR)/$(call RLIB_GLOB,bar)
	$(CRABLANGC) foo.rs
	rm $(TMPDIR)/*bar*
	$(call RUN,foo)
