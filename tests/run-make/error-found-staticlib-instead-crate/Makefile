include ../tools.mk

all:
	$(CRABLANGC) foo.rs --crate-type staticlib
	$(CRABLANGC) bar.rs 2>&1 | $(CGREP) "found staticlib"
