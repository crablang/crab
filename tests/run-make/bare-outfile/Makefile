# ignore-cross-compile
include ../tools.mk

all:
	cp foo.rs $(TMPDIR)
	cd $(TMPDIR) && $(CRABLANGC) -o foo foo.rs
	$(call RUN,foo)
