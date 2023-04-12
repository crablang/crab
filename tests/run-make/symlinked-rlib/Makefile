# ignore-cross-compile
include ../tools.mk

# ignore-windows
# `ln` is actually `cp` on msys.

all:
	$(CRABLANGC) foo.rs --crate-type=rlib -o $(TMPDIR)/foo.xxx
	ln -nsf $(TMPDIR)/foo.xxx $(TMPDIR)/libfoo.rlib
	$(CRABLANGC) bar.rs -L $(TMPDIR)
