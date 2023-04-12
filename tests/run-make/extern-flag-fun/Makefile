# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) bar.rs --crate-type=rlib
	$(CRABLANGC) bar.rs --crate-type=rlib -C extra-filename=-a
	$(CRABLANGC) bar-alt.rs --crate-type=rlib
	$(CRABLANGC) foo.rs --extern bar=no-exist && exit 1 || exit 0
	$(CRABLANGC) foo.rs --extern bar=foo.rs && exit 1 || exit 0
	$(CRABLANGC) foo.rs \
		--extern bar=$(TMPDIR)/libbar.rlib \
		--extern bar=$(TMPDIR)/libbar-alt.rlib \
		&& exit 1 || exit 0
	$(CRABLANGC) foo.rs \
		--extern bar=$(TMPDIR)/libbar.rlib \
		--extern bar=$(TMPDIR)/libbar-a.rlib
	$(CRABLANGC) foo.rs --extern bar=$(TMPDIR)/libbar.rlib
	# Try to be sneaky and load a private crate from with a non-private name.
	$(CRABLANGC) crablangc.rs -Zforce-unstable-if-unmarked --crate-type=rlib
	$(CRABLANGC) gated_unstable.rs --extern alloc=$(TMPDIR)/libcrablangc.rlib 2>&1 | $(CGREP) 'crablangc_private'
