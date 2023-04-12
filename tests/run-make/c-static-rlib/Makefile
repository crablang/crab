# ignore-cross-compile
include ../tools.mk

all: $(call NATIVE_STATICLIB,cfoo)
	$(CRABLANGC) foo.rs
	$(CRABLANGC) bar.rs
	$(call REMOVE_RLIBS,foo)
	rm $(call NATIVE_STATICLIB,cfoo)
	$(call RUN,bar)
