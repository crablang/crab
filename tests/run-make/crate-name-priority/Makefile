# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) foo.rs
	rm $(TMPDIR)/$(call BIN,foo)
	$(CRABLANGC) foo.rs --crate-name bar
	rm $(TMPDIR)/$(call BIN,bar)
	$(CRABLANGC) foo1.rs
	rm $(TMPDIR)/$(call BIN,foo)
	$(CRABLANGC) foo1.rs -o $(TMPDIR)/$(call BIN,bar1)
	rm $(TMPDIR)/$(call BIN,bar1)
