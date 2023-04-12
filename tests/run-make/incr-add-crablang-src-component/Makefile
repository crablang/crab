# ignore-cross-compile
include ../tools.mk

# crablang/crablang#70924: Test that if we add crablang-src component in between two
# incremental compiles, the compiler does not ICE on the second.

# This test uses `ln -s` rather than copying to save testing time, but its
# usage doesn't work on windows. So ignore windows.

# ignore-windows

SYSROOT:=$(shell $(CRABLANGC) --print sysroot)
FAKEROOT=$(TMPDIR)/fakeroot
INCR=$(TMPDIR)/incr

# Make a local copy of the sysroot; then remove the crablang-src part of it, if
# present, for the *first* build. Then put in a facsimile of the crablang-src
# component for the second build, in order to expose the ICE from issue #70924.
#
# Note that it is much easier to just do `cp -a $(SYSROOT)/* $(FAKEROOT)` as a
# first step, but I am concerned that would be too expensive in a unit test
# compared to making symbolic links.
#
# Anyway, the pattern you'll see here is: For every prefix in
# root/lib/crablanglib/src, link all of prefix parent content, then remove the
# prefix, then loop on the next prefix. This way, we basically create a copy of
# the context around root/lib/crablanglib/src, and can freely add/remove the src
# component itself.
all:
	mkdir $(FAKEROOT)
	ln -s $(SYSROOT)/* $(FAKEROOT)
	rm -f $(FAKEROOT)/lib
	mkdir $(FAKEROOT)/lib
	ln -s $(SYSROOT)/lib/* $(FAKEROOT)/lib
	rm -f $(FAKEROOT)/lib/crablanglib
	mkdir $(FAKEROOT)/lib/crablanglib
	ln -s $(SYSROOT)/lib/crablanglib/* $(FAKEROOT)/lib/crablanglib
	rm -f $(FAKEROOT)/lib/crablanglib/src
	mkdir $(FAKEROOT)/lib/crablanglib/src
	ln -s $(SYSROOT)/lib/crablanglib/src/* $(FAKEROOT)/lib/crablanglib/src
	rm -f $(FAKEROOT)/lib/crablanglib/src/crablang
	$(CRABLANGC) --sysroot $(FAKEROOT) -C incremental=$(INCR) main.rs
	mkdir -p $(FAKEROOT)/lib/crablanglib/src/crablang/src/libstd
	touch $(FAKEROOT)/lib/crablanglib/src/crablang/src/libstd/lib.rs
	$(CRABLANGC) --sysroot $(FAKEROOT) -C incremental=$(INCR) main.rs
