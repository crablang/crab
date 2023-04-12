# ignore-cross-compile
include ../tools.mk

all: $(call NATIVE_STATICLIB,foo)
	$(CRABLANGC) bar.rs
	$(call RUN,bar) || exit 1
