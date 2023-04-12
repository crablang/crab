# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) foo.rs -C metadata=a -C extra-filename=-a
	$(CRABLANGC) foo.rs -C metadata=b -C extra-filename=-b
	$(CRABLANGC) bar.rs \
		--extern foo1=$(TMPDIR)/libfoo-a.rlib \
		--extern foo2=$(TMPDIR)/libfoo-b.rlib \
		--print link-args
	$(call RUN,bar)
