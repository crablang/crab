# ignore-cross-compile
include ../tools.mk

all: $(call NATIVE_STATICLIB,a) $(call NATIVE_STATICLIB,b)
	$(CRABLANGC) a.rs
	$(CRABLANGC) b.rs
	$(call RUN,b)
