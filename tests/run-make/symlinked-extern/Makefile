# ignore-cross-compile
include ../tools.mk

# ignore-windows
# `ln` is actually `cp` on msys.

all:
	$(CRABLANGC) foo.rs
	mkdir -p $(TMPDIR)/other
	ln -nsf $(TMPDIR)/libfoo.rlib $(TMPDIR)/other
	$(CRABLANGC) bar.rs -L $(TMPDIR)
	$(CRABLANGC) baz.rs --extern foo=$(TMPDIR)/other/libfoo.rlib  -L $(TMPDIR)
