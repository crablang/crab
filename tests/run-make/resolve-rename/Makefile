include ../tools.mk

all:
	$(CRABLANGC) -C extra-filename=-hash foo.rs
	$(CRABLANGC) bar.rs
	mv $(TMPDIR)/libfoo-hash.rlib $(TMPDIR)/libfoo-another-hash.rlib
	$(CRABLANGC) baz.rs
