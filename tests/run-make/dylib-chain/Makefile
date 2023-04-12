# ignore-cross-compile
include ../tools.mk

all:
	$(CRABLANGC) m1.rs -C prefer-dynamic
	$(CRABLANGC) m2.rs -C prefer-dynamic
	$(CRABLANGC) m3.rs -C prefer-dynamic
	$(CRABLANGC) m4.rs
	$(call RUN,m4)
	$(call REMOVE_DYLIBS,m1)
	$(call REMOVE_DYLIBS,m2)
	$(call REMOVE_DYLIBS,m3)
	$(call FAIL,m4)
