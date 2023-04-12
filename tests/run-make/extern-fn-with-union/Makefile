# ignore-cross-compile
include ../tools.mk

all: $(call NATIVE_STATICLIB,ctest)
	$(CRABLANGC) testcrate.rs
	$(CRABLANGC) test.rs
	$(call RUN,test) || exit 1
