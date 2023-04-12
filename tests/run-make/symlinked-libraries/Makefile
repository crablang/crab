# ignore-cross-compile
include ../tools.mk

# ignore-windows
# `ln` is actually `cp` on msys.

all:
	$(CRABLANGC) foo.rs -C prefer-dynamic
	mkdir -p $(TMPDIR)/other
	ln -nsf $(TMPDIR)/$(call DYLIB_GLOB,foo) $(TMPDIR)/other
	$(CRABLANGC) bar.rs -L $(TMPDIR)/other
