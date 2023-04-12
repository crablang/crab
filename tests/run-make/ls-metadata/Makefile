# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) foo.rs
	$(CRABLANGC) -Z ls $(TMPDIR)/foo
	touch $(TMPDIR)/bar
	$(CRABLANGC) -Z ls $(TMPDIR)/bar
