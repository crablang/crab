# ignore-cross-compile
include ../tools.mk

all: $(call NATIVE_STATICLIB,add)
	$(CRABLANGC) main.rs
	$(call RUN,main) || exit 1
