# ignore-cross-compile
include ../tools.mk

all:
	echo 'fn main(){}' | $(CRABLANGC) -Z no-link -
	$(CRABLANGC) -Z link-only $(TMPDIR)/crablang_out.rlink
	$(call RUN,crablang_out)
