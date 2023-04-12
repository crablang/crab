# ignore-cross-compile
include ../tools.mk

all: $(call NATIVE_STATICLIB,foo) $(call NATIVE_STATICLIB,bar)
	$(CRABLANGC) lib1.rs
	$(CRABLANGC) lib2.rs
	$(CRABLANGC) main.rs -Clto
	$(call RUN,main)

