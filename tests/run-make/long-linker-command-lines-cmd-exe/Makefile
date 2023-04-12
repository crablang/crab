# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) foo.rs -g
	cp foo.bat $(TMPDIR)/
	OUT_DIR="$(TMPDIR)" CRABLANGC="$(CRABLANGC_ORIGINAL)" $(call RUN,foo)
