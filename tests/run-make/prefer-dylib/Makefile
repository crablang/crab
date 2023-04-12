# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) bar.rs --crate-type=dylib --crate-type=rlib -C prefer-dynamic
	$(CRABLANGC) foo.rs -C prefer-dynamic
	$(call RUN,foo)
	rm $(TMPDIR)/*bar*
	$(call FAIL,foo)
