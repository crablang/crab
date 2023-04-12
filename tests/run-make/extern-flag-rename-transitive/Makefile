include ../tools.mk

all:
	$(CRABLANGC) foo.rs
	$(CRABLANGC) bar.rs
	$(CRABLANGC) baz.rs --extern a=$(TMPDIR)/libfoo.rlib

