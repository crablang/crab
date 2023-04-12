# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) a.rs && $(CRABLANGC) b.rs
	$(BARE_CRABLANGC) c.rs -L dependency=$(TMPDIR) --extern b=$(TMPDIR)/libb.rlib \
		--out-dir=$(TMPDIR)
