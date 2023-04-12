include ../tools.mk

DYLIB_NAME := $(shell echo | $(CRABLANGC) --crate-name foo --crate-type dylib --print file-names -)

all:
	echo >> $(TMPDIR)/$(DYLIB_NAME)
	$(CRABLANGC) --crate-type lib --extern foo=$(TMPDIR)/$(DYLIB_NAME) bar.rs 2>&1 | $(CGREP) 'invalid metadata files for crate `foo`'
