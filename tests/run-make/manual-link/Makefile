# ignore-cross-compile
include ../tools.mk

all: $(TMPDIR)/libbar.a
	$(CRABLANGC) foo.rs -lstatic=bar
	$(CRABLANGC) main.rs
	$(call RUN,main)
