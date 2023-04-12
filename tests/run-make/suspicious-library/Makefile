# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) foo.rs -C prefer-dynamic
	touch $(call DYLIB,foo-something-special)
	touch $(call DYLIB,foo-something-special2)
	$(CRABLANGC) bar.rs
