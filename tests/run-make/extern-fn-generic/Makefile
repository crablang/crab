# ignore-cross-compile
include ../tools.mk

all: $(call NATIVE_STATICLIB,test)
	$(CRABLANGC) testcrate.rs
	$(CRABLANGC) test.rs
	$(call RUN,test) || exit 1
