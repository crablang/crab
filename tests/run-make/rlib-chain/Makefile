# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) m1.rs
	$(CRABLANGC) m2.rs
	$(CRABLANGC) m3.rs
	$(CRABLANGC) m4.rs
	$(call RUN,m4)
	rm $(TMPDIR)/*lib
	$(call RUN,m4)
