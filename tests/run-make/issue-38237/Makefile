# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) foo.rs; $(CRABLANGC) bar.rs
	$(CRABLANGDOC) baz.rs -L $(TMPDIR) -o $(TMPDIR)
