# ignore-cross-compile
include ../tools.mk

# ignore-musl
# ignore-windows
# Objects are reproducible but their path is not.

all:  \
	fat_lto \
	sysroot

fat_lto:
	rm -rf $(TMPDIR) && mkdir $(TMPDIR)
	$(CRABLANGC) reproducible-build-aux.rs
	$(CRABLANGC) reproducible-build.rs -C lto=fat
	cp $(TMPDIR)/reproducible-build $(TMPDIR)/reproducible-build-a
	$(CRABLANGC) reproducible-build.rs -C lto=fat
	cmp "$(TMPDIR)/reproducible-build-a" "$(TMPDIR)/reproducible-build" || exit 1

sysroot:
	rm -rf $(TMPDIR) && mkdir $(TMPDIR)
	$(CRABLANGC) reproducible-build-aux.rs
	$(CRABLANGC) reproducible-build.rs --crate-type rlib --sysroot $(shell $(CRABLANGC) --print sysroot) --remap-path-prefix=$(shell $(CRABLANGC) --print sysroot)=/sysroot
	cp -R $(shell $(CRABLANGC) --print sysroot) $(TMPDIR)/sysroot
	cp $(TMPDIR)/libreproducible_build.rlib $(TMPDIR)/libfoo.rlib
	$(CRABLANGC) reproducible-build.rs --crate-type rlib --sysroot $(TMPDIR)/sysroot --remap-path-prefix=$(TMPDIR)/sysroot=/sysroot
	cmp "$(TMPDIR)/libreproducible_build.rlib" "$(TMPDIR)/libfoo.rlib" || exit 1
