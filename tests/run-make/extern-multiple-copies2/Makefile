# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) foo1.rs
	$(CRABLANGC) foo2.rs
	mkdir $(TMPDIR)/foo
	cp $(TMPDIR)/libfoo1.rlib $(TMPDIR)/foo/libfoo1.rlib
	$(CRABLANGC) bar.rs \
		--extern foo1=$(TMPDIR)/foo/libfoo1.rlib \
		--extern foo2=$(TMPDIR)/libfoo2.rlib
