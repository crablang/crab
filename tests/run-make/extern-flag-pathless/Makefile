# ignore-cross-compile
include ../tools.mk

# Test mixing pathless --extern with paths.

all:
	$(CRABLANGC) bar-static.rs --crate-name=bar --crate-type=rlib
	$(CRABLANGC) bar-dynamic.rs --crate-name=bar --crate-type=dylib -C prefer-dynamic
	# rlib preferred over dylib
	$(CRABLANGC) foo.rs --extern bar
	$(call RUN,foo) | $(CGREP) 'static'
	$(CRABLANGC) foo.rs --extern bar=$(TMPDIR)/libbar.rlib --extern bar
	$(call RUN,foo) | $(CGREP) 'static'
	# explicit --extern overrides pathless
	$(CRABLANGC) foo.rs --extern bar=$(call DYLIB,bar) --extern bar
	$(call RUN,foo) | $(CGREP) 'dynamic'
	# prefer-dynamic does what it says
	$(CRABLANGC) foo.rs --extern bar -C prefer-dynamic
	$(call RUN,foo) | $(CGREP) 'dynamic'
