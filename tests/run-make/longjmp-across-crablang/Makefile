# ignore-cross-compile
include ../tools.mk

all: $(call NATIVE_STATICLIB,foo)
	$(CRABLANGC) main.rs
	$(call RUN,main)
