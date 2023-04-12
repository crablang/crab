# ignore-cross-compile
# ignore-i686-pc-windows-gnu

# This test doesn't work on 32-bit MinGW as cdylib has its own copy of unwinder
# so cross-DLL unwinding does not work.

include ../tools.mk

all:
	$(CRABLANGC) bar.rs --crate-type=cdylib
	$(CRABLANGC) foo.rs
	$(call RUN,foo) 2>&1 | $(CGREP) "CrabLang cannot catch foreign exceptions"
